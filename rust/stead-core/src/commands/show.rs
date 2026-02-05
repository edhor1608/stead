//! Show command - display contract details

use crate::storage::{self, Storage};
use anyhow::{bail, Result};
use std::path::Path;

/// Execute the show command
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
                let error = serde_json::json!({"error": format!("Contract not found: {}", id)});
                println!("{}", error);
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
    use crate::storage::sqlite::SqliteStorage;

    fn test_db() -> SqliteStorage {
        SqliteStorage::open_in_memory().unwrap()
    }

    #[test]
    fn test_show_existing_contract() {
        let db = test_db();

        let contract = Contract::new("test task", "echo ok");
        db.save_contract(&contract).unwrap();

        execute_with_storage(&contract.id, false, &db).unwrap();
    }

    #[test]
    fn test_show_nonexistent_contract() {
        let db = test_db();
        let result = execute_with_storage("nonexistent", false, &db);
        assert!(result.is_err());
    }

    #[test]
    fn test_show_json_output() {
        let db = test_db();

        let contract = Contract::new("test task", "echo ok");
        db.save_contract(&contract).unwrap();

        execute_with_storage(&contract.id, true, &db).unwrap();
    }
}
