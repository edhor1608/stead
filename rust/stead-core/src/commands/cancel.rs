//! Cancel command - cancel a contract

use crate::storage::{self, Storage};
use anyhow::{bail, Result};

/// Execute the cancel command
pub fn execute(id: &str, json_output: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let db = storage::sqlite::open_default(&cwd)?;
    execute_with_storage(id, json_output, &db)
}

/// Execute with a specific storage backend
pub fn execute_with_storage(
    id: &str,
    json_output: bool,
    storage: &dyn Storage,
) -> Result<()> {
    let mut contract = match storage.load_contract(id)? {
        Some(c) => c,
        None => bail!("Contract not found: {}", id),
    };

    contract.cancel().map_err(|e| anyhow::anyhow!("{}", e))?;
    storage.update_contract(&contract)?;

    if json_output {
        println!("{}", serde_json::to_string(&contract)?);
    } else {
        println!("Contract {} cancelled", contract.id);
    }

    Ok(())
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
    fn test_cancel_pending_contract() {
        let db = test_db();
        let contract = Contract::new("task", "verify");
        db.save_contract(&contract).unwrap();

        execute_with_storage(&contract.id, false, &db).unwrap();

        let loaded = db.load_contract(&contract.id).unwrap().unwrap();
        assert_eq!(loaded.status, ContractStatus::Cancelled);
    }

    #[test]
    fn test_cancel_completed_fails() {
        let db = test_db();
        let mut contract = Contract::new("task", "verify");
        contract.complete(true, None);
        db.save_contract(&contract).unwrap();

        let result = execute_with_storage(&contract.id, false, &db);
        assert!(result.is_err());
    }

    #[test]
    fn test_cancel_not_found() {
        let db = test_db();
        let result = execute_with_storage("nonexistent", false, &db);
        assert!(result.is_err());
    }
}
