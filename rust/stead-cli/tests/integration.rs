use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[allow(deprecated)]
fn stead() -> Command {
    Command::cargo_bin("stead").unwrap()
}

#[test]
fn test_help_lists_grouped_command_families() {
    stead()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("contract"))
        .stdout(predicate::str::contains("session"))
        .stdout(predicate::str::contains("resource"))
        .stdout(predicate::str::contains("attention"))
        .stdout(predicate::str::contains("context"))
        .stdout(predicate::str::contains("module"))
        .stdout(predicate::str::contains("daemon"));
}

#[test]
fn test_default_status_json_schema_is_stable() {
    let tmp = TempDir::new().unwrap();

    let output = stead()
        .arg("--json")
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(json["status"], "ok");
    assert_eq!(json["daemon"], "ok");
    assert!(json.get("attention").is_some());
    assert!(json.get("decisions").is_some());
    assert!(json.get("anomalies").is_some());
}

#[test]
fn test_daemon_health_json_schema_is_stable() {
    let tmp = TempDir::new().unwrap();

    let output = stead()
        .args(["--json", "daemon", "health"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(json["version"], "v1");
    assert_eq!(json["data"]["status"], "ok");
}

#[test]
fn test_contract_group_create_get_and_transition() {
    let tmp = TempDir::new().unwrap();

    stead()
        .args([
            "contract",
            "create",
            "--id",
            "c-1",
            "--blocked-by",
            "dep-a",
            "--blocked-by",
            "dep-b",
        ])
        .current_dir(tmp.path())
        .assert()
        .success();

    let get_output = stead()
        .args(["--json", "contract", "get", "c-1"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(get_output.status.success());
    let contract_json: serde_json::Value = serde_json::from_slice(&get_output.stdout).unwrap();
    assert_eq!(contract_json["id"], "c-1");
    assert_eq!(contract_json["status"], "pending");

    stead()
        .args(["contract", "transition", "c-1", "--to", "ready"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let list_output = stead()
        .args(["--json", "contract", "list"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(list_output.status.success());
    let list_json: serde_json::Value = serde_json::from_slice(&list_output.stdout).unwrap();
    assert!(list_json
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"] == "c-1"));
}

#[test]
fn test_resource_group_claim_conflict_and_negotiation() {
    let tmp = TempDir::new().unwrap();

    let first = stead()
        .args([
            "--json",
            "resource",
            "claim",
            "--resource",
            "port:3000",
            "--owner",
            "agent-a",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(first.status.success());

    let second = stead()
        .args([
            "--json",
            "resource",
            "claim",
            "--resource",
            "port:3000",
            "--owner",
            "agent-b",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(second.status.success());

    let json: serde_json::Value = serde_json::from_slice(&second.stdout).unwrap();
    assert!(json.get("Negotiated").is_some());
}

#[test]
fn test_attention_group_outputs_counts_json() {
    let tmp = TempDir::new().unwrap();

    let output = stead()
        .args(["--json", "attention", "status"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert!(json.get("needs_decision").is_some());
    assert!(json.get("anomaly").is_some());
    assert!(json.get("completed").is_some());
    assert!(json.get("running").is_some());
    assert!(json.get("queued").is_some());
}

#[test]
fn test_context_group_generates_deterministic_output() {
    let tmp = TempDir::new().unwrap();

    let output = stead()
        .args([
            "--json",
            "context",
            "generate",
            "--task",
            "Fix auth",
            "--fragment",
            "a|First source|docs/a.md",
            "--fragment",
            "b|Second source|docs/b.md",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert!(json.get("prompt").is_some());
    assert!(json.get("content").is_some());
    assert!(json.get("provider").is_some());
    assert!(json.get("citations").is_some());
    assert!(json.get("confidence").is_some());
}

#[test]
fn test_module_group_enable_disable_and_list() {
    let tmp = TempDir::new().unwrap();

    stead()
        .args(["module", "disable", "session_proxy"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let output = stead()
        .args(["--json", "module", "list"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["session_proxy"], false);

    stead()
        .args(["module", "enable", "session_proxy"])
        .current_dir(tmp.path())
        .assert()
        .success();
}

#[test]
fn test_session_group_parses_claude_fixture() {
    let tmp = TempDir::new().unwrap();
    let fixture = tmp.path().join("claude.json");
    std::fs::write(
        &fixture,
        r#"{
  "session_id":"claude-s-1",
  "project_path":"/tmp/p",
  "updated_at":1700000001,
  "messages":[{"role":"user","content":"Hello"}]
}"#,
    )
    .unwrap();

    let output = stead()
        .args([
            "--json",
            "session",
            "parse",
            "--cli",
            "claude",
            "--file",
            fixture.to_str().unwrap(),
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["cli"], "Claude");
    assert_eq!(json["id"], "claude-s-1");
}

#[test]
fn test_session_group_lists_sessions_sorted_and_filtered() {
    let tmp = TempDir::new().unwrap();
    let sessions_root = tmp.path().join(".stead").join("sessions");
    std::fs::create_dir_all(sessions_root.join("claude")).unwrap();
    std::fs::create_dir_all(sessions_root.join("codex")).unwrap();
    std::fs::create_dir_all(sessions_root.join("opencode")).unwrap();

    std::fs::write(
        sessions_root.join("claude").join("a.json"),
        r#"{
  "session_id":"claude-s-1",
  "project_path":"/tmp/p-a",
  "updated_at":1700000002,
  "messages":[{"role":"user","content":"Alpha"}]
}"#,
    )
    .unwrap();

    std::fs::write(
        sessions_root.join("codex").join("b.json"),
        r#"{
  "id":"codex-s-1",
  "cwd":"/tmp/p-b",
  "last_updated":1700000001,
  "events":[{"type":"user","text":"Beta"}]
}"#,
    )
    .unwrap();

    std::fs::write(
        sessions_root.join("opencode").join("c.json"),
        r#"{
  "meta":{"session":"opencode-s-1","project":"/tmp/p-c","updated":1700000003},
  "transcript":[{"speaker":"user","message":"Gamma"}]
}"#,
    )
    .unwrap();

    let output = stead()
        .args(["--json", "session", "list"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json.as_array().unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["id"], "opencode-s-1");
    assert_eq!(rows[1]["id"], "claude-s-1");
    assert_eq!(rows[2]["id"], "codex-s-1");

    let filtered_cli = stead()
        .args(["--json", "session", "list", "--cli", "claude"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(filtered_cli.status.success());
    let filtered_cli_json: serde_json::Value = serde_json::from_slice(&filtered_cli.stdout).unwrap();
    let filtered_cli_rows = filtered_cli_json.as_array().unwrap();
    assert_eq!(filtered_cli_rows.len(), 1);
    assert_eq!(filtered_cli_rows[0]["id"], "claude-s-1");

    let filtered_query = stead()
        .args(["--json", "session", "list", "--query", "gamma"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(filtered_query.status.success());
    let filtered_query_json: serde_json::Value =
        serde_json::from_slice(&filtered_query.stdout).unwrap();
    let filtered_query_rows = filtered_query_json.as_array().unwrap();
    assert_eq!(filtered_query_rows.len(), 1);
    assert_eq!(filtered_query_rows[0]["id"], "opencode-s-1");
}

#[test]
fn test_session_group_list_ignores_corrupt_files() {
    let tmp = TempDir::new().unwrap();
    let sessions_root = tmp.path().join(".stead").join("sessions");
    std::fs::create_dir_all(sessions_root.join("claude")).unwrap();
    std::fs::write(sessions_root.join("claude").join("bad.json"), "{not-json").unwrap();

    let output = stead()
        .args(["--json", "session", "list"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[test]
fn test_session_list_under_target_load_is_below_200ms() {
    let tmp = TempDir::new().unwrap();
    let sessions_root = tmp.path().join(".stead").join("sessions");
    std::fs::create_dir_all(sessions_root.join("claude")).unwrap();
    std::fs::create_dir_all(sessions_root.join("codex")).unwrap();
    std::fs::create_dir_all(sessions_root.join("opencode")).unwrap();

    for i in 1..=50 {
        std::fs::write(
            sessions_root.join("claude").join(format!("{i}.json")),
            format!(
                r#"{{
  "session_id":"claude-{i}",
  "project_path":"/tmp/p-{i}",
  "updated_at":1700000{i},
  "messages":[{{"role":"user","content":"Alpha {i}"}}]
}}"#
            ),
        )
        .unwrap();

        std::fs::write(
            sessions_root.join("codex").join(format!("{i}.json")),
            format!(
                r#"{{
  "id":"codex-{i}",
  "cwd":"/tmp/p-{i}",
  "last_updated":1701000{i},
  "events":[{{"type":"user","text":"Beta {i}"}}]
}}"#
            ),
        )
        .unwrap();

        std::fs::write(
            sessions_root.join("opencode").join(format!("{i}.json")),
            format!(
                r#"{{
  "meta":{{"session":"opencode-{i}","project":"/tmp/p-{i}","updated":1702000{i}}},
  "transcript":[{{"speaker":"user","message":"Gamma {i}"}}]
}}"#
            ),
        )
        .unwrap();
    }

    stead()
        .args(["--json", "session", "list"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let start = Instant::now();
    let output = stead()
        .args(["--json", "session", "list"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let elapsed = start.elapsed();

    assert!(output.status.success());
    assert!(
        elapsed < Duration::from_millis(200),
        "session list took {:?}, expected < 200ms",
        elapsed
    );
}

#[test]
fn test_ding_to_context_restoration_is_below_10_seconds() {
    let tmp = TempDir::new().unwrap();

    stead()
        .args(["contract", "create", "--id", "ding-c1"])
        .current_dir(tmp.path())
        .assert()
        .success();
    stead()
        .args(["contract", "transition", "ding-c1", "--to", "claimed"])
        .current_dir(tmp.path())
        .assert()
        .success();
    stead()
        .args(["contract", "transition", "ding-c1", "--to", "executing"])
        .current_dir(tmp.path())
        .assert()
        .success();
    stead()
        .args(["contract", "transition", "ding-c1", "--to", "verifying"])
        .current_dir(tmp.path())
        .assert()
        .success();
    stead()
        .args(["contract", "transition", "ding-c1", "--to", "failed"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let start = Instant::now();

    let attention_output = stead()
        .args(["--json", "attention", "status"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(attention_output.status.success());
    let attention_json: serde_json::Value = serde_json::from_slice(&attention_output.stdout).unwrap();
    assert!(attention_json["anomaly"].as_u64().unwrap_or(0) >= 1);

    let context_output = stead()
        .args([
            "--json",
            "context",
            "generate",
            "--task",
            "Restore context for ding-c1",
            "--fragment",
            "contract|failed transition|stead.db",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(context_output.status.success());
    let context_json: serde_json::Value = serde_json::from_slice(&context_output.stdout).unwrap();
    assert!(context_json.get("content").is_some());

    assert!(
        start.elapsed() < Duration::from_secs(10),
        "ding-to-context restoration exceeded 10s: {:?}",
        start.elapsed()
    );
}
