//! Integration tests for the stead CLI

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[allow(deprecated)]
fn stead() -> Command {
    Command::cargo_bin("stead").unwrap()
}

#[test]
fn test_help() {
    stead()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Operating environment"));
}

#[test]
fn test_version() {
    stead()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.2.0"));
}

#[test]
fn test_list_empty() {
    let tmp = TempDir::new().unwrap();

    stead()
        .arg("list")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No contracts found"));
}

#[test]
fn test_run_and_list() {
    let tmp = TempDir::new().unwrap();

    // Run a contract
    stead()
        .args(["run", "test task", "--verify", "echo success"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("PASSED"));

    // List should show it
    stead()
        .arg("list")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("test task"))
        .stdout(predicate::str::contains("completed"));
}

#[test]
fn test_run_failing_verification() {
    let tmp = TempDir::new().unwrap();

    stead()
        .args(["run", "test task", "--verify", "false"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("FAILED"));

    // List should show failed status
    stead()
        .arg("list")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("failed"));
}

#[test]
fn test_show_contract() {
    let tmp = TempDir::new().unwrap();

    // Run a contract first
    let output = stead()
        .args(["run", "test task", "--verify", "echo hello", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    // Parse the JSON to get the ID
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let id = json["id"].as_str().unwrap();

    // Show the contract
    stead()
        .args(["show", id])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(&format!("Contract: {}", id)))
        .stdout(predicate::str::contains("Status: completed"));
}

#[test]
fn test_show_not_found() {
    let tmp = TempDir::new().unwrap();

    stead()
        .args(["show", "nonexistent"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Contract not found"));
}

#[test]
fn test_verify_command() {
    let tmp = TempDir::new().unwrap();

    // Run a contract first
    let output = stead()
        .args(["run", "test task", "--verify", "echo verified", "--json"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let id = json["id"].as_str().unwrap();

    // Re-verify
    stead()
        .args(["verify", id])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("PASSED"));
}

#[test]
fn test_list_filter_by_status() {
    let tmp = TempDir::new().unwrap();

    // Create a passing contract
    stead()
        .args(["run", "passing task", "--verify", "true"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // Create a failing contract
    stead()
        .args(["run", "failing task", "--verify", "false"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // Filter by completed - should only show passing
    stead()
        .args(["list", "--status", "completed"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("passing task"))
        .stdout(predicate::str::contains("failing task").not());

    // Filter by failed - should only show failing
    stead()
        .args(["list", "--status", "failed"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("failing task"))
        .stdout(predicate::str::contains("passing task").not());
}

#[test]
fn test_json_output() {
    let tmp = TempDir::new().unwrap();

    // Run with --json
    stead()
        .args(["run", "test task", "--verify", "echo ok", "--json"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""task":"test task""#))
        .stdout(predicate::str::contains(r#""status":"completed""#));

    // List with --json
    stead()
        .args(["--json", "list"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}

#[test]
fn test_list_invalid_status() {
    let tmp = TempDir::new().unwrap();

    stead()
        .args(["list", "--status", "invalid"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid status"));
}

// Session command tests

#[test]
fn test_session_list_help() {
    stead()
        .args(["session", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List sessions"))
        .stdout(predicate::str::contains("--cli"))
        .stdout(predicate::str::contains("--project"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_session_list_runs() {
    // Session list should succeed whether or not AI CLIs are installed
    // Output varies based on what's installed, so just verify it runs
    stead()
        .args(["session", "list"])
        .assert()
        .success();
}

#[test]
fn test_session_list_json() {
    // JSON output should return a valid JSON array
    stead()
        .args(["--json", "session", "list"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}

#[test]
fn test_session_show_not_found() {
    stead()
        .args(["session", "show", "nonexistent-session-id"])
        .assert()
        .success() // Command succeeds but prints error to stderr
        .stderr(predicate::str::contains("Session not found"));
}

#[test]
fn test_session_list_invalid_cli() {
    stead()
        .args(["session", "list", "--cli", "unknown"])
        .assert()
        .success() // Command succeeds but prints error to stderr
        .stderr(predicate::str::contains("Unknown CLI"))
        .stderr(predicate::str::contains("claude, codex, opencode"));
}
