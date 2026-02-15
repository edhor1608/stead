use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
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
