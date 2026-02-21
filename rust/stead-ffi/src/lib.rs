//! stead FFI bindings via UniFFI

uniffi::setup_scaffolding!();

use std::fs;
use std::path::{Path, PathBuf};

use stead_contracts::{Contract, ContractStatus};
use stead_daemon::{ApiError, ApiRequest, ApiResponse, Daemon};
use stead_usf::{
    query_sessions, ClaudeAdapter, CliType, CodexAdapter, OpenCodeAdapter, SessionAdapter,
    SessionRecord,
};

// -- FFI Enum types --

#[derive(uniffi::Enum)]
pub enum FfiContractStatus {
    Pending,
    Ready,
    Claimed,
    Executing,
    Verifying,
    Completed,
    Failed,
    RollingBack,
    RolledBack,
    Cancelled,
}

impl From<ContractStatus> for FfiContractStatus {
    fn from(status: ContractStatus) -> Self {
        match status {
            ContractStatus::Pending => Self::Pending,
            ContractStatus::Ready => Self::Ready,
            ContractStatus::Claimed => Self::Claimed,
            ContractStatus::Executing => Self::Executing,
            ContractStatus::Verifying => Self::Verifying,
            ContractStatus::Completed => Self::Completed,
            ContractStatus::Failed => Self::Failed,
            ContractStatus::RollingBack => Self::RollingBack,
            ContractStatus::RolledBack => Self::RolledBack,
            ContractStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(uniffi::Enum)]
pub enum FfiCliType {
    Claude,
    Codex,
    OpenCode,
    Universal,
}

impl From<CliType> for FfiCliType {
    fn from(cli: CliType) -> Self {
        match cli {
            CliType::Claude => Self::Claude,
            CliType::Codex => Self::Codex,
            CliType::OpenCode => Self::OpenCode,
        }
    }
}

// -- FFI Record types --

#[derive(uniffi::Record)]
pub struct FfiContract {
    pub id: String,
    pub task: String,
    pub verification: String,
    pub status: FfiContractStatus,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub output: Option<String>,
    pub owner: Option<String>,
    pub blocked_by: Vec<String>,
    pub blocks: Vec<String>,
}

impl From<Contract> for FfiContract {
    fn from(contract: Contract) -> Self {
        Self {
            id: contract.id.clone(),
            task: contract.id,
            verification: String::new(),
            status: contract.status.into(),
            created_at: String::new(),
            completed_at: None,
            output: None,
            owner: None,
            blocked_by: contract.blocked_by,
            blocks: Vec::new(),
        }
    }
}

#[derive(uniffi::Record)]
pub struct FfiSessionSummary {
    pub id: String,
    pub cli: FfiCliType,
    pub project_path: String,
    pub title: String,
    pub created: String,
    pub last_modified: String,
    pub message_count: u32,
    pub git_branch: Option<String>,
}

impl From<SessionRecord> for FfiSessionSummary {
    fn from(session: SessionRecord) -> Self {
        let timestamp = session.updated_at.to_string();
        Self {
            id: session.id,
            cli: session.cli.into(),
            project_path: session.project_path,
            title: session.title,
            created: timestamp.clone(),
            last_modified: timestamp,
            message_count: session.message_count as u32,
            git_branch: None,
        }
    }
}

// -- FFI Error type --

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FfiError {
    #[error("Storage error: {message}")]
    Storage { message: String },
    #[error("Not found: {id}")]
    NotFound { id: String },
}

// -- Exported functions --

#[uniffi::export]
pub fn list_contracts(cwd: String) -> Result<Vec<FfiContract>, FfiError> {
    let daemon = daemon_for_workspace(&cwd)?;
    let response = daemon
        .handle(ApiRequest::ListContracts)
        .map_err(|error| map_daemon_error(error, None))?;

    match response.data {
        ApiResponse::Contracts(contracts) => {
            Ok(contracts.into_iter().map(FfiContract::from).collect())
        }
        _ => Err(FfiError::Storage {
            message: "invalid daemon contracts response".to_string(),
        }),
    }
}

#[uniffi::export]
pub fn get_contract(id: String, cwd: String) -> Result<FfiContract, FfiError> {
    let daemon = daemon_for_workspace(&cwd)?;
    let response = daemon
        .handle(ApiRequest::GetContract { id: id.clone() })
        .map_err(|error| map_daemon_error(error, Some(id.clone())))?;

    match response.data {
        ApiResponse::ContractState(contract) => Ok(FfiContract::from(contract)),
        _ => Err(FfiError::Storage {
            message: "invalid daemon contract response".to_string(),
        }),
    }
}

#[uniffi::export]
pub fn list_sessions(
    cli_filter: Option<String>,
    project: Option<String>,
    limit: u32,
) -> Vec<FfiSessionSummary> {
    let sessions = load_sessions_from_workspace();

    let cli_filter = match cli_filter {
        Some(raw) => {
            let Some(parsed) = parse_cli_filter(&raw) else {
                return Vec::new();
            };
            Some(parsed)
        }
        None => None,
    };

    let mut filtered = query_sessions(&sessions, cli_filter, None);

    if let Some(project_filter) = project {
        let needle = project_filter.to_ascii_lowercase();
        filtered.retain(|session| session.project_path.to_ascii_lowercase().contains(&needle));
    }

    filtered.truncate(limit as usize);
    filtered.into_iter().map(FfiSessionSummary::from).collect()
}

fn daemon_for_workspace(cwd: &str) -> Result<Daemon, FfiError> {
    let stead_dir = Path::new(cwd).join(".stead");
    fs::create_dir_all(&stead_dir).map_err(|error| FfiError::Storage {
        message: error.to_string(),
    })?;

    let db = stead_dir.join("stead.db");
    Daemon::new(db).map_err(|error| map_daemon_error(error, None))
}

fn map_daemon_error(error: ApiError, not_found_id: Option<String>) -> FfiError {
    if error.code == "not_found" {
        if let Some(id) = not_found_id {
            return FfiError::NotFound { id };
        }
    }

    FfiError::Storage {
        message: error.message,
    }
}

fn parse_cli_filter(raw: &str) -> Option<CliType> {
    match raw.to_ascii_lowercase().as_str() {
        "claude" => Some(CliType::Claude),
        "codex" => Some(CliType::Codex),
        "opencode" => Some(CliType::OpenCode),
        _ => None,
    }
}

fn load_sessions_from_workspace() -> Vec<SessionRecord> {
    let Ok(cwd) = std::env::current_dir() else {
        return Vec::new();
    };

    let root = cwd.join(".stead").join("sessions");
    let mut sessions = Vec::new();

    collect_sessions_from_dir(&root.join("claude"), &ClaudeAdapter, &mut sessions);
    collect_sessions_from_dir(&root.join("codex"), &CodexAdapter, &mut sessions);
    collect_sessions_from_dir(&root.join("opencode"), &OpenCodeAdapter, &mut sessions);

    sessions
}

fn collect_sessions_from_dir(
    dir: &Path,
    adapter: &dyn SessionAdapter,
    out: &mut Vec<SessionRecord>,
) {
    if !dir.exists() {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    let mut files: Vec<PathBuf> = entries
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
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use stead_daemon::{ApiRequest, Daemon};
    use tempfile::TempDir;

    use super::{get_contract, list_contracts, list_sessions, FfiError};

    struct CurrentDirGuard {
        previous: std::path::PathBuf,
    }

    impl CurrentDirGuard {
        fn enter(path: &Path) -> Self {
            let previous = std::env::current_dir().expect("current dir should be readable");
            std::env::set_current_dir(path).expect("set_current_dir should succeed");
            Self { previous }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.previous);
        }
    }

    #[test]
    fn list_contracts_and_get_contract_read_from_daemon_store() {
        let tmp = TempDir::new().expect("temp dir should exist");
        let stead_dir = tmp.path().join(".stead");
        std::fs::create_dir_all(&stead_dir).expect(".stead dir should be created");
        let daemon = Daemon::new(stead_dir.join("stead.db")).expect("daemon should initialize");

        daemon
            .handle(ApiRequest::CreateContract {
                id: "ffi-c-1".to_string(),
                blocked_by: vec![],
            })
            .expect("contract should be created");

        let listed = list_contracts(tmp.path().to_string_lossy().to_string())
            .expect("list_contracts should succeed");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "ffi-c-1");

        let fetched = get_contract(
            "ffi-c-1".to_string(),
            tmp.path().to_string_lossy().to_string(),
        )
        .expect("get_contract should succeed");
        assert_eq!(fetched.id, "ffi-c-1");
    }

    #[test]
    fn get_contract_not_found_is_typed() {
        let tmp = TempDir::new().expect("temp dir should exist");

        let result = get_contract(
            "missing".to_string(),
            tmp.path().to_string_lossy().to_string(),
        );

        match result {
            Err(FfiError::NotFound { id }) => assert_eq!(id, "missing"),
            Err(other) => panic!("unexpected error variant: {other}"),
            Ok(_) => panic!("expected not found error"),
        }
    }

    #[test]
    fn list_sessions_reads_workspace_local_session_fixtures() {
        let tmp = TempDir::new().expect("temp dir should exist");
        let _cwd = CurrentDirGuard::enter(tmp.path());

        let root = tmp.path().join(".stead").join("sessions").join("claude");
        std::fs::create_dir_all(&root).expect("sessions dir should be created");

        let fixture_path = root.join("session.json");
        std::fs::write(
            fixture_path,
            r#"{
  "session_id":"ffi-s-1",
  "project_path":"/__ffi_workspace_unique__/project",
  "updated_at":1700000001,
  "messages":[{"role":"user","content":"Hello"}]
}"#,
        )
        .expect("fixture should be written");

        let sessions = list_sessions(
            Some("claude".to_string()),
            Some("__ffi_workspace_unique__".to_string()),
            10,
        );

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "ffi-s-1");
    }

    #[test]
    fn ffi_manifest_depends_on_rewrite_crates_not_stead_core() {
        let manifest =
            std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
                .expect("Cargo.toml should exist");

        assert!(manifest.contains("stead-daemon"));
        assert!(manifest.contains("stead-usf"));
        assert!(
            !manifest.contains("stead-core"),
            "stead-ffi should not depend on stead-core in rewrite surface"
        );
    }
}
