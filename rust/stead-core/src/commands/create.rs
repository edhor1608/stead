//! Create command - create a contract without executing it

use crate::schema::Contract;
use crate::storage::{self, Storage};
use anyhow::Result;

/// Execute the create command
pub fn execute(task: &str, verify_cmd: &str, json_output: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let db = storage::sqlite::open_default(&cwd)?;
    execute_with_storage(task, verify_cmd, json_output, &db)
}

/// Execute with a specific storage backend
pub fn execute_with_storage(
    task: &str,
    verify_cmd: &str,
    json_output: bool,
    storage: &dyn Storage,
) -> Result<()> {
    let contract = Contract::new(task, verify_cmd);
    storage.save_contract(&contract)?;

    if json_output {
        println!("{}", serde_json::to_string(&contract)?);
    } else {
        println!("Contract created: {}", contract.id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::ContractStatus;
    use crate::storage::sqlite::SqliteStorage;

    fn test_db() -> SqliteStorage {
        SqliteStorage::open_in_memory().unwrap()
    }

    #[test]
    fn test_create_contract() {
        let db = test_db();
        execute_with_storage("my task", "echo ok", false, &db).unwrap();

        let contracts = db.load_all_contracts().unwrap();
        assert_eq!(contracts.len(), 1);
        assert_eq!(contracts[0].task, "my task");
        assert_eq!(contracts[0].verification, "echo ok");
        assert_eq!(contracts[0].status, ContractStatus::Pending);
    }
}
