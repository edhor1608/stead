//! stead FFI bindings via UniFFI
//!
//! Exposes stead-core types and functions for Swift (and other languages).
//! Types are converted to UniFFI-compatible equivalents (e.g., DateTime -> String).

uniffi::setup_scaffolding!();

use std::path::Path;

use stead_core::storage::Storage;

// -- FFI Enum types --

#[derive(uniffi::Enum)]
pub enum FfiContractStatus {
    Pending,
    Ready,
    Claimed,
    Executing,
    Verifying,
    Completed,
    Failed,
    RollingBack,
    RolledBack,
    Cancelled,
}

impl From<stead_core::schema::ContractStatus> for FfiContractStatus {
    fn from(s: stead_core::schema::ContractStatus) -> Self {
        match s {
            stead_core::schema::ContractStatus::Pending => Self::Pending,
            stead_core::schema::ContractStatus::Ready => Self::Ready,
            stead_core::schema::ContractStatus::Claimed => Self::Claimed,
            stead_core::schema::ContractStatus::Executing => Self::Executing,
            stead_core::schema::ContractStatus::Verifying => Self::Verifying,
            stead_core::schema::ContractStatus::Completed => Self::Completed,
            stead_core::schema::ContractStatus::Failed => Self::Failed,
            stead_core::schema::ContractStatus::RollingBack => Self::RollingBack,
            stead_core::schema::ContractStatus::RolledBack => Self::RolledBack,
            stead_core::schema::ContractStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(uniffi::Enum)]
pub enum FfiCliType {
    Claude,
    Codex,
    OpenCode,
    Universal,
}

impl From<stead_core::usf::CliType> for FfiCliType {
    fn from(c: stead_core::usf::CliType) -> Self {
        match c {
            stead_core::usf::CliType::Claude => Self::Claude,
            stead_core::usf::CliType::Codex => Self::Codex,
            stead_core::usf::CliType::OpenCode => Self::OpenCode,
            stead_core::usf::CliType::Universal => Self::Universal,
        }
    }
}

// -- FFI Record types --

#[derive(uniffi::Record)]
pub struct FfiContract {
    pub id: String,
    pub project_path: String,
    pub task: String,
    pub verification: String,
    pub status: FfiContractStatus,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub output: Option<String>,
    pub owner: Option<String>,
    pub blocked_by: Vec<String>,
    pub blocks: Vec<String>,
}

impl From<stead_core::schema::Contract> for FfiContract {
    fn from(c: stead_core::schema::Contract) -> Self {
        Self {
            id: c.id,
            project_path: c.project_path,
            task: c.task,
            verification: c.verification,
            status: c.status.into(),
            created_at: c.created_at.to_rfc3339(),
            completed_at: c.completed_at.map(|dt| dt.to_rfc3339()),
            output: c.output,
            owner: c.owner,
            blocked_by: c.blocked_by,
            blocks: c.blocks,
        }
    }
}

#[derive(uniffi::Record)]
pub struct FfiSessionSummary {
    pub id: String,
    pub cli: FfiCliType,
    pub project_path: String,
    pub title: String,
    pub created: String,
    pub last_modified: String,
    pub message_count: u32,
    pub git_branch: Option<String>,
}

impl From<stead_core::usf::SessionSummary> for FfiSessionSummary {
    fn from(s: stead_core::usf::SessionSummary) -> Self {
        Self {
            id: s.id,
            cli: s.cli.into(),
            project_path: s.project_path,
            title: s.title,
            created: s.created.to_rfc3339(),
            last_modified: s.last_modified.to_rfc3339(),
            message_count: s.message_count as u32,
            git_branch: s.git_branch,
        }
    }
}

// -- FFI Error type --

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FfiError {
    #[error("Storage error: {message}")]
    Storage { message: String },
    #[error("Not found: {id}")]
    NotFound { id: String },
}

// -- Exported functions --

#[uniffi::export]
pub fn list_contracts(cwd: String) -> Result<Vec<FfiContract>, FfiError> {
    let db = stead_core::storage::sqlite::open_default(Path::new(&cwd)).map_err(|e| {
        FfiError::Storage {
            message: e.to_string(),
        }
    })?;
    let contracts = db.load_all_contracts().map_err(|e| FfiError::Storage {
        message: e.to_string(),
    })?;
    Ok(contracts.into_iter().map(FfiContract::from).collect())
}

#[uniffi::export]
pub fn get_contract(id: String, cwd: String) -> Result<FfiContract, FfiError> {
    let db = stead_core::storage::sqlite::open_default(Path::new(&cwd)).map_err(|e| {
        FfiError::Storage {
            message: e.to_string(),
        }
    })?;
    let contract = db
        .load_contract(&id)
        .map_err(|e| FfiError::Storage {
            message: e.to_string(),
        })?
        .ok_or(FfiError::NotFound { id })?;
    Ok(FfiContract::from(contract))
}

#[uniffi::export]
pub fn claim_contract(id: String, owner: String, cwd: String) -> Result<FfiContract, FfiError> {
    let db = stead_core::storage::sqlite::open_default(Path::new(&cwd)).map_err(|e| {
        FfiError::Storage {
            message: e.to_string(),
        }
    })?;

    let mut contract = db
        .load_contract(&id)
        .map_err(|e| FfiError::Storage {
            message: e.to_string(),
        })?
        .ok_or(FfiError::NotFound { id })?;

    if contract.status == stead_core::schema::ContractStatus::Pending {
        contract.mark_ready().map_err(|e| FfiError::Storage {
            message: e.to_string(),
        })?;
    }
    contract.claim(owner).map_err(|e| FfiError::Storage {
        message: e.to_string(),
    })?;

    db.update_contract(&contract)
        .map_err(|e| FfiError::Storage {
            message: e.to_string(),
        })?;

    Ok(FfiContract::from(contract))
}

#[uniffi::export]
pub fn cancel_contract(id: String, cwd: String) -> Result<FfiContract, FfiError> {
    let db = stead_core::storage::sqlite::open_default(Path::new(&cwd)).map_err(|e| {
        FfiError::Storage {
            message: e.to_string(),
        }
    })?;

    let mut contract = db
        .load_contract(&id)
        .map_err(|e| FfiError::Storage {
            message: e.to_string(),
        })?
        .ok_or(FfiError::NotFound { id })?;

    contract.cancel().map_err(|e| FfiError::Storage {
        message: e.to_string(),
    })?;

    db.update_contract(&contract)
        .map_err(|e| FfiError::Storage {
            message: e.to_string(),
        })?;

    Ok(FfiContract::from(contract))
}

#[uniffi::export]
pub fn verify_contract(id: String, cwd: String) -> Result<FfiContract, FfiError> {
    use std::process::Command;

    let db = stead_core::storage::sqlite::open_default(Path::new(&cwd)).map_err(|e| {
        FfiError::Storage {
            message: e.to_string(),
        }
    })?;

    let mut contract = db
        .load_contract(&id)
        .map_err(|e| FfiError::Storage {
            message: e.to_string(),
        })?
        .ok_or(FfiError::NotFound { id })?;

    let (shell, flag) = if cfg!(target_os = "windows") {
        ("cmd", "/c")
    } else {
        ("sh", "-c")
    };

    let output = Command::new(shell)
        .args([flag, &contract.verification])
        .output()
        .map_err(|e| FfiError::Storage {
            message: format!("Failed to run verification: {}", e),
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = [stdout.trim(), stderr.trim()]
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let output_str = if combined.is_empty() {
        None
    } else {
        Some(combined)
    };

    contract.complete(output.status.success(), output_str);

    db.update_contract(&contract)
        .map_err(|e| FfiError::Storage {
            message: e.to_string(),
        })?;

    Ok(FfiContract::from(contract))
}

#[uniffi::export]
pub fn list_sessions(
    cli_filter: Option<String>,
    project: Option<String>,
    limit: u32,
) -> Vec<FfiSessionSummary> {
    let mut sessions = stead_core::usf::adapters::discover_all_sessions();

    if let Some(cli) = &cli_filter {
        let cli_lower = cli.to_lowercase();
        sessions.retain(|s| s.cli.as_str() == cli_lower);
    }

    if let Some(proj) = &project {
        let proj_lower = proj.to_lowercase();
        sessions.retain(|s| s.project_path.to_lowercase().contains(&proj_lower));
    }

    sessions.truncate(limit as usize);
    sessions.into_iter().map(FfiSessionSummary::from).collect()
}
