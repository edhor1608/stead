//! Verify command - re-run verification for a contract

use crate::storage::{self, Storage};
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Execute the verify command
pub fn execute(id: &str, json_output: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let db = storage::sqlite::open_default(&cwd)?;
    execute_with_storage(id, json_output, &db)
}

/// Execute with explicit working directory (for testing)
pub fn execute_with_cwd(id: &str, json_output: bool, cwd: &Path) -> Result<()> {
    let db = storage::sqlite::open_default(cwd)?;
    execute_with_storage(id, json_output, &db)
}

/// Execute with a specific storage backend
pub fn execute_with_storage(id: &str, json_output: bool, storage: &dyn Storage) -> Result<()> {
    let contract = storage.load_contract(id)?;

    let mut contract = match contract {
        Some(c) => c,
        None => {
            if json_output {
                let error = serde_json::json!({"error": format!("Contract not found: {}", id)});
                println!("{}", error);
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
    storage.update_contract(&contract)?;

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
    use crate::storage::sqlite::SqliteStorage;

    fn test_db() -> SqliteStorage {
        SqliteStorage::open_in_memory().unwrap()
    }

    #[test]
    fn test_verify_existing_contract() {
        let db = test_db();

        let contract = Contract::new("test", "echo verified");
        db.save_contract(&contract).unwrap();

        execute_with_storage(&contract.id, false, &db).unwrap();

        let updated = db.load_contract(&contract.id).unwrap().unwrap();
        assert_eq!(updated.status, ContractStatus::Completed);
    }

    #[test]
    fn test_verify_nonexistent() {
        let db = test_db();
        let result = execute_with_storage("nonexistent", false, &db);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_failing_command() {
        let db = test_db();

        let verify_cmd = if cfg!(target_os = "windows") {
            "exit 1"
        } else {
            "false"
        };

        let contract = Contract::new("test", verify_cmd);
        db.save_contract(&contract).unwrap();

        execute_with_storage(&contract.id, false, &db).unwrap();

        let updated = db.load_contract(&contract.id).unwrap().unwrap();
        assert_eq!(updated.status, ContractStatus::Failed);
    }
}
