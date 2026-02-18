//! Contract type definitions
//!
//! A Contract represents a unit of work with verification.
//! It captures: what to do, how to verify it, and the execution state.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Contract execution status (10-state lifecycle)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContractStatus {
    /// Waiting for dependencies to resolve
    Pending,
    /// Dependencies met, can be claimed
    Ready,
    /// An agent has claimed ownership
    Claimed,
    /// Work is in progress
    Executing,
    /// Running verification command
    Verifying,
    /// Verification passed
    Completed,
    /// Verification failed
    Failed,
    /// Rollback in progress
    RollingBack,
    /// Rollback finished
    RolledBack,
    /// Manually cancelled
    Cancelled,
}

impl ContractStatus {
    /// Valid next states from the current state
    pub fn valid_transitions(&self) -> &[ContractStatus] {
        use ContractStatus::*;
        match self {
            Pending => &[Ready, Cancelled],
            Ready => &[Claimed, Cancelled],
            Claimed => &[Executing, Ready, Cancelled], // unclaim goes back to Ready
            Executing => &[Verifying, Failed, Cancelled],
            Verifying => &[Completed, Failed],
            Completed => &[],                           // terminal
            Failed => &[Ready, RollingBack, Cancelled], // retry or rollback
            RollingBack => &[RolledBack, Failed],
            RolledBack => &[], // terminal
            Cancelled => &[],  // terminal
        }
    }

    /// Whether this status can transition to the target
    pub fn can_transition_to(&self, target: ContractStatus) -> bool {
        self.valid_transitions().contains(&target)
    }

    /// Whether this is a terminal state (no further transitions)
    pub fn is_terminal(&self) -> bool {
        self.valid_transitions().is_empty()
    }
}

impl std::fmt::Display for ContractStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractStatus::Pending => write!(f, "pending"),
            ContractStatus::Ready => write!(f, "ready"),
            ContractStatus::Claimed => write!(f, "claimed"),
            ContractStatus::Executing => write!(f, "executing"),
            ContractStatus::Verifying => write!(f, "verifying"),
            ContractStatus::Completed => write!(f, "completed"),
            ContractStatus::Failed => write!(f, "failed"),
            ContractStatus::RollingBack => write!(f, "rollingback"),
            ContractStatus::RolledBack => write!(f, "rolledback"),
            ContractStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for ContractStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "ready" => Ok(Self::Ready),
            "claimed" => Ok(Self::Claimed),
            "executing" | "running" => Ok(Self::Executing),
            "verifying" => Ok(Self::Verifying),
            "completed" | "passed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "rollingback" => Ok(Self::RollingBack),
            "rolledback" => Ok(Self::RolledBack),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("unknown status: {}", s)),
        }
    }
}

/// Error when attempting an invalid state transition
#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid transition from {from} to {to}")]
pub struct TransitionError {
    pub from: ContractStatus,
    pub to: ContractStatus,
}

/// A contract for agent task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Unique identifier (timestamp-random in base36)
    pub id: String,

    /// Project path this contract belongs to (absolute path preferred).
    ///
    /// For legacy JSONL entries this may be empty.
    #[serde(default)]
    pub project_path: String,

    /// Human-readable task description for the agent
    pub task: String,

    /// Shell command to verify task completion (exit 0 = pass)
    pub verification: String,

    /// Current execution status
    pub status: ContractStatus,

    /// When the contract was created
    pub created_at: DateTime<Utc>,

    /// When execution completed (None while in non-terminal state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,

    /// Captured output from verification command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    /// Agent/user that owns this contract
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    /// Contract IDs that must complete before this one can start
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_by: Vec<String>,

    /// Contract IDs that are waiting on this one
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocks: Vec<String>,
}

impl Contract {
    /// Create a new contract with pending status
    pub fn new(task: impl Into<String>, verification: impl Into<String>) -> Self {
        Self {
            id: generate_id(),
            project_path: String::new(),
            task: task.into(),
            verification: verification.into(),
            status: ContractStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
            output: None,
            owner: None,
            blocked_by: Vec::new(),
            blocks: Vec::new(),
        }
    }

    /// Transition to a new status, enforcing valid transitions
    pub fn transition_to(&mut self, target: ContractStatus) -> Result<(), TransitionError> {
        if !self.status.can_transition_to(target) {
            return Err(TransitionError {
                from: self.status,
                to: target,
            });
        }
        self.status = target;
        if target.is_terminal() {
            self.completed_at = Some(Utc::now());
        }
        Ok(())
    }

    /// Mark as ready (dependencies resolved)
    pub fn mark_ready(&mut self) -> Result<(), TransitionError> {
        self.transition_to(ContractStatus::Ready)
    }

    /// Claim for an owner
    pub fn claim(&mut self, owner: impl Into<String>) -> Result<(), TransitionError> {
        self.transition_to(ContractStatus::Claimed)?;
        self.owner = Some(owner.into());
        Ok(())
    }

    /// Release claim (back to ready)
    pub fn unclaim(&mut self) -> Result<(), TransitionError> {
        self.transition_to(ContractStatus::Ready)?;
        self.owner = None;
        Ok(())
    }

    /// Start execution
    pub fn start(&mut self) -> Result<(), TransitionError> {
        self.transition_to(ContractStatus::Executing)
    }

    /// Begin verification
    pub fn begin_verify(&mut self) -> Result<(), TransitionError> {
        self.transition_to(ContractStatus::Verifying)
    }

    /// Complete the contract with verification result
    pub fn complete(&mut self, passed: bool, output: Option<String>) {
        self.status = if passed {
            ContractStatus::Completed
        } else {
            ContractStatus::Failed
        };
        self.completed_at = Some(Utc::now());
        self.output = output;
    }

    /// Cancel the contract
    pub fn cancel(&mut self) -> Result<(), TransitionError> {
        self.transition_to(ContractStatus::Cancelled)
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

    format!(
        "{}-{}",
        to_base36(timestamp as u64),
        to_base36(random as u64)
    )
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
        assert!(contract.owner.is_none());
        assert!(contract.blocked_by.is_empty());
        assert!(contract.blocks.is_empty());
    }

    #[test]
    fn test_full_lifecycle_happy_path() {
        let mut contract = Contract::new("task", "verify");

        // Pending -> Ready
        contract.mark_ready().unwrap();
        assert_eq!(contract.status, ContractStatus::Ready);

        // Ready -> Claimed
        contract.claim("agent-1").unwrap();
        assert_eq!(contract.status, ContractStatus::Claimed);
        assert_eq!(contract.owner, Some("agent-1".to_string()));

        // Claimed -> Executing
        contract.start().unwrap();
        assert_eq!(contract.status, ContractStatus::Executing);

        // Executing -> Verifying
        contract.begin_verify().unwrap();
        assert_eq!(contract.status, ContractStatus::Verifying);

        // Verifying -> Completed
        contract.complete(true, Some("All tests passed".to_string()));
        assert_eq!(contract.status, ContractStatus::Completed);
        assert!(contract.completed_at.is_some());
        assert_eq!(contract.output, Some("All tests passed".to_string()));
    }

    #[test]
    fn test_failure_path() {
        let mut contract = Contract::new("task", "verify");
        contract.mark_ready().unwrap();
        contract.claim("agent-1").unwrap();
        contract.start().unwrap();
        contract.begin_verify().unwrap();
        contract.complete(false, Some("Test failed".to_string()));

        assert_eq!(contract.status, ContractStatus::Failed);
    }

    #[test]
    fn test_unclaim() {
        let mut contract = Contract::new("task", "verify");
        contract.mark_ready().unwrap();
        contract.claim("agent-1").unwrap();
        contract.unclaim().unwrap();

        assert_eq!(contract.status, ContractStatus::Ready);
        assert!(contract.owner.is_none());
    }

    #[test]
    fn test_cancel_from_pending() {
        let mut contract = Contract::new("task", "verify");
        contract.cancel().unwrap();
        assert_eq!(contract.status, ContractStatus::Cancelled);
        assert!(contract.status.is_terminal());
    }

    #[test]
    fn test_invalid_transition() {
        let mut contract = Contract::new("task", "verify");
        // Can't go directly from Pending to Executing
        let result = contract.start();
        assert!(result.is_err());
        assert_eq!(contract.status, ContractStatus::Pending); // unchanged
    }

    #[test]
    fn test_cannot_transition_from_terminal() {
        let mut contract = Contract::new("task", "verify");
        contract.mark_ready().unwrap();
        contract.claim("agent").unwrap();
        contract.start().unwrap();
        contract.begin_verify().unwrap();
        contract.complete(true, None);

        // Completed is terminal — can't go anywhere
        assert!(contract.status.is_terminal());
        let result = contract.cancel();
        assert!(result.is_err());
    }

    #[test]
    fn test_retry_after_failure() {
        let mut contract = Contract::new("task", "verify");
        contract.mark_ready().unwrap();
        contract.claim("agent").unwrap();
        contract.start().unwrap();
        contract.begin_verify().unwrap();
        contract.complete(false, Some("oops".to_string()));

        // Failed -> Ready (retry)
        contract.mark_ready().unwrap();
        assert_eq!(contract.status, ContractStatus::Ready);
    }

    #[test]
    fn test_valid_transitions() {
        assert!(ContractStatus::Pending.can_transition_to(ContractStatus::Ready));
        assert!(ContractStatus::Pending.can_transition_to(ContractStatus::Cancelled));
        assert!(!ContractStatus::Pending.can_transition_to(ContractStatus::Executing));
        assert!(!ContractStatus::Completed.can_transition_to(ContractStatus::Failed));
    }

    #[test]
    fn test_status_serialization() {
        let status = ContractStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"completed\"");

        let parsed: ContractStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ContractStatus::Completed);
    }

    #[test]
    fn test_new_status_serialization() {
        for (status, expected) in [
            (ContractStatus::Ready, "\"ready\""),
            (ContractStatus::Claimed, "\"claimed\""),
            (ContractStatus::Executing, "\"executing\""),
            (ContractStatus::Verifying, "\"verifying\""),
            (ContractStatus::RollingBack, "\"rollingback\""),
            (ContractStatus::RolledBack, "\"rolledback\""),
            (ContractStatus::Cancelled, "\"cancelled\""),
        ] {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, expected);
            let parsed: ContractStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn test_contract_serialization() {
        let contract = Contract::new("task", "verify");
        let json = serde_json::to_string(&contract).unwrap();

        assert!(json.contains("\"task\":\"task\""));
        assert!(json.contains("\"verification\":\"verify\""));
        assert!(json.contains("\"status\":\"pending\""));

        // Optional/empty fields should be omitted
        assert!(!json.contains("completed_at"));
        assert!(!json.contains("output"));
        assert!(!json.contains("owner"));
        assert!(!json.contains("blocked_by"));
        assert!(!json.contains("blocks"));
    }

    #[test]
    fn test_contract_deserialization() {
        let json = r#"{
            "id": "test123",
            "task": "fix bug",
            "verification": "cargo test",
            "status": "completed",
            "created_at": "2026-02-03T12:00:00Z",
            "completed_at": "2026-02-03T12:05:00Z",
            "output": "Success",
            "owner": "agent-1",
            "blocked_by": ["dep-1"],
            "blocks": ["next-1"]
        }"#;

        let contract: Contract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.id, "test123");
        assert_eq!(contract.status, ContractStatus::Completed);
        assert!(contract.completed_at.is_some());
        assert_eq!(contract.owner, Some("agent-1".to_string()));
        assert_eq!(contract.blocked_by, vec!["dep-1"]);
        assert_eq!(contract.blocks, vec!["next-1"]);
    }

    #[test]
    fn test_backward_compat_deserialization() {
        // Old format without new fields should still parse
        let json = r#"{
            "id": "test123",
            "task": "fix bug",
            "verification": "cargo test",
            "status": "pending",
            "created_at": "2026-02-03T12:00:00Z"
        }"#;

        let contract: Contract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.status, ContractStatus::Pending);
        assert!(contract.owner.is_none());
        assert!(contract.blocked_by.is_empty());
    }

    #[test]
    fn test_id_generation() {
        let id1 = generate_id();
        let id2 = generate_id();

        assert_ne!(id1, id2);
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
        assert_eq!(ContractStatus::Ready.to_string(), "ready");
        assert_eq!(ContractStatus::Claimed.to_string(), "claimed");
        assert_eq!(ContractStatus::Executing.to_string(), "executing");
        assert_eq!(ContractStatus::Verifying.to_string(), "verifying");
        assert_eq!(ContractStatus::Completed.to_string(), "completed");
        assert_eq!(ContractStatus::Failed.to_string(), "failed");
        assert_eq!(ContractStatus::RollingBack.to_string(), "rollingback");
        assert_eq!(ContractStatus::RolledBack.to_string(), "rolledback");
        assert_eq!(ContractStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_status_from_str() {
        assert_eq!(
            "pending".parse::<ContractStatus>().unwrap(),
            ContractStatus::Pending
        );
        assert_eq!(
            "ready".parse::<ContractStatus>().unwrap(),
            ContractStatus::Ready
        );
        assert_eq!(
            "executing".parse::<ContractStatus>().unwrap(),
            ContractStatus::Executing
        );
        assert!("bogus".parse::<ContractStatus>().is_err());
    }
}
