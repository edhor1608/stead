//! Universal Session Format Schema
//!
//! Defines the canonical representation for AI coding CLI sessions.
//! Adapters convert from Claude Code, Codex CLI, and OpenCode formats to this schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal Session Format version
pub const USF_VERSION: &str = "1.0";

/// A session from an AI coding CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSession {
    /// Unique identifier (format: cli-originalId or generated)
    pub id: String,

    /// Schema version
    pub version: String,

    /// Source CLI information
    pub source: SessionSource,

    /// Project context
    pub project: ProjectInfo,

    /// Model information
    pub model: ModelInfo,

    /// Conversation timeline
    pub timeline: Vec<TimelineEntry>,

    /// Session metadata
    pub metadata: SessionMetadata,
}

impl UniversalSession {
    /// Create a new session with minimal required fields
    pub fn new(
        cli: CliType,
        original_id: Option<String>,
        project_path: String,
    ) -> Self {
        let id = match &original_id {
            Some(oid) => format!("{}-{}", cli.as_str(), oid),
            None => format!("{}-{}", cli.as_str(), generate_id()),
        };

        Self {
            id,
            version: USF_VERSION.to_string(),
            source: SessionSource {
                cli,
                original_id,
            },
            project: ProjectInfo {
                path: project_path,
                name: None,
                git: None,
            },
            model: ModelInfo {
                provider: "unknown".to_string(),
                model: "unknown".to_string(),
                config: None,
            },
            timeline: Vec::new(),
            metadata: SessionMetadata {
                created: Utc::now(),
                last_modified: Utc::now(),
                tokens: None,
                cost: None,
            },
        }
    }

    /// Get session title (first user message or generated)
    pub fn title(&self) -> String {
        self.timeline
            .iter()
            .find_map(|e| {
                if let TimelineEntry::User(msg) = e {
                    Some(truncate_title(&msg.content, 60))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| format!("Session {}", &self.id[..8.min(self.id.len())]))
    }

    /// Count messages by role
    pub fn message_counts(&self) -> MessageCounts {
        let mut counts = MessageCounts::default();
        for entry in &self.timeline {
            match entry {
                TimelineEntry::User(_) => counts.user += 1,
                TimelineEntry::Assistant(_) => counts.assistant += 1,
                TimelineEntry::ToolCall(_) => counts.tool_calls += 1,
                TimelineEntry::ToolResult(_) => counts.tool_results += 1,
                TimelineEntry::System(_) => counts.system += 1,
            }
        }
        counts
    }
}

/// Message count summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageCounts {
    pub user: usize,
    pub assistant: usize,
    pub tool_calls: usize,
    pub tool_results: usize,
    pub system: usize,
}

/// Source CLI type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CliType {
    Claude,
    Codex,
    OpenCode,
    Universal,
}

impl CliType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CliType::Claude => "claude",
            CliType::Codex => "codex",
            CliType::OpenCode => "opencode",
            CliType::Universal => "universal",
        }
    }
}

impl std::fmt::Display for CliType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSource {
    pub cli: CliType,
    /// Original session ID from the source CLI (for round-trip)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_id: Option<String>,
}

/// Project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Absolute path to project directory
    pub path: String,
    /// Project name (derived from path or explicit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Git information if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,
}

/// Git repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Provider name (anthropic, openai, etc.)
    pub provider: String,
    /// Model identifier
    pub model: String,
    /// Additional configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, serde_json::Value>>,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub created: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<TokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
}

/// Timeline entry types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TimelineEntry {
    User(UserMessage),
    Assistant(AssistantMessage),
    ToolCall(ToolCall),
    ToolResult(ToolResult),
    System(SystemMessage),
}

/// User message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub content: String,
}

/// Assistant message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub content: String,
    /// Extended thinking content (Claude-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
}

/// Tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    /// Normalized tool name
    pub tool: UniversalTool,
    /// Tool input parameters
    pub input: serde_json::Value,
    /// Original tool name from source CLI (for round-trip)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_tool: Option<String>,
}

/// Tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    /// ID of the corresponding ToolCall
    pub call_id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// System message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub content: String,
}

/// Normalized tool names across CLIs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UniversalTool {
    /// Read file content
    Read,
    /// Write/create file
    Write,
    /// Edit existing file
    Edit,
    /// Execute shell command
    Bash,
    /// Search files (grep/ripgrep)
    Search,
    /// Find files by pattern
    Glob,
    /// List directory
    List,
    /// Ask user question
    Ask,
    /// Spawn subagent
    Task,
    /// Fetch URL
    WebFetch,
    /// Web search
    WebSearch,
    /// Notebook edit
    NotebookEdit,
    /// Unknown/unmapped tool
    Unknown,
}

impl UniversalTool {
    /// Map Claude Code tool name to universal
    pub fn from_claude(name: &str) -> Self {
        match name {
            "Read" => Self::Read,
            "Write" => Self::Write,
            "Edit" => Self::Edit,
            "Bash" => Self::Bash,
            "Grep" => Self::Search,
            "Glob" => Self::Glob,
            "LS" => Self::List,
            "AskUserQuestion" => Self::Ask,
            "Task" => Self::Task,
            "WebFetch" => Self::WebFetch,
            "WebSearch" => Self::WebSearch,
            "NotebookEdit" => Self::NotebookEdit,
            _ => Self::Unknown,
        }
    }

    /// Map Codex CLI tool name to universal
    pub fn from_codex(name: &str) -> Self {
        match name {
            "read_file" | "read" => Self::Read,
            "write_file" | "write" => Self::Write,
            "edit_file" | "edit" | "apply_diff" => Self::Edit,
            "shell" | "bash" | "run_command" => Self::Bash,
            "grep" | "search" => Self::Search,
            "glob" | "list_files" => Self::Glob,
            "ls" | "list_dir" => Self::List,
            "ask" | "ask_user" => Self::Ask,
            "call_agent" | "spawn_agent" => Self::Task,
            _ => Self::Unknown,
        }
    }

    /// Map OpenCode tool name to universal
    pub fn from_opencode(name: &str) -> Self {
        match name {
            "read" | "file_read" => Self::Read,
            "write" | "file_write" => Self::Write,
            "edit" | "file_edit" => Self::Edit,
            "bash" | "shell" | "execute" => Self::Bash,
            "grep" | "search" => Self::Search,
            "glob" | "find" => Self::Glob,
            "ls" | "list" => Self::List,
            "ask" | "confirm" => Self::Ask,
            "task" | "agent" => Self::Task,
            _ => Self::Unknown,
        }
    }
}

impl std::fmt::Display for UniversalTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Edit => "edit",
            Self::Bash => "bash",
            Self::Search => "search",
            Self::Glob => "glob",
            Self::List => "list",
            Self::Ask => "ask",
            Self::Task => "task",
            Self::WebFetch => "web_fetch",
            Self::WebSearch => "web_search",
            Self::NotebookEdit => "notebook_edit",
            Self::Unknown => "unknown",
        };
        write!(f, "{}", s)
    }
}

/// Session list item (lightweight summary for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub cli: CliType,
    pub project_path: String,
    pub title: String,
    pub created: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub message_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
}

impl From<&UniversalSession> for SessionSummary {
    fn from(session: &UniversalSession) -> Self {
        Self {
            id: session.id.clone(),
            cli: session.source.cli,
            project_path: session.project.path.clone(),
            title: session.title(),
            created: session.metadata.created,
            last_modified: session.metadata.last_modified,
            message_count: session.timeline.len(),
            git_branch: session.project.git.as_ref().map(|g| g.branch.clone()),
        }
    }
}

// Helper functions

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    format!("{:x}", timestamp)
}

fn truncate_title(s: &str, max_len: usize) -> String {
    let s = s.trim();
    // Take first line only
    let first_line = s.lines().next().unwrap_or(s);
    if first_line.len() <= max_len {
        first_line.to_string()
    } else {
        format!("{}...", &first_line[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = UniversalSession::new(
            CliType::Claude,
            Some("abc123".to_string()),
            "/home/user/project".to_string(),
        );

        assert!(session.id.starts_with("claude-"));
        assert_eq!(session.version, USF_VERSION);
        assert_eq!(session.source.cli, CliType::Claude);
        assert_eq!(session.source.original_id, Some("abc123".to_string()));
    }

    #[test]
    fn test_session_title_from_message() {
        let mut session = UniversalSession::new(
            CliType::Claude,
            None,
            "/project".to_string(),
        );

        session.timeline.push(TimelineEntry::User(UserMessage {
            id: "1".to_string(),
            timestamp: Utc::now(),
            content: "Help me fix the authentication bug".to_string(),
        }));

        assert_eq!(session.title(), "Help me fix the authentication bug");
    }

    #[test]
    fn test_session_title_truncation() {
        let mut session = UniversalSession::new(CliType::Claude, None, "/p".to_string());

        session.timeline.push(TimelineEntry::User(UserMessage {
            id: "1".to_string(),
            timestamp: Utc::now(),
            content: "This is a very long message that should be truncated because it exceeds the maximum length allowed for a title".to_string(),
        }));

        let title = session.title();
        assert!(title.len() <= 60);
        assert!(title.ends_with("..."));
    }

    #[test]
    fn test_message_counts() {
        let mut session = UniversalSession::new(CliType::Claude, None, "/p".to_string());

        session.timeline.push(TimelineEntry::User(UserMessage {
            id: "1".to_string(),
            timestamp: Utc::now(),
            content: "Hello".to_string(),
        }));
        session.timeline.push(TimelineEntry::Assistant(AssistantMessage {
            id: "2".to_string(),
            timestamp: Utc::now(),
            content: "Hi!".to_string(),
            thinking: None,
        }));
        session.timeline.push(TimelineEntry::ToolCall(ToolCall {
            id: "3".to_string(),
            timestamp: Utc::now(),
            tool: UniversalTool::Read,
            input: serde_json::json!({"path": "/file"}),
            original_tool: Some("Read".to_string()),
        }));

        let counts = session.message_counts();
        assert_eq!(counts.user, 1);
        assert_eq!(counts.assistant, 1);
        assert_eq!(counts.tool_calls, 1);
    }

    #[test]
    fn test_tool_mapping_claude() {
        assert_eq!(UniversalTool::from_claude("Read"), UniversalTool::Read);
        assert_eq!(UniversalTool::from_claude("Write"), UniversalTool::Write);
        assert_eq!(UniversalTool::from_claude("Bash"), UniversalTool::Bash);
        assert_eq!(UniversalTool::from_claude("Grep"), UniversalTool::Search);
        assert_eq!(UniversalTool::from_claude("Unknown"), UniversalTool::Unknown);
    }

    #[test]
    fn test_tool_mapping_codex() {
        assert_eq!(UniversalTool::from_codex("read_file"), UniversalTool::Read);
        assert_eq!(UniversalTool::from_codex("shell"), UniversalTool::Bash);
        assert_eq!(UniversalTool::from_codex("apply_diff"), UniversalTool::Edit);
    }

    #[test]
    fn test_tool_mapping_opencode() {
        assert_eq!(UniversalTool::from_opencode("read"), UniversalTool::Read);
        assert_eq!(UniversalTool::from_opencode("bash"), UniversalTool::Bash);
        assert_eq!(UniversalTool::from_opencode("file_edit"), UniversalTool::Edit);
    }

    #[test]
    fn test_cli_type_display() {
        assert_eq!(CliType::Claude.to_string(), "claude");
        assert_eq!(CliType::Codex.to_string(), "codex");
        assert_eq!(CliType::OpenCode.to_string(), "opencode");
    }

    #[test]
    fn test_session_serialization() {
        let session = UniversalSession::new(
            CliType::Claude,
            Some("test".to_string()),
            "/project".to_string(),
        );

        let json = serde_json::to_string(&session).unwrap();
        let parsed: UniversalSession = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, session.id);
        assert_eq!(parsed.source.cli, CliType::Claude);
    }

    #[test]
    fn test_session_summary() {
        let mut session = UniversalSession::new(
            CliType::Claude,
            Some("abc".to_string()),
            "/home/user/myproject".to_string(),
        );
        session.project.git = Some(GitInfo {
            branch: "main".to_string(),
            commit: None,
            remote: None,
        });
        session.timeline.push(TimelineEntry::User(UserMessage {
            id: "1".to_string(),
            timestamp: Utc::now(),
            content: "Test message".to_string(),
        }));

        let summary = SessionSummary::from(&session);

        assert_eq!(summary.cli, CliType::Claude);
        assert_eq!(summary.project_path, "/home/user/myproject");
        assert_eq!(summary.message_count, 1);
        assert_eq!(summary.git_branch, Some("main".to_string()));
    }
}
