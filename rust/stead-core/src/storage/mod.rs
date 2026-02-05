//! Contract storage backends
//!
//! Supports JSONL (legacy) and SQLite (default).

mod jsonl;
pub mod sqlite;

pub use jsonl::*;

use crate::schema::Contract;

pub(crate) const STEAD_DIR: &str = ".stead";

/// Storage backend trait for contract persistence
pub trait Storage {
    fn save_contract(&self, contract: &Contract) -> Result<(), StorageError>;
    fn load_contract(&self, id: &str) -> Result<Option<Contract>, StorageError>;
    fn load_all_contracts(&self) -> Result<Vec<Contract>, StorageError>;
    fn update_contract(&self, contract: &Contract) -> Result<(), StorageError>;
    fn filter_by_status(&self, status: &str) -> Result<Vec<Contract>, StorageError>;
}
