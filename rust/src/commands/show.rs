//! Show command - display contract details

use crate::storage;
use anyhow::{bail, Result};
use std::path::Path;

/// Execute the show command
pub fn execute(id: &str, json_output: bool) -> Result<()> {
    execute_with_cwd(id, json_output, &std::env::current_dir()?)
}

/// Execute with explicit working directory (for testing)
pub fn execute_with_cwd(id: &str, json_output: bool, cwd: &Path) -> Result<()> {
    let contract = storage::read_contract(id, cwd)?;

    match contract {
        Some(c) => {
            if json_output {
                println!("{}", serde_json::to_string(&c)?);
            } else {
                println!("Contract: {}", c.id);
                println!("Status: {}", c.status);
                println!("Task: {}", c.task);
                println!("Verification: {}", c.verification);
                println!("Created: {}", c.created_at.format("%Y-%m-%d %H:%M:%S"));

                if let Some(completed) = c.completed_at {
                    println!("Completed: {}", completed.format("%Y-%m-%d %H:%M:%S"));
                }

                if let Some(ref output) = c.output {
                    println!("\nOutput:");
                    println!("{}", output);
                }
            }
        }
        None => {
            if json_output {
                println!(r#"{{"error": "Contract not found: {}"}}"#, id);
            } else {
                bail!("Contract not found: {}", id);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Contract;
    use crate::storage::write_contract;
    use tempfile::TempDir;

    #[test]
    fn test_show_existing_contract() {
        let tmp = TempDir::new().unwrap();

        let contract = Contract::new("test task", "echo ok");
        write_contract(&contract, tmp.path()).unwrap();

        // Should not error
        execute_with_cwd(&contract.id, false, tmp.path()).unwrap();
    }

    #[test]
    fn test_show_nonexistent_contract() {
        let tmp = TempDir::new().unwrap();

        let result = execute_with_cwd("nonexistent", false, tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_show_json_output() {
        let tmp = TempDir::new().unwrap();

        let contract = Contract::new("test task", "echo ok");
        write_contract(&contract, tmp.path()).unwrap();

        // JSON mode should not error
        execute_with_cwd(&contract.id, true, tmp.path()).unwrap();
    }
}
