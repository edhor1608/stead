//! Verify command - re-run verification for a contract

use crate::storage;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Execute the verify command
pub fn execute(id: &str, json_output: bool) -> Result<()> {
    execute_with_cwd(id, json_output, &std::env::current_dir()?)
}

/// Execute with explicit working directory (for testing)
pub fn execute_with_cwd(id: &str, json_output: bool, cwd: &Path) -> Result<()> {
    let contract = storage::read_contract(id, cwd)?;

    let mut contract = match contract {
        Some(c) => c,
        None => {
            if json_output {
                println!(r#"{{"error": "Contract not found: {}"}}"#, id);
                return Ok(());
            } else {
                bail!("Contract not found: {}", id);
            }
        }
    };

    if !json_output {
        println!("Running verification: {}", contract.verification);
    }

    // Run verification
    let (passed, output) = run_verification(&contract.verification)?;

    // Update contract
    contract.complete(passed, output);
    storage::update_contract(&contract, cwd)?;

    if json_output {
        println!("{}", serde_json::to_string(&contract)?);
    } else {
        println!(
            "Verification {}: {}",
            if passed { "PASSED" } else { "FAILED" },
            contract.id
        );
        if let Some(ref out) = contract.output {
            if !out.is_empty() {
                println!("\nOutput:\n{}", out);
            }
        }
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
    use crate::schema::{Contract, ContractStatus};
    use crate::storage::write_contract;
    use tempfile::TempDir;

    #[test]
    fn test_verify_existing_contract() {
        let tmp = TempDir::new().unwrap();

        let contract = Contract::new("test", "echo verified");
        write_contract(&contract, tmp.path()).unwrap();

        execute_with_cwd(&contract.id, false, tmp.path()).unwrap();

        // Check contract was updated
        let updated = storage::read_contract(&contract.id, tmp.path())
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, ContractStatus::Passed);
    }

    #[test]
    fn test_verify_nonexistent() {
        let tmp = TempDir::new().unwrap();

        let result = execute_with_cwd("nonexistent", false, tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_failing_command() {
        let tmp = TempDir::new().unwrap();

        let verify_cmd = if cfg!(target_os = "windows") {
            "exit 1"
        } else {
            "false"
        };

        let contract = Contract::new("test", verify_cmd);
        write_contract(&contract, tmp.path()).unwrap();

        execute_with_cwd(&contract.id, false, tmp.path()).unwrap();

        // Check contract was marked failed
        let updated = storage::read_contract(&contract.id, tmp.path())
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, ContractStatus::Failed);
    }
}
