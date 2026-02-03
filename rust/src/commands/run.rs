//! Run command - create and execute a contract

use crate::schema::Contract;
use crate::storage;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Execute the run command
pub fn execute(task: &str, verify_cmd: &str, json_output: bool) -> Result<()> {
    execute_with_cwd(task, verify_cmd, json_output, &std::env::current_dir()?)
}

/// Execute with explicit working directory (for testing)
pub fn execute_with_cwd(
    task: &str,
    verify_cmd: &str,
    json_output: bool,
    cwd: &Path,
) -> Result<()> {
    // Create contract
    let mut contract = Contract::new(task, verify_cmd);
    storage::write_contract(&contract, cwd)?;

    if !json_output {
        println!("Contract created: {}", contract.id);
        println!("Executing task...");
    }

    // Mark as running
    contract.start();
    storage::update_contract(&contract, cwd)?;

    // Execute claude with the task
    let claude_result = spawn_claude(task);

    if let Err(e) = &claude_result {
        if !json_output {
            eprintln!("Warning: Claude execution failed: {}", e);
        }
    }

    if !json_output {
        println!("Running verification...");
    }

    // Run verification
    let (passed, output) = run_verification(verify_cmd)?;

    // Complete the contract
    contract.complete(passed, output);
    storage::update_contract(&contract, cwd)?;

    if json_output {
        println!("{}", serde_json::to_string(&contract)?);
    } else {
        println!(
            "Contract {}: {}",
            contract.id,
            if passed { "PASSED" } else { "FAILED" }
        );
        if let Some(ref out) = contract.output {
            if !out.is_empty() {
                println!("\nOutput:\n{}", out);
            }
        }
    }

    Ok(())
}

/// Spawn claude with the task
fn spawn_claude(task: &str) -> Result<()> {
    let output = Command::new("claude")
        .args(["-p", task])
        .output()
        .context("Failed to execute claude")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Claude exited with error: {}", stderr);
    }

    Ok(())
}

/// Run verification command and capture output
fn run_verification(cmd: &str) -> Result<(bool, Option<String>)> {
    let (shell, flag) = if cfg!(target_os = "windows") {
        ("cmd", "/c")
    } else {
        ("sh", "-c")
    };

    let output = Command::new(shell)
        .args([flag, cmd])
        .output()
        .context("Failed to run verification command")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let combined = [stdout.trim(), stderr.trim()]
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let output_str = if combined.is_empty() {
        None
    } else {
        Some(combined)
    };

    Ok((output.status.success(), output_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_pass() {
        let (passed, output) = run_verification("echo hello").unwrap();
        assert!(passed);
        assert_eq!(output, Some("hello".to_string()));
    }

    #[test]
    fn test_verification_fail() {
        let cmd = if cfg!(target_os = "windows") {
            "exit 1"
        } else {
            "false"
        };
        let (passed, _) = run_verification(cmd).unwrap();
        assert!(!passed);
    }

    #[test]
    fn test_verification_captures_stderr() {
        let cmd = if cfg!(target_os = "windows") {
            "echo error 1>&2"
        } else {
            "echo error >&2"
        };
        let (passed, output) = run_verification(cmd).unwrap();
        assert!(passed);
        assert!(output.unwrap().contains("error"));
    }
}
