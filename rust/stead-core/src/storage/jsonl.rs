//! JSONL storage implementation
//!
//! Contracts are stored as JSON Lines in .stead/contracts.jsonl
//! Each contract is one line, enabling append-only writes and streaming reads.

use crate::schema::Contract;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

const CONTRACTS_FILE: &str = "contracts.jsonl";

/// Storage-related errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error at line {line}: {message}")]
    Json { line: usize, message: String },

    #[error("Contract not found: {0}")]
    NotFound(String),
}

/// Get the path to the contracts file
pub fn get_contracts_path(cwd: &Path) -> PathBuf {
    cwd.join(super::STEAD_DIR).join(CONTRACTS_FILE)
}

/// Get the .stead directory path
pub fn get_stead_dir(cwd: &Path) -> PathBuf {
    cwd.join(super::STEAD_DIR)
}

/// Ensure the .stead directory exists
pub fn ensure_stead_dir(cwd: &Path) -> Result<PathBuf, StorageError> {
    let dir = get_stead_dir(cwd);

    if let Err(e) = fs::create_dir_all(&dir) {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            return Err(StorageError::PermissionDenied(format!(
                "Cannot create directory: {}",
                dir.display()
            )));
        }
        return Err(e.into());
    }

    Ok(dir)
}

/// Write a contract to storage (append)
pub fn write_contract(contract: &Contract, cwd: &Path) -> Result<(), StorageError> {
    ensure_stead_dir(cwd)?;

    let path = get_contracts_path(cwd);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                StorageError::PermissionDenied(format!("Cannot write to: {}", path.display()))
            } else {
                e.into()
            }
        })?;

    let json = serde_json::to_string(contract).map_err(|e| StorageError::Json {
        line: 0,
        message: e.to_string(),
    })?;

    writeln!(file, "{}", json)?;

    Ok(())
}

/// Update a contract in storage (rewrite file with updated contract)
pub fn update_contract(contract: &Contract, cwd: &Path) -> Result<(), StorageError> {
    let mut contracts = list_contracts(cwd)?;

    // Find and update the contract
    let found = contracts.iter_mut().find(|c| c.id == contract.id);

    match found {
        Some(existing) => {
            *existing = contract.clone();
        }
        None => {
            return Err(StorageError::NotFound(contract.id.clone()));
        }
    }

    // Rewrite the entire file
    rewrite_contracts(&contracts, cwd)
}

/// Rewrite all contracts to the file
fn rewrite_contracts(contracts: &[Contract], cwd: &Path) -> Result<(), StorageError> {
    ensure_stead_dir(cwd)?;

    let path = get_contracts_path(cwd);

    let mut file = File::create(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            StorageError::PermissionDenied(format!("Cannot write to: {}", path.display()))
        } else {
            e.into()
        }
    })?;

    for contract in contracts {
        let json = serde_json::to_string(contract).map_err(|e| StorageError::Json {
            line: 0,
            message: e.to_string(),
        })?;
        writeln!(file, "{}", json)?;
    }

    Ok(())
}

/// Read a contract by ID
pub fn read_contract(id: &str, cwd: &Path) -> Result<Option<Contract>, StorageError> {
    let contracts = list_contracts(cwd)?;
    Ok(contracts.into_iter().find(|c| c.id == id))
}

/// List all contracts, sorted by created_at descending
pub fn list_contracts(cwd: &Path) -> Result<Vec<Contract>, StorageError> {
    let path = get_contracts_path(cwd);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            StorageError::PermissionDenied(format!("Cannot read: {}", path.display()))
        } else {
            e.into()
        }
    })?;

    let reader = BufReader::new(file);
    let mut contracts = Vec::new();
    let mut errors = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<Contract>(&line) {
            Ok(contract) => contracts.push(contract),
            Err(e) => {
                errors.push((line_num + 1, e.to_string()));
            }
        }
    }

    // Log warnings for corrupted entries
    if !errors.is_empty() {
        eprintln!(
            "Warning: {} contract(s) could not be loaded:",
            errors.len()
        );
        for (line, error) in errors {
            eprintln!("  - Line {}: {}", line, error);
        }
    }

    // Sort by created_at descending (newest first)
    contracts.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(contracts)
}

/// Check if stead is initialized in this directory
pub fn is_initialized(cwd: &Path) -> bool {
    get_stead_dir(cwd).is_dir()
}

/// JSONL storage backend
pub struct JsonlStorage {
    cwd: PathBuf,
}

impl JsonlStorage {
    pub fn new(cwd: &Path) -> Self {
        Self {
            cwd: cwd.to_path_buf(),
        }
    }
}

impl super::Storage for JsonlStorage {
    fn save_contract(&self, contract: &Contract) -> Result<(), StorageError> {
        write_contract(contract, &self.cwd)
    }

    fn load_contract(&self, id: &str) -> Result<Option<Contract>, StorageError> {
        read_contract(id, &self.cwd)
    }

    fn load_all_contracts(&self) -> Result<Vec<Contract>, StorageError> {
        list_contracts(&self.cwd)
    }

    fn update_contract(&self, contract: &Contract) -> Result<(), StorageError> {
        update_contract(contract, &self.cwd)
    }

    fn filter_by_status(&self, status: &str) -> Result<Vec<Contract>, StorageError> {
        let contracts = list_contracts(&self.cwd)?;
        let status_lower = status.to_lowercase();
        Ok(contracts
            .into_iter()
            .filter(|c| c.status.to_string() == status_lower)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::ContractStatus;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn test_write_and_read_contract() {
        let tmp = setup();
        let contract = Contract::new("test task", "echo ok");

        write_contract(&contract, tmp.path()).unwrap();

        let loaded = read_contract(&contract.id, tmp.path()).unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, contract.id);
        assert_eq!(loaded.task, "test task");
    }

    #[test]
    fn test_list_empty() {
        let tmp = setup();
        let contracts = list_contracts(tmp.path()).unwrap();
        assert!(contracts.is_empty());
    }

    #[test]
    fn test_list_multiple_contracts() {
        let tmp = setup();

        // Create contracts with slight delay for ordering
        let c1 = Contract::new("task 1", "verify 1");
        write_contract(&c1, tmp.path()).unwrap();

        let c2 = Contract::new("task 2", "verify 2");
        write_contract(&c2, tmp.path()).unwrap();

        let contracts = list_contracts(tmp.path()).unwrap();
        assert_eq!(contracts.len(), 2);

        // Newest first
        assert_eq!(contracts[0].task, "task 2");
        assert_eq!(contracts[1].task, "task 1");
    }

    #[test]
    fn test_update_contract() {
        let tmp = setup();
        let mut contract = Contract::new("task", "verify");
        write_contract(&contract, tmp.path()).unwrap();

        // Update status
        contract.status = ContractStatus::Completed;
        contract.output = Some("Success!".to_string());
        update_contract(&contract, tmp.path()).unwrap();

        let loaded = read_contract(&contract.id, tmp.path()).unwrap().unwrap();
        assert_eq!(loaded.status, ContractStatus::Completed);
        assert_eq!(loaded.output, Some("Success!".to_string()));
    }

    #[test]
    fn test_update_nonexistent_contract() {
        let tmp = setup();
        let contract = Contract::new("task", "verify");

        let result = update_contract(&contract, tmp.path());
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[test]
    fn test_contract_not_found() {
        let tmp = setup();
        let loaded = read_contract("nonexistent", tmp.path()).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_graceful_corruption_handling() {
        let tmp = setup();

        // Write valid contract
        let contract = Contract::new("task", "verify");
        write_contract(&contract, tmp.path()).unwrap();

        // Append corrupted line directly
        let path = get_contracts_path(tmp.path());
        let mut file = OpenOptions::new().append(true).open(&path).unwrap();
        writeln!(file, "{{invalid json").unwrap();

        // Write another valid contract
        let contract2 = Contract::new("task 2", "verify 2");
        write_contract(&contract2, tmp.path()).unwrap();

        // Should load valid contracts despite corruption
        let contracts = list_contracts(tmp.path()).unwrap();
        assert_eq!(contracts.len(), 2);
    }

    #[test]
    fn test_is_initialized() {
        let tmp = setup();
        assert!(!is_initialized(tmp.path()));

        ensure_stead_dir(tmp.path()).unwrap();
        assert!(is_initialized(tmp.path()));
    }

    #[test]
    fn test_jsonl_format() {
        let tmp = setup();
        let contract = Contract::new("task", "verify");
        write_contract(&contract, tmp.path()).unwrap();

        // Read raw file content
        let path = get_contracts_path(tmp.path());
        let content = fs::read_to_string(&path).unwrap();

        // Should be single line JSON
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);

        // Should be valid JSON
        let _: Contract = serde_json::from_str(lines[0]).unwrap();
    }
}
