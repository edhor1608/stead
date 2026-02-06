//! OpenCode session adapter
//!
//! Parses sessions from ~/.local/share/opencode/storage/

use super::{expand_home, AdapterError, SessionAdapter};
use crate::usf::{
    AssistantMessage, CliType, ModelInfo, ProjectInfo, SessionMetadata, SessionSource,
    SessionSummary, TimelineEntry, ToolCall, ToolResult, UniversalSession, UniversalTool,
    UserMessage, USF_VERSION,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;

const OPENCODE_DIR: &str = "~/.local/share/opencode";
const STORAGE_DIR: &str = "storage";

/// OpenCode session adapter
pub struct OpenCodeAdapter {
    base_dir: PathBuf,
}

impl OpenCodeAdapter {
    /// Create a new adapter if OpenCode is installed
    pub fn new() -> Option<Self> {
        let base_dir = expand_home(OPENCODE_DIR)?;
        if base_dir.join(STORAGE_DIR).is_dir() {
            Some(Self { base_dir })
        } else {
            None
        }
    }

    fn storage_dir(&self) -> PathBuf {
        self.base_dir.join(STORAGE_DIR)
    }

    fn sessions_dir(&self) -> PathBuf {
        self.storage_dir().join("session")
    }

    fn messages_dir(&self) -> PathBuf {
        self.storage_dir().join("message")
    }

    fn parts_dir(&self) -> PathBuf {
        self.storage_dir().join("part")
    }

    fn projects_dir(&self) -> PathBuf {
        self.storage_dir().join("project")
    }

    /// Load a full session with all messages and parts
    fn load_full_session(&self, session_id: &str) -> Result<UniversalSession, AdapterError> {
        // Find session file
        let session_path = self.find_session_file(session_id)?;
        let session_meta: OpenCodeSession = self.load_json_file(&session_path)?;

        // Load project info
        let project_info = self.load_project_info(&session_meta.project_id);

        // Load messages for this session
        let messages_dir = self.messages_dir().join(session_id);
        let mut timeline: Vec<TimelineEntry> = Vec::new();
        let mut tool_call_map: HashMap<String, String> = HashMap::new();

        if messages_dir.exists() {
            let mut messages: Vec<OpenCodeMessage> = Vec::new();

            for entry in fs::read_dir(&messages_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(msg) = self.load_json_file::<OpenCodeMessage>(&path) {
                        messages.push(msg);
                    }
                }
            }

            // Sort messages by creation time
            messages.sort_by_key(|m| m.time.created);

            // Load parts for each message and build timeline
            for msg in messages {
                let parts = self.load_message_parts(&msg.id);

                for part in parts {
                    let ts = timestamp_to_datetime(part.time.start.max(msg.time.created));

                    match part.part_type.as_str() {
                        "text" => {
                            if let Some(text) = part.text {
                                if !text.is_empty() {
                                    match msg.role.as_str() {
                                        "user" => {
                                            timeline.push(TimelineEntry::User(UserMessage {
                                                id: part.id.clone(),
                                                timestamp: ts,
                                                content: text,
                                            }));
                                        }
                                        "assistant" => {
                                            timeline.push(TimelineEntry::Assistant(
                                                AssistantMessage {
                                                    id: part.id.clone(),
                                                    timestamp: ts,
                                                    content: text,
                                                    thinking: None,
                                                },
                                            ));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        "tool-invocation" => {
                            if let Some(tool_name) = part.tool_name {
                                let tool = UniversalTool::from_opencode(&tool_name);
                                let input = part
                                    .tool_invocation_input
                                    .map(|s| {
                                        serde_json::from_str(&s).unwrap_or(serde_json::Value::Null)
                                    })
                                    .unwrap_or(serde_json::Value::Null);

                                tool_call_map.insert(part.id.clone(), part.id.clone());

                                timeline.push(TimelineEntry::ToolCall(ToolCall {
                                    id: part.id.clone(),
                                    timestamp: ts,
                                    tool,
                                    input,
                                    original_tool: Some(tool_name),
                                }));
                            }
                        }
                        "tool-result" => {
                            // Tool results reference their invocation
                            let call_id = part.tool_invocation_id.unwrap_or_default();
                            let output = part.text;

                            timeline.push(TimelineEntry::ToolResult(ToolResult {
                                id: part.id,
                                timestamp: ts,
                                call_id,
                                success: true, // OpenCode doesn't have explicit error flag in parts
                                output,
                                error: None,
                            }));
                        }
                        _ => {}
                    }
                }
            }
        }

        let created = timestamp_to_datetime(session_meta.time.created);
        let last_modified = timestamp_to_datetime(session_meta.time.updated);

        let project_path = project_info
            .as_ref()
            .map(|p| p.directory.clone())
            .unwrap_or_else(|| "/unknown".to_string());

        Ok(UniversalSession {
            id: format!("opencode-{}", session_id),
            version: USF_VERSION.to_string(),
            source: SessionSource {
                cli: CliType::OpenCode,
                original_id: Some(session_id.to_string()),
            },
            project: ProjectInfo {
                path: project_path.clone(),
                name: project_path.split('/').next_back().map(|s| s.to_string()),
                git: None, // OpenCode doesn't store git info in sessions
            },
            model: ModelInfo {
                provider: "unknown".to_string(),
                model: "unknown".to_string(),
                config: None,
            },
            timeline,
            metadata: SessionMetadata {
                created,
                last_modified,
                tokens: None,
                cost: None,
            },
        })
    }

    fn find_session_file(&self, session_id: &str) -> Result<PathBuf, AdapterError> {
        // Sessions are stored in directories named by project ID
        // Session files are named: ses_{id}.json
        let sessions_dir = self.sessions_dir();

        for project_entry in fs::read_dir(&sessions_dir)? {
            let project_entry = project_entry?;
            let project_path = project_entry.path();

            if !project_path.is_dir() {
                continue;
            }

            // Look for session file
            let session_file = project_path.join(format!("{}.json", session_id));
            if session_file.exists() {
                return Ok(session_file);
            }

            // Also check files in the directory
            for file_entry in fs::read_dir(&project_path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();

                if let Some(stem) = file_path.file_stem().and_then(|s| s.to_str()) {
                    if stem == session_id || stem.contains(session_id) {
                        return Ok(file_path);
                    }
                }
            }
        }

        Err(AdapterError::NotFound(session_id.to_string()))
    }

    fn load_json_file<T: for<'de> Deserialize<'de>>(
        &self,
        path: &PathBuf,
    ) -> Result<T, AdapterError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    fn load_project_info(&self, project_id: &str) -> Option<OpenCodeProject> {
        let projects_dir = self.projects_dir();

        // Try direct file
        let project_file = projects_dir.join(format!("{}.json", project_id));
        if project_file.exists() {
            return self.load_json_file(&project_file).ok();
        }

        // Try in subdirectory
        let project_subdir = projects_dir.join(project_id);
        if project_subdir.is_dir() {
            for entry in fs::read_dir(&project_subdir).ok()? {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(project) = self.load_json_file::<OpenCodeProject>(&path) {
                        return Some(project);
                    }
                }
            }
        }

        None
    }

    fn load_message_parts(&self, message_id: &str) -> Vec<OpenCodePart> {
        let parts_dir = self.parts_dir().join(message_id);
        let mut parts = Vec::new();

        if !parts_dir.exists() {
            return parts;
        }

        if let Ok(entries) = fs::read_dir(&parts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(part) = self.load_json_file::<OpenCodePart>(&path) {
                        parts.push(part);
                    }
                }
            }
        }

        // Sort by start time
        parts.sort_by_key(|p| p.time.start);

        parts
    }

    /// Build a session summary from metadata only
    fn build_session_summary(&self, session_meta: &OpenCodeSession) -> SessionSummary {
        let project_info = self.load_project_info(&session_meta.project_id);
        let project_path = project_info
            .as_ref()
            .map(|p| p.directory.clone())
            .unwrap_or_else(|| "/unknown".to_string());

        // Get title from session or generate
        let title = session_meta
            .title
            .clone()
            .filter(|t| !t.starts_with("New session"))
            .unwrap_or_else(|| {
                // Try to get first message
                let messages_dir = self.messages_dir().join(&session_meta.id);
                if messages_dir.exists() {
                    if let Ok(entries) = fs::read_dir(&messages_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if let Ok(msg) = self.load_json_file::<OpenCodeMessage>(&path) {
                                if msg.role == "user" {
                                    let parts = self.load_message_parts(&msg.id);
                                    for part in parts {
                                        if let Some(text) = part.text {
                                            if !text.is_empty() {
                                                return truncate(&text, 60);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                format!(
                    "Session {}",
                    &session_meta.id[..8.min(session_meta.id.len())]
                )
            });

        // Count messages
        let messages_dir = self.messages_dir().join(&session_meta.id);
        let message_count = if messages_dir.exists() {
            fs::read_dir(&messages_dir)
                .map(|entries| entries.count())
                .unwrap_or(0)
        } else {
            0
        };

        SessionSummary {
            id: format!("opencode-{}", session_meta.id),
            cli: CliType::OpenCode,
            project_path,
            title,
            created: timestamp_to_datetime(session_meta.time.created),
            last_modified: timestamp_to_datetime(session_meta.time.updated),
            message_count,
            git_branch: None,
        }
    }
}

impl SessionAdapter for OpenCodeAdapter {
    fn cli_type(&self) -> CliType {
        CliType::OpenCode
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

        // Sessions are in project subdirectories
        for project_entry in fs::read_dir(&sessions_dir)? {
            let project_entry = project_entry?;
            let project_path = project_entry.path();

            if !project_path.is_dir() {
                continue;
            }

            for session_entry in fs::read_dir(&project_path)? {
                let session_entry = session_entry?;
                let session_path = session_entry.path();

                if session_path
                    .extension()
                    .map(|e| e != "json")
                    .unwrap_or(true)
                {
                    continue;
                }

                if let Ok(session_meta) = self.load_json_file::<OpenCodeSession>(&session_path) {
                    sessions.push(self.build_session_summary(&session_meta));
                }
            }
        }

        // Sort by last_modified descending
        sessions.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

        Ok(sessions)
    }

    fn load_session(&self, id: &str) -> Result<UniversalSession, AdapterError> {
        // ID format: opencode-{session_id} or just {session_id}
        let session_id = id.strip_prefix("opencode-").unwrap_or(id);
        self.load_full_session(session_id)
    }
}

fn timestamp_to_datetime(ts: i64) -> DateTime<Utc> {
    // OpenCode timestamps are in milliseconds
    DateTime::from_timestamp_millis(ts).unwrap_or_else(Utc::now)
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

// OpenCode data structures
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenCodeSession {
    id: String,
    project_id: String,
    title: Option<String>,
    time: OpenCodeTime,
}

#[derive(Debug, Deserialize)]
struct OpenCodeTime {
    created: i64,
    updated: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenCodeProject {
    directory: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenCodeMessage {
    id: String,
    role: String,
    #[allow(dead_code)]
    session_id: String,
    time: OpenCodeMessageTime,
}

#[derive(Debug, Deserialize)]
struct OpenCodeMessageTime {
    created: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenCodePart {
    id: String,
    #[serde(rename = "type")]
    part_type: String,
    text: Option<String>,
    tool_name: Option<String>,
    tool_invocation_input: Option<String>,
    tool_invocation_id: Option<String>,
    #[allow(dead_code)]
    message_id: String,
    #[allow(dead_code)]
    session_id: String,
    time: OpenCodePartTime,
}

#[derive(Debug, Deserialize)]
struct OpenCodePartTime {
    start: i64,
    #[allow(dead_code)]
    end: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let adapter = OpenCodeAdapter::new();
        if let Some(adapter) = adapter {
            assert_eq!(adapter.cli_type(), CliType::OpenCode);
        }
    }

    #[test]
    fn test_timestamp_conversion() {
        use chrono::Datelike;
        let ts = 1759497134754i64; // Example timestamp from OpenCode
        let dt = timestamp_to_datetime(ts);
        assert!(dt.year() > 2020);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a longer string", 10), "this is...");
    }

    #[test]
    fn test_session_parsing() {
        let json = r#"{"id":"ses_test","projectId":"proj_123","title":"Test Session","time":{"created":1759497134754,"updated":1759497134811}}"#;
        let session: OpenCodeSession = serde_json::from_str(json).unwrap();
        assert_eq!(session.id, "ses_test");
        assert_eq!(session.project_id, "proj_123");
    }

    #[test]
    fn test_part_parsing() {
        let json = r#"{"id":"prt_test","type":"text","text":"Hello","synthetic":false,"time":{"start":0,"end":0},"messageId":"msg_test","sessionId":"ses_test"}"#;
        let part: OpenCodePart = serde_json::from_str(json).unwrap();
        assert_eq!(part.id, "prt_test");
        assert_eq!(part.part_type, "text");
        assert_eq!(part.text, Some("Hello".to_string()));
    }
}
