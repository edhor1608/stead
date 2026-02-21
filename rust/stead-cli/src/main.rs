use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use stead_contracts::{Contract, ContractStatus};
use stead_daemon::{ApiError, ApiRequest, ApiResponse, AttentionCounts, Daemon, API_VERSION};
use stead_endpoints::{EndpointClaimResult, EndpointLease};
use stead_module_sdk::{
    project_endpoint_name, ContextFragment, ContextGenerator, ContextProvider,
    ContextProviderError, ModuleManager, ModuleName,
};
use stead_resources::{ClaimResult, ResourceKey};
use stead_usf::{
    query_sessions, CliType, ClaudeAdapter, CodexAdapter, OpenCodeAdapter, SessionAdapter,
    SessionRecord,
};

#[derive(Parser, Debug)]
#[command(name = "stead")]
#[command(version = "0.2.0")]
#[command(about = "Operating environment for agent-driven development")]
struct Cli {
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Option<CommandFamily>,
}

#[derive(Subcommand, Debug)]
enum CommandFamily {
    Contract {
        #[command(subcommand)]
        command: ContractCommand,
    },
    Session {
        #[command(subcommand)]
        command: SessionCommand,
    },
    Resource {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    Attention {
        #[command(subcommand)]
        command: AttentionCommand,
    },
    Context {
        #[command(subcommand)]
        command: ContextCommand,
    },
    Module {
        #[command(subcommand)]
        command: ModuleCommand,
    },
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ContractCommand {
    Create {
        #[arg(long)]
        id: String,
        #[arg(long = "blocked-by")]
        blocked_by: Vec<String>,
    },
    Get {
        id: String,
    },
    List,
    Transition {
        id: String,
        #[arg(long)]
        to: String,
    },
}

#[derive(Subcommand, Debug)]
enum SessionCommand {
    List {
        #[arg(long)]
        cli: Option<String>,
        #[arg(long)]
        query: Option<String>,
    },
    Endpoint {
        #[arg(long)]
        project: String,
        #[arg(long)]
        owner: String,
    },
    Show {
        id: String,
    },
    Parse {
        #[arg(long)]
        cli: String,
        #[arg(long)]
        file: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum ResourceCommand {
    Claim {
        #[arg(long)]
        resource: String,
        #[arg(long)]
        owner: String,
    },
    Endpoint {
        #[command(subcommand)]
        command: EndpointCommand,
    },
}

#[derive(Subcommand, Debug)]
enum EndpointCommand {
    Claim {
        #[arg(long)]
        name: String,
        #[arg(long)]
        owner: String,
        #[arg(long)]
        port: Option<u16>,
    },
    List,
    Release {
        #[arg(long)]
        name: String,
        #[arg(long)]
        owner: String,
    },
}

#[derive(Subcommand, Debug)]
enum AttentionCommand {
    Status,
}

#[derive(Subcommand, Debug)]
enum ContextCommand {
    Generate {
        #[arg(long)]
        task: String,
        #[arg(long = "fragment")]
        fragment: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
enum ModuleCommand {
    List,
    Enable { name: String },
    Disable { name: String },
}

#[derive(Subcommand, Debug)]
enum DaemonCommand {
    Health,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => print_status_overview(cli.json),
        Some(CommandFamily::Daemon { command }) => handle_daemon(command, cli.json),
        Some(CommandFamily::Contract { command }) => handle_contract(command, cli.json),
        Some(CommandFamily::Resource { command }) => handle_resource(command, cli.json),
        Some(CommandFamily::Attention { command }) => handle_attention(command, cli.json),
        Some(CommandFamily::Context { command }) => handle_context(command, cli.json),
        Some(CommandFamily::Module { command }) => handle_module(command, cli.json),
        Some(CommandFamily::Session { command }) => handle_session(command, cli.json),
    }
}

fn print_status_overview(json_output: bool) -> Result<()> {
    let daemon = daemon_from_cwd()?;
    let health = daemon_handle(&daemon, ApiRequest::Health)?;
    let attention = daemon_handle(&daemon, ApiRequest::AttentionStatus)?;

    let status = match health {
        ApiResponse::Health { status } => status,
        _ => bail!("invalid daemon health response"),
    };

    let counts = match attention {
        ApiResponse::Attention(counts) => counts,
        _ => bail!("invalid daemon attention response"),
    };

    let payload = json!({
        "status": status,
        "daemon": "ok",
        "attention": counts.needs_decision + counts.anomaly,
        "decisions": counts.needs_decision,
        "anomalies": counts.anomaly,
    });

    if json_output {
        println!("{}", payload);
    } else {
        println!("Stead Status");
        println!("Daemon: ok");
        println!("Attention: {}", payload["attention"]);
        println!("Decisions: {}", payload["decisions"]);
        println!("Anomalies: {}", payload["anomalies"]);
    }

    Ok(())
}

fn handle_daemon(command: DaemonCommand, json_output: bool) -> Result<()> {
    let daemon = daemon_from_cwd()?;

    match command {
        DaemonCommand::Health => {
            let response = daemon_handle(&daemon, ApiRequest::Health)?;
            let status = match response {
                ApiResponse::Health { status } => status,
                _ => bail!("invalid daemon response"),
            };

            let payload = json!({
                "version": API_VERSION,
                "data": {
                    "status": status,
                }
            });

            if json_output {
                println!("{}", payload);
            } else {
                println!("Daemon {}", API_VERSION);
                println!("Health: {}", payload["data"]["status"]);
            }
        }
    }

    Ok(())
}

fn handle_contract(command: ContractCommand, json_output: bool) -> Result<()> {
    let daemon = daemon_from_cwd()?;

    match command {
        ContractCommand::Create { id, blocked_by } => {
            let response = daemon_handle(
                &daemon,
                ApiRequest::CreateContract {
                    id: id.clone(),
                    blocked_by,
                },
            )?;
            let contract = unwrap_contract_state(response)?;

            if json_output {
                println!("{}", contract_to_json(&contract));
            } else {
                println!("Created contract {}", contract.id);
            }
        }
        ContractCommand::Get { id } => {
            let response = daemon_handle(&daemon, ApiRequest::GetContract { id })?;
            let contract = unwrap_contract_state(response)?;

            if json_output {
                println!("{}", contract_to_json(&contract));
            } else {
                println!("{} [{}]", contract.id, status_to_str(contract.status));
            }
        }
        ContractCommand::List => {
            let response = daemon_handle(&daemon, ApiRequest::ListContracts)?;
            let contracts = match response {
                ApiResponse::Contracts(contracts) => contracts,
                _ => bail!("invalid daemon list response"),
            };

            if json_output {
                let out: Vec<Value> = contracts.iter().map(contract_to_json).collect();
                println!("{}", serde_json::to_string(&out)?);
            } else {
                for contract in contracts {
                    println!("{} [{}]", contract.id, status_to_str(contract.status));
                }
            }
        }
        ContractCommand::Transition { id, to } => {
            let to = parse_contract_status(&to)?;
            let response = daemon_handle(&daemon, ApiRequest::TransitionContract { id, to })?;
            let contract = unwrap_contract_state(response)?;

            if json_output {
                println!("{}", contract_to_json(&contract));
            } else {
                println!(
                    "Transitioned {} to {}",
                    contract.id,
                    status_to_str(contract.status)
                );
            }
        }
    }

    Ok(())
}

fn handle_resource(command: ResourceCommand, json_output: bool) -> Result<()> {
    let daemon = daemon_from_cwd()?;

    match command {
        ResourceCommand::Claim { resource, owner } => {
            let key = parse_resource_key(&resource)?;
            let response = daemon_handle(
                &daemon,
                ApiRequest::ClaimResource {
                    resource: key,
                    owner,
                },
            )?;
            let claim = match response {
                ApiResponse::ResourceClaim(claim) => claim,
                _ => bail!("invalid resource claim response"),
            };

            if json_output {
                println!("{}", claim_to_json(&claim));
            } else {
                println!("{:?}", claim);
            }
        }
        ResourceCommand::Endpoint { command } => handle_endpoint_command(&daemon, command, json_output)?,
    }

    Ok(())
}

fn handle_endpoint_command(daemon: &Daemon, command: EndpointCommand, json_output: bool) -> Result<()> {
    match command {
        EndpointCommand::Claim { name, owner, port } => {
            let response = match daemon_handle_raw(
                daemon,
                ApiRequest::ClaimEndpoint {
                    name,
                    owner,
                    port,
                },
            ) {
                Ok(response) => response,
                Err(error) => return render_daemon_error(error, json_output),
            };

            let claim = match response {
                ApiResponse::EndpointClaim(claim) => claim,
                _ => bail!("invalid endpoint claim response"),
            };

            if json_output {
                println!("{}", endpoint_claim_to_json(&claim));
            } else {
                match claim {
                    EndpointClaimResult::Claimed(lease) => {
                        println!("claimed {} -> {}", lease.name, lease.url());
                    }
                    EndpointClaimResult::Negotiated { assigned, .. } => {
                        println!("negotiated {} -> {}", assigned.name, assigned.url());
                    }
                    EndpointClaimResult::Conflict(conflict) => {
                        println!("conflict {}", conflict.name);
                    }
                }
            }
        }
        EndpointCommand::List => {
            let response = match daemon_handle_raw(daemon, ApiRequest::ListEndpoints) {
                Ok(response) => response,
                Err(error) => return render_daemon_error(error, json_output),
            };
            let leases = match response {
                ApiResponse::Endpoints(leases) => leases,
                _ => bail!("invalid endpoint list response"),
            };

            if json_output {
                let out: Vec<Value> = leases.iter().map(endpoint_lease_to_json).collect();
                println!("{}", serde_json::to_string(&out)?);
            } else {
                for lease in leases {
                    println!("{} {}", lease.name, lease.url());
                }
            }
        }
        EndpointCommand::Release { name, owner } => {
            let response = match daemon_handle_raw(daemon, ApiRequest::ReleaseEndpoint { name, owner }) {
                Ok(response) => response,
                Err(error) => return render_daemon_error(error, json_output),
            };
            let lease = match response {
                ApiResponse::EndpointReleased(lease) => lease,
                _ => bail!("invalid endpoint release response"),
            };

            if json_output {
                println!("{}", endpoint_lease_to_json(&lease));
            } else {
                println!("released {}", lease.name);
            }
        }
    }

    Ok(())
}

fn handle_attention(command: AttentionCommand, json_output: bool) -> Result<()> {
    let daemon = daemon_from_cwd()?;

    match command {
        AttentionCommand::Status => {
            let response = daemon_handle(&daemon, ApiRequest::AttentionStatus)?;
            let counts = match response {
                ApiResponse::Attention(counts) => counts,
                _ => bail!("invalid attention response"),
            };

            let payload = attention_to_json(&counts);
            if json_output {
                println!("{}", payload);
            } else {
                println!("needs_decision: {}", counts.needs_decision);
                println!("anomaly: {}", counts.anomaly);
                println!("completed: {}", counts.completed);
                println!("running: {}", counts.running);
                println!("queued: {}", counts.queued);
            }
        }
    }

    Ok(())
}

fn handle_context(command: ContextCommand, json_output: bool) -> Result<()> {
    match command {
        ContextCommand::Generate { task, fragment } => {
            let fragments = parse_fragments(&fragment)?;
            let generator = ContextGenerator::new(Box::new(StaticContextProvider), None);
            let context = generator.generate(&task, &fragments);

            let citations: Vec<Value> = context
                .citations
                .iter()
                .map(|citation| {
                    json!({
                        "source_id": citation.source_id,
                        "citation": citation.citation,
                    })
                })
                .collect();

            let payload = json!({
                "prompt": context.prompt,
                "content": context.content,
                "provider": context.provider,
                "citations": citations,
                "confidence": context.confidence,
                "used_fallback": context.used_fallback,
            });

            if json_output {
                println!("{}", payload);
            } else {
                println!("{}", payload["content"]);
            }
        }
    }

    Ok(())
}

fn handle_module(command: ModuleCommand, json_output: bool) -> Result<()> {
    let mut config = load_module_config()?;

    match command {
        ModuleCommand::List => {
            let payload = json!({
                "session_proxy": config.session_proxy,
                "context_generator": config.context_generator,
            });
            if json_output {
                println!("{}", payload);
            } else {
                println!(
                    "session_proxy={} context_generator={}",
                    config.session_proxy, config.context_generator
                );
            }
        }
        ModuleCommand::Enable { name } => {
            config.set(&name, true)?;
            save_module_config(&config)?;
            if !json_output {
                println!("enabled {name}");
            }
        }
        ModuleCommand::Disable { name } => {
            config.set(&name, false)?;
            save_module_config(&config)?;
            if !json_output {
                println!("disabled {name}");
            }
        }
    }

    Ok(())
}

fn handle_session(command: SessionCommand, json_output: bool) -> Result<()> {
    match command {
        SessionCommand::List { cli, query } => {
            let cli_filter = match cli.as_deref() {
                Some(raw) => Some(parse_cli_type(raw)?),
                None => None,
            };

            let sessions = load_sessions_from_workspace()?;
            let filtered = query_sessions(&sessions, cli_filter, query.as_deref());

            if json_output {
                let out: Vec<Value> = filtered.iter().map(session_record_to_json).collect();
                println!("{}", serde_json::to_string(&out)?);
            } else {
                for session in filtered {
                    println!("{:?} {} {}", session.cli, session.id, session.title);
                }
            }
        }
        SessionCommand::Endpoint { project, owner } => {
            let config = load_module_config()?;
            if !config.session_proxy {
                if json_output {
                    println!("null");
                } else {
                    println!("session_proxy module is disabled");
                }
                return Ok(());
            }

            let daemon = daemon_from_cwd()?;
            let endpoint_name = project_endpoint_name(&project);
            let response = match daemon_handle_raw(
                &daemon,
                ApiRequest::ClaimEndpoint {
                    name: endpoint_name,
                    owner,
                    port: None,
                },
            ) {
                Ok(response) => response,
                Err(error) => return render_daemon_error(error, json_output),
            };
            let claim = match response {
                ApiResponse::EndpointClaim(claim) => claim,
                _ => bail!("invalid session endpoint response"),
            };

            if json_output {
                println!("{}", endpoint_claim_to_json(&claim));
            } else {
                match claim {
                    EndpointClaimResult::Claimed(lease) => println!("{}", lease.url()),
                    EndpointClaimResult::Negotiated { assigned, .. } => {
                        println!("{}", assigned.url())
                    }
                    EndpointClaimResult::Conflict(conflict) => println!("conflict {}", conflict.name),
                }
            }
        }
        SessionCommand::Show { id } => {
            let sessions = load_sessions_from_workspace()?;
            let Some(record) = sessions.into_iter().find(|session| session.id == id) else {
                return render_json_error(
                    "not_found",
                    &format!("session not found: {id}"),
                    json_output,
                );
            };

            let payload = session_record_to_json(&record);
            if json_output {
                println!("{}", payload);
            } else {
                println!("{} {}", payload["cli"], payload["id"]);
            }
        }
        SessionCommand::Parse { cli, file } => {
            let raw = fs::read_to_string(&file)?;
            let record = parse_session_record(&cli, &raw)?;
            let payload = session_record_to_json(&record);

            if json_output {
                println!("{}", payload);
            } else {
                println!("{} {}", payload["cli"], payload["id"]);
            }
        }
    }

    Ok(())
}

fn daemon_from_cwd() -> Result<Daemon> {
    let cwd = env::current_dir()?;
    let stead_dir = cwd.join(".stead");
    fs::create_dir_all(&stead_dir)?;
    let db = stead_dir.join("stead.db");
    Daemon::new(db).map_err(|error| anyhow!("{}", error.message))
}

fn daemon_handle(daemon: &Daemon, request: ApiRequest) -> Result<ApiResponse> {
    daemon_handle_raw(daemon, request).map_err(|error| anyhow!("{}", error.message))
}

fn daemon_handle_raw(
    daemon: &Daemon,
    request: ApiRequest,
) -> std::result::Result<ApiResponse, ApiError> {
    daemon
        .handle(request)
        .map(|envelope| envelope.data)
}

fn render_daemon_error(error: ApiError, json_output: bool) -> Result<()> {
    render_json_error(error.code, &error.message, json_output)
}

fn render_json_error(code: &str, message: &str, json_output: bool) -> Result<()> {
    if json_output {
        println!(
            "{}",
            json!({
                "error": {
                    "code": code,
                    "message": message,
                }
            })
        );
    }

    bail!("{message}")
}

fn unwrap_contract_state(data: ApiResponse) -> Result<Contract> {
    match data {
        ApiResponse::ContractState(contract) => Ok(contract),
        _ => bail!("invalid contract response"),
    }
}

fn contract_to_json(contract: &Contract) -> Value {
    json!({
        "id": contract.id,
        "status": status_to_str(contract.status),
        "blocked_by": contract.blocked_by,
    })
}

fn attention_to_json(counts: &AttentionCounts) -> Value {
    json!({
        "needs_decision": counts.needs_decision,
        "anomaly": counts.anomaly,
        "completed": counts.completed,
        "running": counts.running,
        "queued": counts.queued,
    })
}

fn claim_to_json(claim: &ClaimResult) -> Value {
    match claim {
        ClaimResult::Claimed(lease) => json!({
            "Claimed": {
                "resource": resource_key_to_string(&lease.resource),
                "owner": lease.owner,
            }
        }),
        ClaimResult::Negotiated {
            requested,
            assigned,
            held_by,
        } => json!({
            "Negotiated": {
                "requested": resource_key_to_string(requested),
                "assigned": {
                    "resource": resource_key_to_string(&assigned.resource),
                    "owner": assigned.owner,
                },
                "held_by": {
                    "resource": resource_key_to_string(&held_by.resource),
                    "owner": held_by.owner,
                }
            }
        }),
        ClaimResult::Conflict(conflict) => json!({
            "Conflict": {
                "requested": resource_key_to_string(&conflict.requested),
                "held_by": {
                    "resource": resource_key_to_string(&conflict.held_by.resource),
                    "owner": conflict.held_by.owner,
                }
            }
        }),
    }
}

fn endpoint_claim_to_json(claim: &EndpointClaimResult) -> Value {
    match claim {
        EndpointClaimResult::Claimed(lease) => json!({
            "type": "claimed",
            "lease": endpoint_lease_to_json(lease),
        }),
        EndpointClaimResult::Negotiated {
            requested_port,
            assigned,
            held_by,
        } => json!({
            "type": "negotiated",
            "requested_port": requested_port,
            "lease": endpoint_lease_to_json(assigned),
            "held_by": endpoint_lease_to_json(held_by),
        }),
        EndpointClaimResult::Conflict(conflict) => json!({
            "type": "conflict",
            "name": conflict.name,
            "requested_port": conflict.requested_port,
            "held_by": conflict.held_by.as_ref().map(endpoint_lease_to_json),
        }),
    }
}

fn endpoint_lease_to_json(lease: &EndpointLease) -> Value {
    json!({
        "name": lease.name,
        "owner": lease.owner,
        "port": lease.port,
        "url": lease.url(),
    })
}

fn parse_contract_status(raw: &str) -> Result<ContractStatus> {
    match raw.to_ascii_lowercase().as_str() {
        "pending" => Ok(ContractStatus::Pending),
        "ready" => Ok(ContractStatus::Ready),
        "claimed" => Ok(ContractStatus::Claimed),
        "executing" => Ok(ContractStatus::Executing),
        "verifying" => Ok(ContractStatus::Verifying),
        "completed" => Ok(ContractStatus::Completed),
        "failed" => Ok(ContractStatus::Failed),
        "rolling_back" | "rollingback" => Ok(ContractStatus::RollingBack),
        "rolled_back" | "rolledback" => Ok(ContractStatus::RolledBack),
        "cancelled" | "canceled" => Ok(ContractStatus::Cancelled),
        _ => bail!("unknown status: {raw}"),
    }
}

fn status_to_str(status: ContractStatus) -> &'static str {
    match status {
        ContractStatus::Pending => "pending",
        ContractStatus::Ready => "ready",
        ContractStatus::Claimed => "claimed",
        ContractStatus::Executing => "executing",
        ContractStatus::Verifying => "verifying",
        ContractStatus::Completed => "completed",
        ContractStatus::Failed => "failed",
        ContractStatus::RollingBack => "rolling_back",
        ContractStatus::RolledBack => "rolled_back",
        ContractStatus::Cancelled => "cancelled",
    }
}

fn parse_resource_key(raw: &str) -> Result<ResourceKey> {
    let Some((kind, value)) = raw.split_once(':') else {
        bail!("resource must be in kind:value format")
    };

    match kind {
        "port" => Ok(ResourceKey::port(value.parse()?)),
        _ => bail!("unsupported resource kind: {kind}"),
    }
}

fn resource_key_to_string(key: &ResourceKey) -> String {
    match key {
        ResourceKey::Port(value) => format!("port:{value}"),
    }
}

fn parse_fragments(raw: &[String]) -> Result<Vec<ContextFragment>> {
    raw.iter()
        .map(|entry| {
            let mut parts = entry.splitn(3, '|');
            let source_id = parts
                .next()
                .ok_or_else(|| anyhow!("invalid fragment: {entry}"))?;
            let content = parts
                .next()
                .ok_or_else(|| anyhow!("invalid fragment: {entry}"))?;
            let citation = parts
                .next()
                .ok_or_else(|| anyhow!("invalid fragment: {entry}"))?;

            Ok(ContextFragment::new(source_id, content, citation))
        })
        .collect()
}

fn parse_session_record(cli: &str, raw: &str) -> Result<SessionRecord> {
    match cli {
        "claude" => ClaudeAdapter.parse(raw).map_err(to_anyhow),
        "codex" => CodexAdapter.parse(raw).map_err(to_anyhow),
        "opencode" => OpenCodeAdapter.parse(raw).map_err(to_anyhow),
        _ => bail!("unsupported cli: {cli}"),
    }
}

fn parse_cli_type(raw: &str) -> Result<CliType> {
    match raw.to_ascii_lowercase().as_str() {
        "claude" => Ok(CliType::Claude),
        "codex" => Ok(CliType::Codex),
        "opencode" => Ok(CliType::OpenCode),
        _ => bail!("unsupported cli: {raw}"),
    }
}

fn load_sessions_from_workspace() -> Result<Vec<SessionRecord>> {
    let root = env::current_dir()?.join(".stead").join("sessions");
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    collect_sessions_from_dir(&root.join("claude"), &ClaudeAdapter, &mut sessions)?;
    collect_sessions_from_dir(&root.join("codex"), &CodexAdapter, &mut sessions)?;
    collect_sessions_from_dir(&root.join("opencode"), &OpenCodeAdapter, &mut sessions)?;

    Ok(sessions)
}

fn collect_sessions_from_dir(
    dir: &Path,
    adapter: &dyn SessionAdapter,
    out: &mut Vec<SessionRecord>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .filter(|path| path.is_file())
        .collect();
    files.sort();

    for path in files {
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(record) = adapter.parse(&raw) else {
            continue;
        };
        out.push(record);
    }

    Ok(())
}

fn to_anyhow(error: stead_usf::UsfError) -> anyhow::Error {
    anyhow!("{}: {}", error.code(), error.message())
}

fn session_record_to_json(record: &SessionRecord) -> Value {
    json!({
        "cli": format!("{:?}", record.cli),
        "id": record.id,
        "project_path": record.project_path,
        "title": record.title,
        "updated_at": record.updated_at,
        "message_count": record.message_count,
    })
}

#[derive(Debug, Clone, Copy)]
struct StaticContextProvider;

impl ContextProvider for StaticContextProvider {
    fn name(&self) -> &'static str {
        "stead-static"
    }

    fn generate(&self, prompt: &str) -> std::result::Result<String, ContextProviderError> {
        Ok(format!("generated context for: {prompt}"))
    }
}

#[derive(Debug, Clone)]
struct ModuleConfig {
    session_proxy: bool,
    context_generator: bool,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        let manager = ModuleManager::default();
        Self {
            session_proxy: manager.is_enabled(ModuleName::SessionProxy),
            context_generator: manager.is_enabled(ModuleName::ContextGenerator),
        }
    }
}

impl ModuleConfig {
    fn set(&mut self, key: &str, enabled: bool) -> Result<()> {
        match key {
            "session_proxy" => self.session_proxy = enabled,
            "context_generator" => self.context_generator = enabled,
            _ => bail!("unknown module: {key}"),
        }
        Ok(())
    }
}

fn module_config_path() -> Result<PathBuf> {
    let cwd = env::current_dir()?;
    let stead_dir = cwd.join(".stead");
    fs::create_dir_all(&stead_dir)?;
    Ok(stead_dir.join("modules.json"))
}

fn load_module_config() -> Result<ModuleConfig> {
    let path = module_config_path()?;
    if !Path::new(&path).exists() {
        return Ok(ModuleConfig::default());
    }

    let raw = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&raw)?;

    Ok(ModuleConfig {
        session_proxy: value
            .get("session_proxy")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        context_generator: value
            .get("context_generator")
            .and_then(Value::as_bool)
            .unwrap_or(true),
    })
}

fn save_module_config(config: &ModuleConfig) -> Result<()> {
    let path = module_config_path()?;
    let value = json!({
        "session_proxy": config.session_proxy,
        "context_generator": config.context_generator,
    });
    fs::write(path, serde_json::to_string_pretty(&value)?)?;
    Ok(())
}
