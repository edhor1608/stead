//! Claude Code session adapter
//!
//! Parses sessions from ~/.claude/projects/

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

const CLAUDE_DIR: &str = "~/.claude";
const PROJECTS_DIR: &str = "projects";

/// Claude Code session adapter
pub struct ClaudeAdapter {
    base_dir: PathBuf,
}

impl ClaudeAdapter {
    /// Create a new adapter if Claude Code is installed
    pub fn new() -> Option<Self> {
        let base_dir = expand_home(CLAUDE_DIR)?;
        if base_dir.join(PROJECTS_DIR).is_dir() {
            Some(Self { base_dir })
        } else {
            None
        }
    }

    fn projects_dir(&self) -> PathBuf {
        self.base_dir.join(PROJECTS_DIR)
    }

    /// Parse a session JSONL file
    fn parse_session_file(&self, path: &PathBuf) -> Result<UniversalSession, AdapterError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut session_id: Option<String> = None;
        let mut cwd: Option<String> = None;
        let mut git_branch: Option<String> = None;
        let mut model: Option<String> = None;
        let mut created: Option<DateTime<Utc>> = None;
        let mut last_modified: Option<DateTime<Utc>> = None;
        let mut timeline: Vec<TimelineEntry> = Vec::new();

        // Track tool calls to match with results
        let mut pending_tool_calls: HashMap<String, (String, UniversalTool, serde_json::Value)> =
            HashMap::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse as a Claude entry
            let entry: Result<ClaudeEntry, _> = serde_json::from_str(&line);
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // Skip unparseable lines gracefully
            };

            // Extract session metadata from first entry
            if session_id.is_none() {
                session_id = entry.session_id.clone();
            }
            if cwd.is_none() {
                cwd = entry.cwd.clone();
            }
            if git_branch.is_none() {
                git_branch = entry.git_branch.clone();
            }

            // Track timestamps
            if let Some(ts) = entry.timestamp {
                if created.is_none() || ts < created.unwrap() {
                    created = Some(ts);
                }
                if last_modified.is_none() || ts > last_modified.unwrap() {
                    last_modified = Some(ts);
                }
            }

            // Process message content
            if let Some(msg) = &entry.message {
                // Extract model from assistant messages
                if model.is_none() {
                    if let Some(m) = &msg.model {
                        model = Some(m.clone());
                    }
                }

                match msg.role.as_str() {
                    "user" => {
                        // User messages may contain text or tool results
                        if let Some(content) = &msg.content {
                            for item in content {
                                match item {
                                    ContentItem::Text { text } => {
                                        timeline.push(TimelineEntry::User(UserMessage {
                                            id: entry.uuid.clone().unwrap_or_default(),
                                            timestamp: entry.timestamp.unwrap_or_else(Utc::now),
                                            content: text.clone(),
                                        }));
                                    }
                                    ContentItem::ToolResult {
                                        tool_use_id,
                                        content: result_content,
                                        is_error,
                                    } => {
                                        // Match with pending tool call
                                        let (original_id, _tool, _input) = pending_tool_calls
                                            .remove(tool_use_id)
                                            .unwrap_or_else(|| {
                                                (
                                                    tool_use_id.clone(),
                                                    UniversalTool::Unknown,
                                                    serde_json::Value::Null,
                                                )
                                            });

                                        timeline.push(TimelineEntry::ToolResult(ToolResult {
                                            id: entry.uuid.clone().unwrap_or_default(),
                                            timestamp: entry.timestamp.unwrap_or_else(Utc::now),
                                            call_id: original_id,
                                            success: !is_error.unwrap_or(false),
                                            output: Some(result_content.clone()),
                                            error: if is_error.unwrap_or(false) {
                                                Some(result_content.clone())
                                            } else {
                                                None
                                            },
                                        }));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    "assistant" => {
                        if let Some(content) = &msg.content {
                            for item in content {
                                match item {
                                    ContentItem::Text { text } => {
                                        timeline.push(TimelineEntry::Assistant(AssistantMessage {
                                            id: entry.uuid.clone().unwrap_or_default(),
                                            timestamp: entry.timestamp.unwrap_or_else(Utc::now),
                                            content: text.clone(),
                                            thinking: None,
                                        }));
                                    }
                                    ContentItem::ToolUse { id, name, input } => {
                                        let tool = UniversalTool::from_claude(name);
                                        let tool_call_id = id.clone();

                                        // Store for matching with result
                                        pending_tool_calls
                                            .insert(id.clone(), (id.clone(), tool, input.clone()));

                                        timeline.push(TimelineEntry::ToolCall(ToolCall {
                                            id: tool_call_id,
                                            timestamp: entry.timestamp.unwrap_or_else(Utc::now),
                                            tool,
                                            input: input.clone(),
                                            original_tool: Some(name.clone()),
                                        }));
                                    }
                                    ContentItem::Thinking { thinking } => {
                                        // Add thinking to the last assistant message if exists
                                        if let Some(TimelineEntry::Assistant(msg)) =
                                            timeline.last_mut()
                                        {
                                            msg.thinking = Some(thinking.clone());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
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

        let mut session = UniversalSession {
            id: format!("claude-{}", session_id),
            version: USF_VERSION.to_string(),
            source: SessionSource {
                cli: CliType::Claude,
                original_id: Some(session_id),
            },
            project: ProjectInfo {
                path: project_path,
                name: None,
                git: git_branch.map(|branch| GitInfo {
                    branch,
                    commit: None,
                    remote: None,
                }),
            },
            model: ModelInfo {
                provider: "anthropic".to_string(),
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

        // Set project name from path
        session.project.name = session
            .project
            .path
            .split('/')
            .next_back()
            .map(|s| s.to_string());

        Ok(session)
    }
}

impl SessionAdapter for ClaudeAdapter {
    fn cli_type(&self) -> CliType {
        CliType::Claude
    }

    fn is_available(&self) -> bool {
        self.projects_dir().is_dir()
    }

    fn base_dir(&self) -> Option<PathBuf> {
        Some(self.base_dir.clone())
    }

    fn list_sessions(&self) -> Result<Vec<SessionSummary>, AdapterError> {
        let mut sessions = Vec::new();
        let projects_dir = self.projects_dir();

        if !projects_dir.exists() {
            return Ok(sessions);
        }

        // Iterate over project directories
        for project_entry in fs::read_dir(&projects_dir)? {
            let project_entry = project_entry?;
            let project_path = project_entry.path();

            if !project_path.is_dir() {
                continue;
            }

            // Find session JSONL files in each project directory
            for session_entry in fs::read_dir(&project_path)? {
                let session_entry = session_entry?;
                let session_path = session_entry.path();

                // Only process .jsonl files
                if session_path
                    .extension()
                    .map(|e| e != "jsonl")
                    .unwrap_or(true)
                {
                    continue;
                }

                // Parse just enough to build summary (first few lines)
                match self.parse_session_summary(&session_path) {
                    Ok(summary) => sessions.push(summary),
                    Err(_) => continue, // Skip unparseable sessions
                }
            }
        }

        // Sort by last_modified descending
        sessions.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

        Ok(sessions)
    }

    fn load_session(&self, id: &str) -> Result<UniversalSession, AdapterError> {
        // ID format: claude-{session_id} or just {session_id}
        let session_id = id.strip_prefix("claude-").unwrap_or(id);

        // Search for the session file in all project directories
        let projects_dir = self.projects_dir();

        for project_entry in fs::read_dir(&projects_dir)? {
            let project_entry = project_entry?;
            let project_path = project_entry.path();

            if !project_path.is_dir() {
                continue;
            }

            // Look for {session_id}.jsonl
            let session_file = project_path.join(format!("{}.jsonl", session_id));
            if session_file.exists() {
                return self.parse_session_file(&session_file);
            }
        }

        Err(AdapterError::NotFound(id.to_string()))
    }
}

impl ClaudeAdapter {
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

            // Parse entry
            let entry: Result<ClaudeEntry, _> = serde_json::from_str(&line);
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Extract metadata
            if session_id.is_none() {
                session_id = entry.session_id.clone();
            }
            if cwd.is_none() {
                cwd = entry.cwd.clone();
            }
            if git_branch.is_none() {
                git_branch = entry.git_branch.clone();
            }

            // Track timestamps
            if let Some(ts) = entry.timestamp {
                if created.is_none() || ts < created.unwrap() {
                    created = Some(ts);
                }
                if last_modified.is_none() || ts > last_modified.unwrap() {
                    last_modified = Some(ts);
                }
            }

            // Count messages and get first user message for title
            if let Some(msg) = &entry.message {
                if msg.role == "user" || msg.role == "assistant" {
                    message_count += 1;
                }
                if first_user_message.is_none() && msg.role == "user" {
                    if let Some(content) = &msg.content {
                        for item in content {
                            if let ContentItem::Text { text } = item {
                                first_user_message = Some(text.clone());
                                break;
                            }
                        }
                    }
                }
            }

            // Stop after reading enough lines for summary (optimization)
            if line_num > 100 && session_id.is_some() && first_user_message.is_some() {
                // Continue counting messages but faster
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
            id: format!("claude-{}", session_id),
            cli: CliType::Claude,
            project_path: cwd.unwrap_or_else(|| "/unknown".to_string()),
            title,
            created: created.unwrap_or(now),
            last_modified: last_modified.unwrap_or(now),
            message_count,
            git_branch,
        })
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

// Claude Code JSONL entry structure
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeEntry {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    entry_type: Option<String>,
    uuid: Option<String>,
    session_id: Option<String>,
    timestamp: Option<DateTime<Utc>>,
    cwd: Option<String>,
    git_branch: Option<String>,
    message: Option<ClaudeMessage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeMessage {
    role: String,
    model: Option<String>,
    content: Option<Vec<ContentItem>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentItem {
    Text {
        text: String,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: Option<bool>,
    },
    Thinking {
        thinking: String,
    },
    #[serde(other)]
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        // This test checks if the adapter can be created (may fail if Claude not installed)
        let adapter = ClaudeAdapter::new();
        // Just verify no panic - actual availability depends on system
        if let Some(adapter) = adapter {
            assert_eq!(adapter.cli_type(), CliType::Claude);
        }
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a longer string", 10), "this is...");
        assert_eq!(truncate("line1\nline2", 20), "line1");
    }

    #[test]
    fn test_content_item_parsing() {
        let json = r#"{"type": "text", "text": "Hello"}"#;
        let item: ContentItem = serde_json::from_str(json).unwrap();
        match item {
            ContentItem::Text { text } => assert_eq!(text, "Hello"),
            _ => panic!("Expected text"),
        }
    }

    #[test]
    fn test_tool_use_parsing() {
        let json =
            r#"{"type": "tool_use", "id": "123", "name": "Read", "input": {"path": "/file"}}"#;
        let item: ContentItem = serde_json::from_str(json).unwrap();
        match item {
            ContentItem::ToolUse { id, name, input } => {
                assert_eq!(id, "123");
                assert_eq!(name, "Read");
                assert_eq!(input["path"], "/file");
            }
            _ => panic!("Expected tool_use"),
        }
    }

    #[test]
    fn test_unknown_content_type() {
        let json = r#"{"type": "unknown_type", "foo": "bar"}"#;
        let item: ContentItem = serde_json::from_str(json).unwrap();
        assert!(matches!(item, ContentItem::Other));
    }
}
