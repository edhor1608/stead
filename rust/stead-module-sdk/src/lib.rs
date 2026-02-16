use std::collections::{HashMap, HashSet};

use stead_endpoints::{EndpointClaimResult, EndpointRegistry};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionIdentity(String);

impl SessionIdentity {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken {
    project: String,
    identity: SessionIdentity,
    nonce: u64,
}

impl SessionToken {
    pub fn project(&self) -> &str {
        &self.project
    }

    pub fn identity(&self) -> &SessionIdentity {
        &self.identity
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionProxyError {
    ProjectIsolationViolation,
    UnknownIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEndpoint {
    pub project: String,
    pub name: String,
    pub owner: String,
    pub port: u16,
    pub url: String,
}

#[derive(Debug, Default)]
pub struct SessionProxy {
    next_identity: u64,
    next_token: u64,
    identities_by_project: HashMap<String, HashSet<SessionIdentity>>,
    endpoint_registry: EndpointRegistry,
}

impl SessionProxy {
    pub fn create_identity(&mut self, project: impl Into<String>) -> SessionIdentity {
        let project = project.into();
        self.next_identity += 1;
        let identity = SessionIdentity(format!("{}-identity-{}", project, self.next_identity));
        self.identities_by_project
            .entry(project)
            .or_default()
            .insert(identity.clone());
        identity
    }

    pub fn issue_token(
        &mut self,
        project: impl Into<String>,
        identity: &SessionIdentity,
    ) -> Result<SessionToken, SessionProxyError> {
        let project = project.into();

        if !self.identity_exists(&project, identity) {
            return Err(SessionProxyError::UnknownIdentity);
        }

        self.next_token += 1;
        Ok(SessionToken {
            project,
            identity: identity.clone(),
            nonce: self.next_token,
        })
    }

    pub fn validate_token(
        &self,
        project: impl AsRef<str>,
        token: &SessionToken,
    ) -> Result<SessionIdentity, SessionProxyError> {
        let project = project.as_ref();
        if token.project != project {
            return Err(SessionProxyError::ProjectIsolationViolation);
        }

        if !self.identity_exists(project, &token.identity) {
            return Err(SessionProxyError::UnknownIdentity);
        }

        Ok(token.identity.clone())
    }

    pub fn destroy_identity(&mut self, project: impl AsRef<str>, identity: &SessionIdentity) {
        if let Some(identities) = self.identities_by_project.get_mut(project.as_ref()) {
            identities.remove(identity);
        }
    }

    pub fn resolve_project_endpoint(
        &mut self,
        modules: &ModuleManager,
        project: impl AsRef<str>,
        owner: impl Into<String>,
    ) -> Result<Option<SessionEndpoint>, ModuleError> {
        if !modules.is_enabled(ModuleName::SessionProxy) {
            return Ok(None);
        }

        let project = project.as_ref();
        let owner = owner.into();
        let endpoint_name = project_endpoint_name(project);
        let claim = self
            .endpoint_registry
            .claim(endpoint_name, owner, None);

        let lease = match claim {
            EndpointClaimResult::Claimed(lease) => lease,
            EndpointClaimResult::Negotiated { assigned, .. } => assigned,
            EndpointClaimResult::Conflict(_) => return Ok(None),
        };

        Ok(Some(SessionEndpoint {
            project: project.to_string(),
            name: lease.name.clone(),
            owner: lease.owner.clone(),
            port: lease.port,
            url: lease.url(),
        }))
    }

    fn identity_exists(&self, project: &str, identity: &SessionIdentity) -> bool {
        self.identities_by_project
            .get(project)
            .map(|identities| identities.contains(identity))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleName {
    SessionProxy,
    ContextGenerator,
}

impl ModuleName {
    fn as_str(self) -> &'static str {
        match self {
            Self::SessionProxy => "session_proxy",
            Self::ContextGenerator => "context_generator",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleError {
    ModuleDisabled(ModuleName),
}

#[derive(Debug, Clone)]
pub struct ModuleManager {
    enabled: HashSet<ModuleName>,
}

impl Default for ModuleManager {
    fn default() -> Self {
        let mut enabled = HashSet::new();
        enabled.insert(ModuleName::SessionProxy);
        enabled.insert(ModuleName::ContextGenerator);
        Self { enabled }
    }
}

impl ModuleManager {
    pub fn is_enabled(&self, module: ModuleName) -> bool {
        self.enabled.contains(&module)
    }

    pub fn enable(&mut self, module: ModuleName) {
        self.enabled.insert(module);
    }

    pub fn disable(&mut self, module: ModuleName) {
        self.enabled.remove(&module);
    }

    pub fn ensure_enabled(&self, module: ModuleName) -> Result<(), ModuleError> {
        if self.is_enabled(module) {
            Ok(())
        } else {
            Err(ModuleError::ModuleDisabled(module))
        }
    }

    pub fn run_core_operation<T>(&self, operation: impl FnOnce() -> T) -> T {
        operation()
    }

    pub fn module_key(module: ModuleName) -> &'static str {
        module.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextFragment {
    pub source_id: String,
    pub content: String,
    pub citation: String,
}

impl ContextFragment {
    pub fn new(
        source_id: impl Into<String>,
        content: impl Into<String>,
        citation: impl Into<String>,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            content: content.into(),
            citation: citation.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextCitation {
    pub source_id: String,
    pub citation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextProviderError {
    Unavailable,
    Failed(String),
}

pub trait ContextProvider {
    fn name(&self) -> &'static str;
    fn generate(&self, prompt: &str) -> Result<String, ContextProviderError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedContext {
    pub prompt: String,
    pub content: String,
    pub provider: String,
    pub used_fallback: bool,
    pub citations: Vec<ContextCitation>,
    pub confidence: f32,
}

pub struct ContextGenerator {
    primary: Box<dyn ContextProvider>,
    fallback: Option<Box<dyn ContextProvider>>,
}

impl ContextGenerator {
    pub fn new(
        primary: Box<dyn ContextProvider>,
        fallback: Option<Box<dyn ContextProvider>>,
    ) -> Self {
        Self { primary, fallback }
    }

    pub fn assemble_prompt(&self, task: &str, fragments: &[ContextFragment]) -> String {
        let mut ordered = fragments.to_vec();
        ordered.sort_by(|left, right| left.source_id.cmp(&right.source_id));

        let mut lines = vec![format!("Task: {task}")];
        lines.extend(
            ordered
                .iter()
                .map(|fragment| format!("[{}] {}", fragment.source_id, fragment.content)),
        );

        lines.join("\n")
    }

    pub fn generate(&self, task: &str, fragments: &[ContextFragment]) -> GeneratedContext {
        let prompt = self.assemble_prompt(task, fragments);
        let citations = citations_from_fragments(fragments);

        match self.primary.generate(&prompt) {
            Ok(content) => GeneratedContext {
                prompt,
                content,
                provider: self.primary.name().to_string(),
                used_fallback: false,
                citations,
                confidence: 0.9,
            },
            Err(ContextProviderError::Unavailable) => {
                if let Some(fallback) = &self.fallback {
                    match fallback.generate(&prompt) {
                        Ok(content) => GeneratedContext {
                            prompt,
                            content,
                            provider: fallback.name().to_string(),
                            used_fallback: true,
                            citations,
                            confidence: 0.7,
                        },
                        Err(_) => deterministic_context_fallback(prompt, citations),
                    }
                } else {
                    deterministic_context_fallback(prompt, citations)
                }
            }
            Err(_) => deterministic_context_fallback(prompt, citations),
        }
    }
}

fn citations_from_fragments(fragments: &[ContextFragment]) -> Vec<ContextCitation> {
    let mut ordered = fragments.to_vec();
    ordered.sort_by(|left, right| left.source_id.cmp(&right.source_id));
    ordered
        .into_iter()
        .map(|fragment| ContextCitation {
            source_id: fragment.source_id,
            citation: fragment.citation,
        })
        .collect()
}

fn deterministic_context_fallback(
    prompt: String,
    citations: Vec<ContextCitation>,
) -> GeneratedContext {
    GeneratedContext {
        prompt,
        content: "fallback: deterministic context summary".to_string(),
        provider: "deterministic-fallback".to_string(),
        used_fallback: true,
        citations,
        confidence: 0.4,
    }
}

pub fn project_endpoint_name(project: &str) -> String {
    let mut normalized = project
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();

    while normalized.contains("--") {
        normalized = normalized.replace("--", "-");
    }
    let normalized = normalized.trim_matches('-');

    if normalized.is_empty() {
        "stead-project".to_string()
    } else {
        format!("stead-{}", normalized)
    }
}

pub fn crate_identity() -> &'static str {
    "stead-module-sdk"
}
