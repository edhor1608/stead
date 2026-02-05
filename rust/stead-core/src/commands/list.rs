//! List command - display contracts with optional filtering

use crate::schema::ContractStatus;
use crate::storage::{self, Storage};
use anyhow::{bail, Result};
use std::path::Path;

/// Execute the list command
pub fn execute(status_filter: Option<&str>, json_output: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let db = storage::sqlite::open_default(&cwd)?;
    execute_with_storage(status_filter, json_output, &db)
}

/// Execute with explicit working directory (for testing)
pub fn execute_with_cwd(
    status_filter: Option<&str>,
    json_output: bool,
    cwd: &Path,
) -> Result<()> {
    let db = storage::sqlite::open_default(cwd)?;
    execute_with_storage(status_filter, json_output, &db)
}

/// Execute with a specific storage backend
pub fn execute_with_storage(
    status_filter: Option<&str>,
    json_output: bool,
    storage: &dyn Storage,
) -> Result<()> {
    let mut contracts = storage.load_all_contracts()?;

    // Filter by status if provided
    if let Some(status_str) = status_filter {
        let status = parse_status(status_str)?;
        contracts.retain(|c| c.status == status);
    }

    if json_output {
        println!("{}", serde_json::to_string(&contracts)?);
        return Ok(());
    }

    if contracts.is_empty() {
        println!("No contracts found");
        return Ok(());
    }

    // Print table header
    println!(
        "{:15} {:9} {:30} {:16}",
        "ID", "STATUS", "TASK", "CREATED"
    );
    println!("{}", "-".repeat(72));

    // Print each contract
    for contract in contracts {
        println!(
            "{:15} {:9} {:30} {:16}",
            truncate(&contract.id, 15),
            contract.status,
            truncate(&contract.task, 30),
            format_date(&contract.created_at)
        );
    }

    Ok(())
}

/// Parse status string to enum
fn parse_status(s: &str) -> Result<ContractStatus> {
    match s.to_lowercase().as_str() {
        "pending" => Ok(ContractStatus::Pending),
        "running" => Ok(ContractStatus::Running),
        "passed" => Ok(ContractStatus::Passed),
        "failed" => Ok(ContractStatus::Failed),
        _ => bail!(
            "Invalid status '{}'. Valid values: pending, running, passed, failed",
            s
        ),
    }
}

/// Truncate string with ellipsis (UTF-8 safe)
fn truncate(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

/// Format date as YYYY-MM-DD HH:mm
fn format_date(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M").to_string()
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
    fn test_parse_status() {
        assert_eq!(parse_status("pending").unwrap(), ContractStatus::Pending);
        assert_eq!(parse_status("PENDING").unwrap(), ContractStatus::Pending);
        assert_eq!(parse_status("passed").unwrap(), ContractStatus::Passed);
        assert!(parse_status("invalid").is_err());
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 2), "hi");
        // UTF-8 safety: should not panic on multi-byte characters
        assert_eq!(truncate("héllo wörld", 8), "héllo...");
        assert_eq!(truncate("日本語テスト", 5), "日本...");
    }

    #[test]
    fn test_format_date() {
        use chrono::TimeZone;
        let dt = chrono::Utc.with_ymd_and_hms(2026, 2, 3, 14, 30, 0).unwrap();
        assert_eq!(format_date(&dt), "2026-02-03 14:30");
    }

    #[test]
    fn test_list_empty() {
        let db = test_db();
        execute_with_storage(None, false, &db).unwrap();
    }

    #[test]
    fn test_list_with_contracts() {
        let db = test_db();

        let c1 = Contract::new("task 1", "verify 1");
        db.save_contract(&c1).unwrap();

        let c2 = Contract::new("task 2", "verify 2");
        db.save_contract(&c2).unwrap();

        execute_with_storage(None, false, &db).unwrap();
    }

    #[test]
    fn test_list_json() {
        let db = test_db();

        let c = Contract::new("task", "verify");
        db.save_contract(&c).unwrap();

        execute_with_storage(None, true, &db).unwrap();
    }

    #[test]
    fn test_list_filter_by_status() {
        let db = test_db();

        let c = Contract::new("task", "verify");
        db.save_contract(&c).unwrap();

        // Should work with valid status
        execute_with_storage(Some("pending"), false, &db).unwrap();
    }

    #[test]
    fn test_list_invalid_status() {
        let db = test_db();
        let result = execute_with_storage(Some("invalid"), false, &db);
        assert!(result.is_err());
    }
}
