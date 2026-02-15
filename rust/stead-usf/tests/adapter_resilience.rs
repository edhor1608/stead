use std::fs;
use std::path::PathBuf;

use stead_usf::{ClaudeAdapter, CodexAdapter, OpenCodeAdapter, SessionAdapter};

fn fixture(path: &str) -> String {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    fs::read_to_string(base.join(path)).expect("fixture must exist")
}

#[test]
fn claude_partial_input_returns_typed_error() {
    let raw = fixture("claude/partial_session.json");
    let adapter = ClaudeAdapter;

    let error = adapter
        .parse(&raw)
        .expect_err("partial fixture should fail");
    assert_eq!(error.code(), "invalid_format");
}

#[test]
fn codex_corrupt_input_returns_typed_error() {
    let raw = fixture("codex/corrupt_session.json");
    let adapter = CodexAdapter;

    let error = adapter
        .parse(&raw)
        .expect_err("corrupt fixture should fail");
    assert_eq!(error.code(), "invalid_json");
}

#[test]
fn opencode_partial_input_returns_typed_error() {
    let raw = fixture("opencode/partial_session.json");
    let adapter = OpenCodeAdapter;

    let error = adapter
        .parse(&raw)
        .expect_err("partial fixture should fail");
    assert_eq!(error.code(), "invalid_format");
}
