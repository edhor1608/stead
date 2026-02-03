//! Contract type definitions
//!
//! A Contract represents a unit of work with verification.
//! It captures: what to do, how to verify it, and the execution state.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Contract execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContractStatus {
    /// Created, awaiting execution
    Pending,
    /// Task is being executed
    Running,
    /// Verification passed (exit code 0)
    Passed,
    /// Verification failed (non-zero exit code)
    Failed,
}

impl std::fmt::Display for ContractStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractStatus::Pending => write!(f, "pending"),
            ContractStatus::Running => write!(f, "running"),
            ContractStatus::Passed => write!(f, "passed"),
            ContractStatus::Failed => write!(f, "failed"),
        }
    }
}

/// A contract for agent task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Unique identifier (timestamp-random in base36)
    pub id: String,

    /// Human-readable task description for the agent
    pub task: String,

    /// Shell command to verify task completion (exit 0 = pass)
    pub verification: String,

    /// Current execution status
    pub status: ContractStatus,

    /// When the contract was created
    pub created_at: DateTime<Utc>,

    /// When execution completed (None while pending/running)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,

    /// Captured output from verification command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

impl Contract {
    /// Create a new contract with pending status
    pub fn new(task: impl Into<String>, verification: impl Into<String>) -> Self {
        Self {
            id: generate_id(),
            task: task.into(),
            verification: verification.into(),
            status: ContractStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
            output: None,
        }
    }

    /// Mark contract as running
    pub fn start(&mut self) {
        self.status = ContractStatus::Running;
    }

    /// Complete the contract with a result
    pub fn complete(&mut self, passed: bool, output: Option<String>) {
        self.status = if passed {
            ContractStatus::Passed
        } else {
            ContractStatus::Failed
        };
        self.completed_at = Some(Utc::now());
        self.output = output;
    }
}

/// Generate a unique contract ID (base36 timestamp + random)
pub fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    let random: u32 = rand_simple();

    format!("{}-{}", to_base36(timestamp as u64), to_base36(random as u64))
}

/// Simple random number generator (no external dependency)
fn rand_simple() -> u32 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    let state = RandomState::new();
    let mut hasher = state.build_hasher();
    hasher.write_u64(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
    );
    hasher.finish() as u32
}

/// Convert number to base36 string
fn to_base36(mut n: u64) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

    if n == 0 {
        return "0".to_string();
    }

    let mut result = Vec::new();
    while n > 0 {
        result.push(DIGITS[(n % 36) as usize]);
        n /= 36;
    }
    result.reverse();
    String::from_utf8(result).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_creation() {
        let contract = Contract::new("fix the bug", "cargo test");

        assert!(!contract.id.is_empty());
        assert_eq!(contract.task, "fix the bug");
        assert_eq!(contract.verification, "cargo test");
        assert_eq!(contract.status, ContractStatus::Pending);
        assert!(contract.completed_at.is_none());
        assert!(contract.output.is_none());
    }

    #[test]
    fn test_contract_lifecycle() {
        let mut contract = Contract::new("task", "verify");

        // Start
        contract.start();
        assert_eq!(contract.status, ContractStatus::Running);

        // Complete with pass
        contract.complete(true, Some("All tests passed".to_string()));
        assert_eq!(contract.status, ContractStatus::Passed);
        assert!(contract.completed_at.is_some());
        assert_eq!(contract.output, Some("All tests passed".to_string()));
    }

    #[test]
    fn test_contract_failure() {
        let mut contract = Contract::new("task", "verify");
        contract.start();
        contract.complete(false, Some("Test failed".to_string()));

        assert_eq!(contract.status, ContractStatus::Failed);
    }

    #[test]
    fn test_status_serialization() {
        let status = ContractStatus::Passed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"passed\"");

        let parsed: ContractStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ContractStatus::Passed);
    }

    #[test]
    fn test_contract_serialization() {
        let contract = Contract::new("task", "verify");
        let json = serde_json::to_string(&contract).unwrap();

        // Should contain expected fields
        assert!(json.contains("\"task\":\"task\""));
        assert!(json.contains("\"verification\":\"verify\""));
        assert!(json.contains("\"status\":\"pending\""));

        // Should not contain None fields
        assert!(!json.contains("completed_at"));
        assert!(!json.contains("output"));
    }

    #[test]
    fn test_contract_deserialization() {
        let json = r#"{
            "id": "test123",
            "task": "fix bug",
            "verification": "cargo test",
            "status": "passed",
            "created_at": "2026-02-03T12:00:00Z",
            "completed_at": "2026-02-03T12:05:00Z",
            "output": "Success"
        }"#;

        let contract: Contract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.id, "test123");
        assert_eq!(contract.status, ContractStatus::Passed);
        assert!(contract.completed_at.is_some());
    }

    #[test]
    fn test_id_generation() {
        let id1 = generate_id();
        let id2 = generate_id();

        // IDs should be unique
        assert_ne!(id1, id2);

        // IDs should contain a hyphen (timestamp-random format)
        assert!(id1.contains('-'));
        assert!(id2.contains('-'));
    }

    #[test]
    fn test_base36_conversion() {
        assert_eq!(to_base36(0), "0");
        assert_eq!(to_base36(10), "a");
        assert_eq!(to_base36(35), "z");
        assert_eq!(to_base36(36), "10");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(ContractStatus::Pending.to_string(), "pending");
        assert_eq!(ContractStatus::Running.to_string(), "running");
        assert_eq!(ContractStatus::Passed.to_string(), "passed");
        assert_eq!(ContractStatus::Failed.to_string(), "failed");
    }
}
