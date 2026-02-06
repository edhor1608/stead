//! Codex CLI session adapter
//!
//! Parses sessions from ~/.codex/sessions/

use super::{expand_home, AdapterError, SessionAdapter};
use crate::usf::{
    AssistantMessage, CliType, GitInfo, ModelInfo, ProjectInfo, SessionMetadata, SessionSource,
    SessionSummary, TimelineEntry, ToolCall, ToolResult, UniversalSession, UniversalTool,
    UserMessage, USF_VERSION,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const CODEX_DIR: &str = "~/.codex";
const SESSIONS_DIR: &str = "sessions";

/// Codex CLI session adapter
pub struct CodexAdapter {
    base_dir: PathBuf,
}

impl CodexAdapter {
    /// Create a new adapter if Codex CLI is installed
    pub fn new() -> Option<Self> {
        let base_dir = expand_home(CODEX_DIR)?;
        if base_dir.join(SESSIONS_DIR).is_dir() {
            Some(Self { base_dir })
        } else {
            None
        }
    }

    fn sessions_dir(&self) -> PathBuf {
        self.base_dir.join(SESSIONS_DIR)
    }

    /// Parse a session JSONL file
    fn parse_session_file(&self, path: &PathBuf) -> Result<UniversalSession, AdapterError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut session_id: Option<String> = None;
        let mut cwd: Option<String> = None;
        let mut git_branch: Option<String> = None;
        let mut git_commit: Option<String> = None;
        let mut git_remote: Option<String> = None;
        let mut model: Option<String> = None;
        let mut provider: Option<String> = None;
        let mut created: Option<DateTime<Utc>> = None;
        let mut last_modified: Option<DateTime<Utc>> = None;
        let mut timeline: Vec<TimelineEntry> = Vec::new();

        // Track tool calls to match with results
        let mut pending_tool_calls: HashMap<String, (UniversalTool, serde_json::Value)> =
            HashMap::new();
        let mut entry_index = 0u64;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: Result<CodexEntry, _> = serde_json::from_str(&line);
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Track timestamps
            if let Some(ts) = &entry.timestamp {
                if let Ok(parsed) = DateTime::parse_from_rfc3339(ts) {
                    let ts = parsed.with_timezone(&Utc);
                    if created.is_none() || ts < created.unwrap() {
                        created = Some(ts);
                    }
                    if last_modified.is_none() || ts > last_modified.unwrap() {
                        last_modified = Some(ts);
                    }
                }
            }

            match entry.entry_type.as_str() {
                "session_meta" => {
                    if let Some(payload) = entry.payload {
                        if let Some(id) = payload.id {
                            session_id = Some(id);
                        }
                        if let Some(c) = payload.cwd {
                            cwd = Some(c);
                        }
                        if let Some(m) = payload.model_provider {
                            provider = Some(m);
                        }
                        if let Some(git) = payload.git {
                            git_branch = git.branch;
                            git_commit = git.commit_hash;
                            git_remote = git.repository_url;
                        }
                    }
                }
                "turn_context" => {
                    if let Some(payload) = entry.payload {
                        if model.is_none() {
                            model = payload.model;
                        }
                    }
                }
                "response_item" => {
                    if let Some(payload) = entry.payload {
                        let ts = entry
                            .timestamp
                            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(Utc::now);

                        if let Some(item_type) = payload.item_type {
                            match item_type.as_str() {
                                "message" => {
                                    if let Some(role) = payload.role {
                                        if let Some(content) = payload.content {
                                            for item in content {
                                                match item.content_type.as_deref() {
                                                    Some("input_text") | Some("text") => {
                                                        if let Some(text) = item.text {
                                                            if role == "user" {
                                                                timeline.push(TimelineEntry::User(
                                                                    UserMessage {
                                                                        id: format!(
                                                                            "{}",
                                                                            entry_index
                                                                        ),
                                                                        timestamp: ts,
                                                                        content: text,
                                                                    },
                                                                ));
                                                            } else if role == "assistant" {
                                                                timeline.push(
                                                                    TimelineEntry::Assistant(
                                                                        AssistantMessage {
                                                                            id: format!(
                                                                                "{}",
                                                                                entry_index
                                                                            ),
                                                                            timestamp: ts,
                                                                            content: text,
                                                                            thinking: None,
                                                                        },
                                                                    ),
                                                                );
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                                "function_call" => {
                                    if let Some(name) = payload.name {
                                        let tool = UniversalTool::from_codex(&name);
                                        let id = payload
                                            .call_id
                                            .unwrap_or_else(|| format!("{}", entry_index));
                                        let arguments = payload
                                            .arguments
                                            .map(|s| {
                                                serde_json::from_str(&s)
                                                    .unwrap_or(serde_json::Value::Null)
                                            })
                                            .unwrap_or(serde_json::Value::Null);

                                        pending_tool_calls
                                            .insert(id.clone(), (tool, arguments.clone()));

                                        timeline.push(TimelineEntry::ToolCall(ToolCall {
                                            id: id.clone(),
                                            timestamp: ts,
                                            tool,
                                            input: arguments,
                                            original_tool: Some(name),
                                        }));
                                    }
                                }
                                "function_call_output" => {
                                    let call_id = payload.call_id.unwrap_or_default();
                                    let output = payload.output;

                                    timeline.push(TimelineEntry::ToolResult(ToolResult {
                                        id: format!("{}", entry_index),
                                        timestamp: ts,
                                        call_id,
                                        success: true, // Codex doesn't have explicit error flag
                                        output,
                                        error: None,
                                    }));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                "event_msg" => {
                    if let Some(payload) = entry.payload {
                        let ts = entry
                            .timestamp
                            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(Utc::now);

                        if let Some(msg_type) = &payload.item_type {
                            if msg_type == "user_message" {
                                if let Some(message) = payload.message {
                                    timeline.push(TimelineEntry::User(UserMessage {
                                        id: format!("{}", entry_index),
                                        timestamp: ts,
                                        content: message,
                                    }));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            entry_index += 1;
        }

        // Build the session
        let now = Utc::now();
        let session_id = session_id.unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        });
        let project_path = cwd.unwrap_or_else(|| "/unknown".to_string());

        let git_info = if git_branch.is_some() || git_commit.is_some() || git_remote.is_some() {
            Some(GitInfo {
                branch: git_branch.unwrap_or_default(),
                commit: git_commit,
                remote: git_remote,
            })
        } else {
            None
        };

        let mut session = UniversalSession {
            id: format!("codex-{}", session_id),
            version: USF_VERSION.to_string(),
            source: SessionSource {
                cli: CliType::Codex,
                original_id: Some(session_id),
            },
            project: ProjectInfo {
                path: project_path,
                name: None,
                git: git_info,
            },
            model: ModelInfo {
                provider: provider.unwrap_or_else(|| "openai".to_string()),
                model: model.unwrap_or_else(|| "unknown".to_string()),
                config: None,
            },
            timeline,
            metadata: SessionMetadata {
                created: created.unwrap_or(now),
                last_modified: last_modified.unwrap_or(now),
                tokens: None,
                cost: None,
            },
        };

        session.project.name = session
            .project
            .path
            .split('/')
            .next_back()
            .map(|s| s.to_string());

        Ok(session)
    }

    /// Parse just enough of a session file to build a summary
    fn parse_session_summary(&self, path: &PathBuf) -> Result<SessionSummary, AdapterError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut session_id: Option<String> = None;
        let mut cwd: Option<String> = None;
        let mut git_branch: Option<String> = None;
        let mut created: Option<DateTime<Utc>> = None;
        let mut last_modified: Option<DateTime<Utc>> = None;
        let mut first_user_message: Option<String> = None;
        let mut message_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: Result<CodexEntry, _> = serde_json::from_str(&line);
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Track timestamps
            if let Some(ts) = &entry.timestamp {
                if let Ok(parsed) = DateTime::parse_from_rfc3339(ts) {
                    let ts = parsed.with_timezone(&Utc);
                    if created.is_none() || ts < created.unwrap() {
                        created = Some(ts);
                    }
                    if last_modified.is_none() || ts > last_modified.unwrap() {
                        last_modified = Some(ts);
                    }
                }
            }

            if entry.entry_type == "session_meta" {
                if let Some(payload) = &entry.payload {
                    if session_id.is_none() {
                        session_id = payload.id.clone();
                    }
                    if cwd.is_none() {
                        cwd = payload.cwd.clone();
                    }
                    if let Some(git) = &payload.git {
                        if git_branch.is_none() {
                            git_branch = git.branch.clone();
                        }
                    }
                }
            }

            if entry.entry_type == "event_msg" {
                if let Some(payload) = &entry.payload {
                    if payload.item_type.as_deref() == Some("user_message") {
                        message_count += 1;
                        if first_user_message.is_none() {
                            first_user_message = payload.message.clone();
                        }
                    }
                }
            }

            if entry.entry_type == "response_item" {
                if let Some(payload) = &entry.payload {
                    if payload.item_type.as_deref() == Some("message") {
                        message_count += 1;
                        if first_user_message.is_none() && payload.role.as_deref() == Some("user") {
                            if let Some(content) = &payload.content {
                                for item in content {
                                    if item.content_type.as_deref() == Some("input_text") {
                                        first_user_message = item.text.clone();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Stop after reading enough lines for summary
            if line_num > 50 && session_id.is_some() && first_user_message.is_some() {
                break;
            }
        }

        let now = Utc::now();
        let session_id = session_id.unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

        let title = first_user_message
            .map(|s| truncate(&s, 60))
            .unwrap_or_else(|| format!("Session {}", &session_id[..8.min(session_id.len())]));

        Ok(SessionSummary {
            id: format!("codex-{}", session_id),
            cli: CliType::Codex,
            project_path: cwd.unwrap_or_else(|| "/unknown".to_string()),
            title,
            created: created.unwrap_or(now),
            last_modified: last_modified.unwrap_or(now),
            message_count,
            git_branch,
        })
    }
}

impl SessionAdapter for CodexAdapter {
    fn cli_type(&self) -> CliType {
        CliType::Codex
    }

    fn is_available(&self) -> bool {
        self.sessions_dir().is_dir()
    }

    fn base_dir(&self) -> Option<PathBuf> {
        Some(self.base_dir.clone())
    }

    fn list_sessions(&self) -> Result<Vec<SessionSummary>, AdapterError> {
        let mut sessions = Vec::new();
        let sessions_dir = self.sessions_dir();

        if !sessions_dir.exists() {
            return Ok(sessions);
        }

        // Codex stores sessions in year/month/day directories
        // e.g., sessions/2026/01/04/rollout-....jsonl
        Self::walk_session_files(&sessions_dir, &mut |path| {
            if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                if let Ok(summary) = self.parse_session_summary(&path) {
                    sessions.push(summary);
                }
            }
        })?;

        // Sort by last_modified descending
        sessions.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

        Ok(sessions)
    }

    fn load_session(&self, id: &str) -> Result<UniversalSession, AdapterError> {
        // ID format: codex-{session_id} or just {session_id}
        let session_id = id.strip_prefix("codex-").unwrap_or(id);
        let sessions_dir = self.sessions_dir();

        // Search recursively for the session file
        let mut found_path: Option<PathBuf> = None;
        Self::walk_session_files(&sessions_dir, &mut |path| {
            if found_path.is_some() {
                return;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                // Codex session files are named: rollout-{timestamp}-{session_id}.jsonl
                if stem.contains(session_id) {
                    found_path = Some(path.clone());
                }
            }
        })?;

        match found_path {
            Some(path) => self.parse_session_file(&path),
            None => Err(AdapterError::NotFound(id.to_string())),
        }
    }
}

impl CodexAdapter {
    fn walk_session_files<F>(dir: &PathBuf, callback: &mut F) -> Result<(), AdapterError>
    where
        F: FnMut(PathBuf),
    {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::walk_session_files(&path, callback)?;
            } else if path.is_file() {
                callback(path);
            }
        }

        Ok(())
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    let s = s.trim();
    let first_line = s.lines().next().unwrap_or(s);
    if first_line.len() <= max_len {
        first_line.to_string()
    } else {
        format!("{}...", &first_line[..max_len - 3])
    }
}

// Codex JSONL entry structure
#[derive(Debug, Deserialize)]
struct CodexEntry {
    #[serde(rename = "type")]
    entry_type: String,
    timestamp: Option<String>,
    payload: Option<CodexPayload>,
}

#[derive(Debug, Deserialize)]
struct CodexPayload {
    // session_meta fields
    id: Option<String>,
    cwd: Option<String>,
    model_provider: Option<String>,
    git: Option<CodexGit>,

    // turn_context fields
    model: Option<String>,

    // response_item fields (type field used for both item_type and msg_type)
    #[serde(rename = "type")]
    item_type: Option<String>,
    role: Option<String>,
    content: Option<Vec<CodexContentItem>>,
    name: Option<String>,
    call_id: Option<String>,
    arguments: Option<String>,
    output: Option<String>,

    // event_msg fields (uses item_type for type discrimination)
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexGit {
    branch: Option<String>,
    commit_hash: Option<String>,
    repository_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexContentItem {
    #[serde(rename = "type")]
    content_type: Option<String>,
    text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let adapter = CodexAdapter::new();
        if let Some(adapter) = adapter {
            assert_eq!(adapter.cli_type(), CliType::Codex);
        }
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a longer string", 10), "this is...");
    }

    #[test]
    fn test_session_meta_parsing() {
        let json = r#"{"timestamp":"2026-01-04T18:54:29.163Z","type":"session_meta","payload":{"id":"test-id","cwd":"/home/user/project","model_provider":"openai","git":{"branch":"main","commit_hash":"abc123"}}}"#;
        let entry: CodexEntry = serde_json::from_str(json).unwrap();

        assert_eq!(entry.entry_type, "session_meta");
        let payload = entry.payload.unwrap();
        assert_eq!(payload.id, Some("test-id".to_string()));
        assert_eq!(payload.cwd, Some("/home/user/project".to_string()));
    }
}
