//! Session commands - list and show AI CLI sessions

use crate::usf::{
    adapters::{discover_all_sessions, load_session_by_id},
    CliType, SessionSummary, TimelineEntry, UniversalSession,
};
use chrono::{DateTime, Local, Utc};

/// List sessions from all installed AI CLIs
pub fn list_sessions(
    cli_filter: Option<&str>,
    project_filter: Option<&str>,
    limit: usize,
    json: bool,
) -> anyhow::Result<()> {
    let mut sessions = discover_all_sessions();

    // Apply CLI filter
    if let Some(cli) = cli_filter {
        let cli_type = match cli.to_lowercase().as_str() {
            "claude" => Some(CliType::Claude),
            "codex" => Some(CliType::Codex),
            "opencode" => Some(CliType::OpenCode),
            _ => {
                eprintln!("Unknown CLI: {}. Valid options: claude, codex, opencode", cli);
                return Ok(());
            }
        };
        if let Some(ct) = cli_type {
            sessions.retain(|s| s.cli == ct);
        }
    }

    // Apply project filter
    if let Some(project) = project_filter {
        let project_lower = project.to_lowercase();
        sessions.retain(|s| s.project_path.to_lowercase().contains(&project_lower));
    }

    // Limit results
    sessions.truncate(limit);

    if json {
        println!("{}", serde_json::to_string_pretty(&sessions)?);
    } else {
        print_session_list(&sessions);
    }

    Ok(())
}

/// Show details of a specific session
pub fn show_session(id: &str, full: bool, json: bool) -> anyhow::Result<()> {
    match load_session_by_id(id) {
        Ok(session) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&session)?);
            } else {
                print_session_detail(&session, full);
            }
        }
        Err(_) => {
            eprintln!("Session not found: {}", id);
            eprintln!("Use 'stead session list' to see available sessions.");
        }
    }
    Ok(())
}

fn print_session_list(sessions: &[SessionSummary]) {
    if sessions.is_empty() {
        println!("No sessions found.");
        println!("Make sure you have Claude Code, Codex CLI, or OpenCode installed.");
        return;
    }

    // Group by CLI
    let claude_sessions: Vec<_> = sessions.iter().filter(|s| s.cli == CliType::Claude).collect();
    let codex_sessions: Vec<_> = sessions.iter().filter(|s| s.cli == CliType::Codex).collect();
    let opencode_sessions: Vec<_> = sessions
        .iter()
        .filter(|s| s.cli == CliType::OpenCode)
        .collect();

    let total = sessions.len();
    println!("Found {} sessions\n", total);

    if !claude_sessions.is_empty() {
        println!("─── Claude Code ({}) ───", claude_sessions.len());
        for s in &claude_sessions {
            print_session_row(s);
        }
        println!();
    }

    if !codex_sessions.is_empty() {
        println!("─── Codex CLI ({}) ───", codex_sessions.len());
        for s in &codex_sessions {
            print_session_row(s);
        }
        println!();
    }

    if !opencode_sessions.is_empty() {
        println!("─── OpenCode ({}) ───", opencode_sessions.len());
        for s in &opencode_sessions {
            print_session_row(s);
        }
        println!();
    }
}

fn print_session_row(s: &SessionSummary) {
    let age = format_relative_time(s.last_modified);
    let project = s
        .project_path
        .split('/')
        .last()
        .unwrap_or(&s.project_path);
    let branch = s
        .git_branch
        .as_ref()
        .map(|b| format!(" ({})", b))
        .unwrap_or_default();

    println!(
        "  {} │ {}{} │ {} │ {}",
        &s.id[..16.min(s.id.len())],
        project,
        branch,
        age,
        truncate(&s.title, 40)
    );
}

fn print_session_detail(session: &UniversalSession, full: bool) {
    // Header
    println!("═══════════════════════════════════════════════════════════════");
    println!("Session: {}", session.id);
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    // Metadata
    println!("Source:   {:?}", session.source.cli);
    println!("Project:  {}", session.project.path);
    if let Some(name) = &session.project.name {
        println!("Name:     {}", name);
    }
    if let Some(git) = &session.project.git {
        println!("Branch:   {}", git.branch);
        if let Some(commit) = &git.commit {
            println!("Commit:   {}", &commit[..8.min(commit.len())]);
        }
    }
    println!("Model:    {}/{}", session.model.provider, session.model.model);
    println!(
        "Created:  {}",
        format_datetime(session.metadata.created)
    );
    println!(
        "Modified: {}",
        format_datetime(session.metadata.last_modified)
    );

    // Message counts
    let counts = session.message_counts();
    println!(
        "Messages: {} user, {} assistant, {} tool calls",
        counts.user, counts.assistant, counts.tool_calls
    );
    println!();

    // Timeline
    if full {
        println!("─── Timeline ───");
        println!();
        for entry in &session.timeline {
            print_timeline_entry(entry);
        }
    } else {
        // Show summary: first user message + stats
        let title = session.title();
        println!("─── Summary ───");
        println!("{}", title);
        println!();
        println!("Use --full to see complete timeline.");
    }
}

fn print_timeline_entry(entry: &TimelineEntry) {
    match entry {
        TimelineEntry::User(msg) => {
            println!(
                "[{}] USER:",
                format_time(msg.timestamp)
            );
            println!("{}", indent(&msg.content, "  "));
            println!();
        }
        TimelineEntry::Assistant(msg) => {
            println!(
                "[{}] ASSISTANT:",
                format_time(msg.timestamp)
            );
            if let Some(thinking) = &msg.thinking {
                println!("  <thinking>");
                println!("{}", indent(thinking, "    "));
                println!("  </thinking>");
            }
            println!("{}", indent(&msg.content, "  "));
            println!();
        }
        TimelineEntry::ToolCall(call) => {
            let tool_name = call
                .original_tool
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_else(|| format!("{:?}", call.tool).leak());
            println!(
                "[{}] TOOL CALL: {}",
                format_time(call.timestamp),
                tool_name
            );
            // Show input summary (truncated for readability)
            let input_str = serde_json::to_string(&call.input).unwrap_or_default();
            if input_str.len() > 100 {
                println!("  Input: {}...", &input_str[..100]);
            } else {
                println!("  Input: {}", input_str);
            }
            println!();
        }
        TimelineEntry::ToolResult(result) => {
            let status = if result.success { "✓" } else { "✗" };
            println!(
                "[{}] TOOL RESULT {} ({})",
                format_time(result.timestamp),
                status,
                &result.call_id[..8.min(result.call_id.len())]
            );
            if let Some(output) = &result.output {
                let truncated = truncate(output, 200);
                println!("{}", indent(&truncated, "  "));
            }
            if let Some(error) = &result.error {
                println!("  Error: {}", truncate(error, 100));
            }
            println!();
        }
        TimelineEntry::System(msg) => {
            println!(
                "[{}] SYSTEM: {}",
                format_time(msg.timestamp),
                truncate(&msg.content, 100)
            );
            println!();
        }
    }
}

fn format_relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_minutes() < 1 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}d ago", duration.num_days())
    } else {
        format!("{}w ago", duration.num_weeks())
    }
}

fn format_datetime(dt: DateTime<Utc>) -> String {
    let local: DateTime<Local> = dt.into();
    local.format("%Y-%m-%d %H:%M").to_string()
}

fn format_time(dt: DateTime<Utc>) -> String {
    let local: DateTime<Local> = dt.into();
    local.format("%H:%M:%S").to_string()
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

fn indent(s: &str, prefix: &str) -> String {
    s.lines()
        .map(|line| format!("{}{}", prefix, line))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a longer string", 10), "this is...");
        assert_eq!(truncate("line1\nline2", 20), "line1");
    }

    #[test]
    fn test_indent() {
        assert_eq!(indent("hello\nworld", "  "), "  hello\n  world");
    }

    #[test]
    fn test_format_relative_time() {
        let now = Utc::now();
        assert_eq!(format_relative_time(now), "just now");

        let hour_ago = now - chrono::Duration::hours(2);
        assert_eq!(format_relative_time(hour_ago), "2h ago");

        let day_ago = now - chrono::Duration::days(3);
        assert_eq!(format_relative_time(day_ago), "3d ago");
    }
}
