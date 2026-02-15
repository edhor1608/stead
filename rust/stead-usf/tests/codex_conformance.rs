use std::fs;
use std::path::PathBuf;

use stead_usf::{CliType, CodexAdapter, SessionAdapter};

fn fixture(path: &str) -> String {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    fs::read_to_string(base.join(path)).expect("fixture must exist")
}

#[test]
fn codex_adapter_matches_locked_fixture_contract() {
    let raw = fixture("codex/valid_session.json");
    let adapter = CodexAdapter;

    let session = adapter.parse(&raw).expect("fixture should parse");

    assert_eq!(session.cli, CliType::Codex);
    assert_eq!(session.id, "codex-s-101");
    assert_eq!(session.project_path, "/tmp/project-beta");
    assert_eq!(session.updated_at, 1700002000);
    assert_eq!(session.message_count, 2);
    assert_eq!(session.title, "Refactor parser");
}
