use std::fs;
use std::path::PathBuf;

use stead_usf::{CliType, OpenCodeAdapter, SessionAdapter};

fn fixture(path: &str) -> String {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    fs::read_to_string(base.join(path)).expect("fixture must exist")
}

#[test]
fn opencode_adapter_matches_locked_fixture_contract() {
    let raw = fixture("opencode/valid_session.json");
    let adapter = OpenCodeAdapter;

    let session = adapter.parse(&raw).expect("fixture should parse");

    assert_eq!(session.cli, CliType::OpenCode);
    assert_eq!(session.id, "opencode-s-77");
    assert_eq!(session.project_path, "/tmp/project-gamma");
    assert_eq!(session.updated_at, 1700003000);
    assert_eq!(session.message_count, 2);
    assert_eq!(session.title, "Add health endpoint");
}
