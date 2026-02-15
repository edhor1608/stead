use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliType {
    Claude,
    Codex,
    OpenCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    pub cli: CliType,
    pub id: String,
    pub project_path: String,
    pub title: String,
    pub updated_at: i64,
    pub message_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsfError {
    code: &'static str,
    message: String,
}

impl UsfError {
    pub fn code(&self) -> &'static str {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    fn invalid_json(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_json",
            message: message.into(),
        }
    }

    fn invalid_format(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_format",
            message: message.into(),
        }
    }
}

pub trait SessionAdapter {
    fn parse(&self, raw: &str) -> Result<SessionRecord, UsfError>;
}

#[derive(Debug, Clone, Copy)]
pub struct ClaudeAdapter;

impl SessionAdapter for ClaudeAdapter {
    fn parse(&self, raw: &str) -> Result<SessionRecord, UsfError> {
        let parsed: ClaudeFixture =
            serde_json::from_str(raw).map_err(|err| UsfError::invalid_json(err.to_string()))?;

        if parsed.session_id.is_empty() || parsed.project_path.is_empty() {
            return Err(UsfError::invalid_format(
                "session_id and project_path are required",
            ));
        }

        let title = parsed
            .messages
            .iter()
            .find(|msg| msg.role == "user")
            .map(|msg| msg.content.clone())
            .unwrap_or_else(|| "untitled session".to_string());

        Ok(SessionRecord {
            cli: CliType::Claude,
            id: parsed.session_id,
            project_path: parsed.project_path,
            title,
            updated_at: parsed.updated_at,
            message_count: parsed.messages.len(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CodexAdapter;

impl SessionAdapter for CodexAdapter {
    fn parse(&self, raw: &str) -> Result<SessionRecord, UsfError> {
        let parsed: CodexFixture =
            serde_json::from_str(raw).map_err(|err| UsfError::invalid_json(err.to_string()))?;

        if parsed.id.is_empty() || parsed.cwd.is_empty() {
            return Err(UsfError::invalid_format("id and cwd are required"));
        }

        let title = parsed
            .events
            .iter()
            .find(|event| event.kind == "user")
            .map(|event| event.text.clone())
            .unwrap_or_else(|| "untitled session".to_string());

        Ok(SessionRecord {
            cli: CliType::Codex,
            id: parsed.id,
            project_path: parsed.cwd,
            title,
            updated_at: parsed.last_updated,
            message_count: parsed.events.len(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OpenCodeAdapter;

impl SessionAdapter for OpenCodeAdapter {
    fn parse(&self, raw: &str) -> Result<SessionRecord, UsfError> {
        let parsed: OpenCodeFixture =
            serde_json::from_str(raw).map_err(|err| UsfError::invalid_json(err.to_string()))?;

        if parsed.meta.session.is_empty() || parsed.meta.project.is_empty() {
            return Err(UsfError::invalid_format(
                "meta.session and meta.project are required",
            ));
        }

        let title = parsed
            .transcript
            .iter()
            .find(|item| item.speaker == "user")
            .map(|item| item.message.clone())
            .unwrap_or_else(|| "untitled session".to_string());

        Ok(SessionRecord {
            cli: CliType::OpenCode,
            id: parsed.meta.session,
            project_path: parsed.meta.project,
            title,
            updated_at: parsed.meta.updated,
            message_count: parsed.transcript.len(),
        })
    }
}

pub fn query_sessions(
    sessions: &[SessionRecord],
    cli_filter: Option<CliType>,
    text_filter: Option<&str>,
) -> Vec<SessionRecord> {
    let needle = text_filter.map(|value| value.to_ascii_lowercase());

    let mut filtered: Vec<SessionRecord> = sessions
        .iter()
        .filter(|session| match cli_filter {
            Some(cli) => session.cli == cli,
            None => true,
        })
        .filter(|session| {
            if let Some(needle) = &needle {
                let haystack = format!("{} {} {}", session.id, session.title, session.project_path)
                    .to_ascii_lowercase();
                haystack.contains(needle)
            } else {
                true
            }
        })
        .cloned()
        .collect();

    filtered.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.id.cmp(&right.id))
    });

    filtered
}

#[derive(Debug, Deserialize)]
struct ClaudeFixture {
    session_id: String,
    project_path: String,
    updated_at: i64,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct CodexFixture {
    id: String,
    cwd: String,
    last_updated: i64,
    events: Vec<CodexEvent>,
}

#[derive(Debug, Deserialize)]
struct CodexEvent {
    #[serde(rename = "type")]
    kind: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct OpenCodeFixture {
    meta: OpenCodeMeta,
    transcript: Vec<OpenCodeTranscriptItem>,
}

#[derive(Debug, Deserialize)]
struct OpenCodeMeta {
    session: String,
    project: String,
    updated: i64,
}

#[derive(Debug, Deserialize)]
struct OpenCodeTranscriptItem {
    speaker: String,
    message: String,
}

pub fn crate_identity() -> &'static str {
    "stead-usf"
}
