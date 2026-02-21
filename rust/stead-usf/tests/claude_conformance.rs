use std::fs;
use std::path::PathBuf;

use stead_usf::{ClaudeAdapter, CliType, SessionAdapter};

fn fixture(path: &str) -> String {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    fs::read_to_string(base.join(path)).expect("fixture must exist")
}

#[test]
fn claude_adapter_matches_locked_fixture_contract() {
    let raw = fixture("claude/valid_session.json");
    let adapter = ClaudeAdapter;

    let session = adapter.parse(&raw).expect("fixture should parse");

    assert_eq!(session.cli, CliType::Claude);
    assert_eq!(session.id, "claude-s-001");
    assert_eq!(session.project_path, "/tmp/project-alpha");
    assert_eq!(session.updated_at, 1700001000);
    assert_eq!(session.message_count, 3);
    assert_eq!(session.title, "Implement auth middleware");
}
