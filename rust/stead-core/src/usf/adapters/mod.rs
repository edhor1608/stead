//! Session adapters for different AI CLIs
//!
//! Each adapter converts from a CLI's native format to UniversalSession.

pub mod claude;
pub mod codex;
pub mod opencode;

use crate::usf::{SessionSummary, UniversalSession};
use thiserror::Error;

/// Adapter errors
#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Invalid session format: {0}")]
    InvalidFormat(String),

    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),
}

/// Common trait for session adapters
pub trait SessionAdapter {
    /// Get the CLI type this adapter handles
    fn cli_type(&self) -> crate::usf::CliType;

    /// Check if this adapter's CLI is installed/available
    fn is_available(&self) -> bool;

    /// Get the base directory for this CLI's data
    fn base_dir(&self) -> Option<std::path::PathBuf>;

    /// List all available sessions (summaries only for performance)
    fn list_sessions(&self) -> Result<Vec<SessionSummary>, AdapterError>;

    /// Load a full session by ID
    fn load_session(&self, id: &str) -> Result<UniversalSession, AdapterError>;
}

/// Discover all available sessions across all installed CLIs
pub fn discover_all_sessions() -> Vec<SessionSummary> {
    let mut sessions = Vec::new();

    // Try each adapter
    if let Some(adapter) = claude::ClaudeAdapter::new() {
        if let Ok(claude_sessions) = adapter.list_sessions() {
            sessions.extend(claude_sessions);
        }
    }

    if let Some(adapter) = codex::CodexAdapter::new() {
        if let Ok(codex_sessions) = adapter.list_sessions() {
            sessions.extend(codex_sessions);
        }
    }

    if let Some(adapter) = opencode::OpenCodeAdapter::new() {
        if let Ok(opencode_sessions) = adapter.list_sessions() {
            sessions.extend(opencode_sessions);
        }
    }

    // Sort by last_modified descending
    sessions.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    sessions
}

/// Load a session by CLI type and ID
pub fn load_session(cli: crate::usf::CliType, id: &str) -> Result<UniversalSession, AdapterError> {
    match cli {
        crate::usf::CliType::Claude => {
            claude::ClaudeAdapter::new()
                .ok_or_else(|| AdapterError::DirectoryNotFound("~/.claude not found".to_string()))?
                .load_session(id)
        }
        crate::usf::CliType::Codex => {
            codex::CodexAdapter::new()
                .ok_or_else(|| AdapterError::DirectoryNotFound("~/.codex not found".to_string()))?
                .load_session(id)
        }
        crate::usf::CliType::OpenCode => {
            opencode::OpenCodeAdapter::new()
                .ok_or_else(|| AdapterError::DirectoryNotFound("~/.local/share/opencode not found".to_string()))?
                .load_session(id)
        }
        crate::usf::CliType::Universal => {
            Err(AdapterError::InvalidFormat("Cannot load universal sessions directly".to_string()))
        }
    }
}

/// Try to load a session by ID, auto-detecting the CLI from the ID prefix
pub fn load_session_by_id(id: &str) -> Result<UniversalSession, AdapterError> {
    // ID format: cli-originalId
    if let Some((cli_str, _original_id)) = id.split_once('-') {
        let cli = match cli_str {
            "claude" => crate::usf::CliType::Claude,
            "codex" => crate::usf::CliType::Codex,
            "opencode" => crate::usf::CliType::OpenCode,
            _ => return Err(AdapterError::InvalidFormat(format!("Unknown CLI prefix: {}", cli_str))),
        };
        return load_session(cli, id);
    }

    // Try each adapter if no prefix
    if let Some(adapter) = claude::ClaudeAdapter::new() {
        if let Ok(session) = adapter.load_session(id) {
            return Ok(session);
        }
    }
    if let Some(adapter) = codex::CodexAdapter::new() {
        if let Ok(session) = adapter.load_session(id) {
            return Ok(session);
        }
    }
    if let Some(adapter) = opencode::OpenCodeAdapter::new() {
        if let Ok(session) = adapter.load_session(id) {
            return Ok(session);
        }
    }

    Err(AdapterError::NotFound(id.to_string()))
}

/// Helper to expand ~ in paths
pub(crate) fn expand_home(path: &str) -> Option<std::path::PathBuf> {
    if path.starts_with("~/") {
        dirs::home_dir().map(|home| home.join(&path[2..]))
    } else {
        Some(std::path::PathBuf::from(path))
    }
}
