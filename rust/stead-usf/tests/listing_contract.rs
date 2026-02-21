use stead_usf::{CliType, SessionRecord, query_sessions};

#[test]
fn unified_listing_sorts_by_recency_and_filters_by_cli_and_text() {
    let sessions = vec![
        SessionRecord {
            cli: CliType::Claude,
            id: "claude-1".into(),
            project_path: "/tmp/a".into(),
            title: "Auth patch".into(),
            updated_at: 10,
            message_count: 3,
        },
        SessionRecord {
            cli: CliType::Codex,
            id: "codex-1".into(),
            project_path: "/tmp/b".into(),
            title: "Parser rewrite".into(),
            updated_at: 30,
            message_count: 4,
        },
        SessionRecord {
            cli: CliType::OpenCode,
            id: "open-1".into(),
            project_path: "/tmp/c".into(),
            title: "Health endpoint".into(),
            updated_at: 20,
            message_count: 2,
        },
    ];

    let all = query_sessions(&sessions, None, None);
    let ordered_ids: Vec<&str> = all.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ordered_ids, vec!["codex-1", "open-1", "claude-1"]);

    let only_codex = query_sessions(&sessions, Some(CliType::Codex), None);
    assert_eq!(only_codex.len(), 1);
    assert_eq!(only_codex[0].id, "codex-1");

    let auth_matches = query_sessions(&sessions, None, Some("auth"));
    assert_eq!(auth_matches.len(), 1);
    assert_eq!(auth_matches[0].id, "claude-1");
}
