//! SQLite storage implementation
//!
//! Default storage backend using .stead/stead.db

use crate::schema::{Contract, ContractStatus};
use crate::storage::StorageError;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

const DB_FILE: &str = "stead.db";

/// SQLite storage backend
pub struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    /// Open (or create) the SQLite database at .stead/stead.db
    pub fn open(cwd: &Path) -> Result<Self, StorageError> {
        let dir = super::ensure_stead_dir(cwd)?;
        let db_path = dir.join(DB_FILE);
        let conn = Connection::open(&db_path).map_err(|e| {
            StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    /// Create an in-memory database (for tests)
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory().map_err(|e| {
            StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> Result<(), StorageError> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS contracts (
                    id TEXT PRIMARY KEY,
                    task TEXT NOT NULL,
                    verify_cmd TEXT NOT NULL,
                    status TEXT NOT NULL,
                    output TEXT,
                    created_at TEXT NOT NULL,
                    completed_at TEXT,
                    project_path TEXT NOT NULL DEFAULT '',
                    owner TEXT,
                    blocked_by TEXT NOT NULL DEFAULT '[]',
                    blocks TEXT NOT NULL DEFAULT '[]'
                );
                CREATE INDEX IF NOT EXISTS idx_contracts_status ON contracts(status);
                CREATE INDEX IF NOT EXISTS idx_contracts_project_path ON contracts(project_path);
                CREATE INDEX IF NOT EXISTS idx_contracts_owner ON contracts(owner);",
            )
            .map_err(|e| {
                StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?;

        // Migration: add new columns if they don't exist (for existing DBs)
        for col in ["owner TEXT", "blocked_by TEXT NOT NULL DEFAULT '[]'", "blocks TEXT NOT NULL DEFAULT '[]'"] {
            let col_name = col.split_whitespace().next().unwrap();
            let _ = self.conn.execute_batch(
                &format!("ALTER TABLE contracts ADD COLUMN {}", col),
            );
            // Ignore error — column already exists
            let _ = col_name; // suppress unused warning
        }

        Ok(())
    }

    /// Get the database file path for a project directory
    pub fn db_path(cwd: &Path) -> PathBuf {
        cwd.join(super::STEAD_DIR).join(DB_FILE)
    }
}

impl super::Storage for SqliteStorage {
    fn save_contract(&self, contract: &Contract) -> Result<(), StorageError> {
        self.conn
            .execute(
                "INSERT INTO contracts (id, task, verify_cmd, status, output, created_at, completed_at, project_path, owner, blocked_by, blocks)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    contract.id,
                    contract.task,
                    contract.verification,
                    contract.status.to_string(),
                    contract.output,
                    contract.created_at.to_rfc3339(),
                    contract.completed_at.map(|dt| dt.to_rfc3339()),
                    "",
                    contract.owner,
                    serde_json::to_string(&contract.blocked_by).unwrap_or_default(),
                    serde_json::to_string(&contract.blocks).unwrap_or_default(),
                ],
            )
            .map_err(|e| {
                StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?;
        Ok(())
    }

    fn load_contract(&self, id: &str) -> Result<Option<Contract>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, task, verify_cmd, status, output, created_at, completed_at, owner, blocked_by, blocks FROM contracts WHERE id = ?1")
            .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let result = stmt
            .query_row(params![id], |row| row_to_contract(row))
            .optional()
            .map_err(|e| {
                StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?;

        Ok(result)
    }

    fn load_all_contracts(&self) -> Result<Vec<Contract>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, task, verify_cmd, status, output, created_at, completed_at, owner, blocked_by, blocks FROM contracts ORDER BY created_at DESC")
            .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let contracts = stmt
            .query_map([], |row| row_to_contract(row))
            .map_err(|e| {
                StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?;

        Ok(contracts)
    }

    fn update_contract(&self, contract: &Contract) -> Result<(), StorageError> {
        let rows = self
            .conn
            .execute(
                "UPDATE contracts SET task = ?1, verify_cmd = ?2, status = ?3, output = ?4, completed_at = ?5, owner = ?6, blocked_by = ?7, blocks = ?8 WHERE id = ?9",
                params![
                    contract.task,
                    contract.verification,
                    contract.status.to_string(),
                    contract.output,
                    contract.completed_at.map(|dt| dt.to_rfc3339()),
                    contract.owner,
                    serde_json::to_string(&contract.blocked_by).unwrap_or_default(),
                    serde_json::to_string(&contract.blocks).unwrap_or_default(),
                    contract.id,
                ],
            )
            .map_err(|e| {
                StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?;

        if rows == 0 {
            return Err(StorageError::NotFound(contract.id.clone()));
        }
        Ok(())
    }

    fn filter_by_status(&self, status: &str) -> Result<Vec<Contract>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, task, verify_cmd, status, output, created_at, completed_at, owner, blocked_by, blocks FROM contracts WHERE status = ?1 ORDER BY created_at DESC")
            .map_err(|e| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let contracts = stmt
            .query_map(params![status], |row| row_to_contract(row))
            .map_err(|e| {
                StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?;

        Ok(contracts)
    }
}

/// Parse a rusqlite Row into a Contract
fn row_to_contract(row: &rusqlite::Row) -> rusqlite::Result<Contract> {
    let id: String = row.get(0)?;
    let task: String = row.get(1)?;
    let verification: String = row.get(2)?;
    let status_str: String = row.get(3)?;
    let output: Option<String> = row.get(4)?;
    let created_at_str: String = row.get(5)?;
    let completed_at_str: Option<String> = row.get(6)?;
    let owner: Option<String> = row.get(7)?;
    let blocked_by_str: String = row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "[]".to_string());
    let blocks_str: String = row.get::<_, Option<String>>(9)?.unwrap_or_else(|| "[]".to_string());

    let status = status_str.parse::<ContractStatus>().unwrap_or(ContractStatus::Pending);

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let completed_at = completed_at_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    });

    let blocked_by: Vec<String> = serde_json::from_str(&blocked_by_str).unwrap_or_default();
    let blocks: Vec<String> = serde_json::from_str(&blocks_str).unwrap_or_default();

    Ok(Contract {
        id,
        task,
        verification,
        status,
        created_at,
        completed_at,
        output,
        owner,
        blocked_by,
        blocks,
    })
}

/// Import contracts from JSONL file into SQLite
pub fn migrate_from_jsonl(cwd: &Path) -> Result<usize, StorageError> {
    use super::Storage;

    let jsonl_path = super::get_contracts_path(cwd);
    if !jsonl_path.exists() {
        return Ok(0);
    }

    let db_path = SqliteStorage::db_path(cwd);
    if db_path.exists() {
        return Ok(0);
    }

    let contracts = super::list_contracts(cwd)?;
    if contracts.is_empty() {
        return Ok(0);
    }

    let storage = SqliteStorage::open(cwd)?;
    let count = contracts.len();
    for contract in &contracts {
        storage.save_contract(contract)?;
    }

    Ok(count)
}

/// Convenience: get the default storage for a project directory.
/// Auto-migrates from JSONL if needed.
pub fn open_default(cwd: &Path) -> Result<SqliteStorage, StorageError> {
    migrate_from_jsonl(cwd)?;
    SqliteStorage::open(cwd)
}

// Need the optional() method on rusqlite results
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Contract;
    use crate::storage::Storage;

    #[test]
    fn test_save_and_load() {
        let db = SqliteStorage::open_in_memory().unwrap();
        let contract = Contract::new("test task", "echo ok");

        db.save_contract(&contract).unwrap();

        let loaded = db.load_contract(&contract.id).unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, contract.id);
        assert_eq!(loaded.task, "test task");
        assert_eq!(loaded.verification, "echo ok");
        assert_eq!(loaded.status, ContractStatus::Pending);
    }

    #[test]
    fn test_load_not_found() {
        let db = SqliteStorage::open_in_memory().unwrap();
        let loaded = db.load_contract("nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_load_all_empty() {
        let db = SqliteStorage::open_in_memory().unwrap();
        let contracts = db.load_all_contracts().unwrap();
        assert!(contracts.is_empty());
    }

    #[test]
    fn test_load_all_sorted() {
        let db = SqliteStorage::open_in_memory().unwrap();

        let c1 = Contract::new("task 1", "verify 1");
        db.save_contract(&c1).unwrap();

        let c2 = Contract::new("task 2", "verify 2");
        db.save_contract(&c2).unwrap();

        let all = db.load_all_contracts().unwrap();
        assert_eq!(all.len(), 2);
        // Newest first
        assert!(all[0].created_at >= all[1].created_at);
    }

    #[test]
    fn test_update() {
        let db = SqliteStorage::open_in_memory().unwrap();
        let mut contract = Contract::new("task", "verify");
        db.save_contract(&contract).unwrap();

        contract.complete(true, Some("All good".to_string()));
        db.update_contract(&contract).unwrap();

        let loaded = db.load_contract(&contract.id).unwrap().unwrap();
        assert_eq!(loaded.status, ContractStatus::Completed);
        assert_eq!(loaded.output, Some("All good".to_string()));
        assert!(loaded.completed_at.is_some());
    }

    #[test]
    fn test_update_not_found() {
        let db = SqliteStorage::open_in_memory().unwrap();
        let contract = Contract::new("task", "verify");

        let result = db.update_contract(&contract);
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[test]
    fn test_filter_by_status() {
        let db = SqliteStorage::open_in_memory().unwrap();

        let c1 = Contract::new("pending task", "verify");
        db.save_contract(&c1).unwrap();

        let mut c2 = Contract::new("completed task", "verify");
        c2.complete(true, None);
        db.save_contract(&c2).unwrap();

        let pending = db.filter_by_status("pending").unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].task, "pending task");

        let completed = db.filter_by_status("completed").unwrap();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].task, "completed task");
    }

    #[test]
    fn test_migration_from_jsonl() {
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();

        // Write some contracts using JSONL
        let c1 = Contract::new("task 1", "verify 1");
        super::super::write_contract(&c1, tmp.path()).unwrap();

        let c2 = Contract::new("task 2", "verify 2");
        super::super::write_contract(&c2, tmp.path()).unwrap();

        // Migrate
        let count = migrate_from_jsonl(tmp.path()).unwrap();
        assert_eq!(count, 2);

        // Verify data is in SQLite
        let db = SqliteStorage::open(tmp.path()).unwrap();
        let all = db.load_all_contracts().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_migration_skips_if_db_exists() {
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();

        // Write a JSONL contract
        let c = Contract::new("task", "verify");
        super::super::write_contract(&c, tmp.path()).unwrap();

        // Create the DB first
        let _db = SqliteStorage::open(tmp.path()).unwrap();

        // Migration should be a no-op (DB already exists)
        let count = migrate_from_jsonl(tmp.path()).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_concurrent_reads() {
        let db = SqliteStorage::open_in_memory().unwrap();

        let c = Contract::new("task", "verify");
        db.save_contract(&c).unwrap();

        // Multiple sequential reads should work fine
        let r1 = db.load_contract(&c.id).unwrap();
        let r2 = db.load_contract(&c.id).unwrap();
        let r3 = db.load_all_contracts().unwrap();

        assert!(r1.is_some());
        assert!(r2.is_some());
        assert_eq!(r3.len(), 1);
    }

    #[test]
    fn test_open_default() {
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let db = open_default(tmp.path()).unwrap();

        let c = Contract::new("task", "verify");
        db.save_contract(&c).unwrap();

        let loaded = db.load_contract(&c.id).unwrap();
        assert!(loaded.is_some());
    }
}
