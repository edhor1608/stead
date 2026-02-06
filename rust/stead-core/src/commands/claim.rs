//! Claim command - claim a contract for execution

use crate::storage::{self, Storage};
use anyhow::{bail, Result};

/// Execute the claim command
pub fn execute(id: &str, owner: &str, json_output: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let db = storage::sqlite::open_default(&cwd)?;
    execute_with_storage(id, owner, json_output, &db)
}

/// Execute with a specific storage backend
pub fn execute_with_storage(
    id: &str,
    owner: &str,
    json_output: bool,
    storage: &dyn Storage,
) -> Result<()> {
    let mut contract = match storage.load_contract(id)? {
        Some(c) => c,
        None => bail!("Contract not found: {}", id),
    };

    // Pending → Ready (if needed)
    if contract.status == crate::schema::ContractStatus::Pending {
        contract
            .mark_ready()
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }

    // Ready → Claimed
    contract
        .claim(owner)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    storage.update_contract(&contract)?;

    if json_output {
        println!("{}", serde_json::to_string(&contract)?);
    } else {
        println!("Contract {} claimed by {}", contract.id, owner);
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
    fn test_claim_pending_contract() {
        let db = test_db();
        let contract = Contract::new("task", "verify");
        db.save_contract(&contract).unwrap();

        execute_with_storage(&contract.id, "agent-1", false, &db).unwrap();

        let loaded = db.load_contract(&contract.id).unwrap().unwrap();
        assert_eq!(loaded.status, ContractStatus::Claimed);
        assert_eq!(loaded.owner, Some("agent-1".to_string()));
    }

    #[test]
    fn test_claim_not_found() {
        let db = test_db();
        let result = execute_with_storage("nonexistent", "agent", false, &db);
        assert!(result.is_err());
    }
}
