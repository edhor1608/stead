use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContractStatus {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    System,
    Agent,
    Human,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionAction {
    DepsMet,
    Claim,
    Unclaim,
    Start,
    Verify,
    Pass,
    Fail,
    Rollback,
    RollbackDone,
    Cancel,
}

impl TransitionAction {
    pub fn is_allowed_for(self, actor: Actor) -> bool {
        use Actor::*;
        use TransitionAction::*;

        match (self, actor) {
            (DepsMet, System) => true,
            (Claim, Agent | Human) => true,
            (Unclaim, Agent | Human) => true,
            (Start, Agent | Human) => true,
            (Verify, Agent | Human) => true,
            (Pass, System) => true,
            (Fail, System) => true,
            (Rollback, Agent | Human) => true,
            (RollbackDone, System) => true,
            (Cancel, Human) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionError {
    pub from: ContractStatus,
    pub to: ContractStatus,
}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid transition from {:?} to {:?}",
            self.from, self.to
        )
    }
}

impl std::error::Error for TransitionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contract {
    pub id: String,
    pub status: ContractStatus,
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractEvent {
    pub contract_id: String,
    pub from: ContractStatus,
    pub to: ContractStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionTier {
    NeedsDecision,
    Anomaly,
    Completed,
    Running,
    Queued,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionItem {
    pub id: i64,
    pub contract_id: String,
    pub summary: String,
}

impl Contract {
    pub fn new(id: impl Into<String>, blocked_by: Vec<String>) -> Self {
        let status = if blocked_by.is_empty() {
            ContractStatus::Ready
        } else {
            ContractStatus::Pending
        };

        Self {
            id: id.into(),
            status,
            blocked_by,
        }
    }

    pub fn transition_to(
        &mut self,
        target: ContractStatus,
    ) -> Result<ContractEvent, TransitionError> {
        let from = self.status;
        let to = from.transition_to(target)?;
        self.status = to;

        Ok(ContractEvent {
            contract_id: self.id.clone(),
            from,
            to,
        })
    }

    pub fn finish_verification(&mut self, passed: bool) -> Result<ContractEvent, TransitionError> {
        let target = if passed {
            ContractStatus::Completed
        } else {
            ContractStatus::Failed
        };

        self.transition_to(target)
    }

    pub fn rollback(&mut self) -> Result<ContractEvent, TransitionError> {
        self.transition_to(ContractStatus::RollingBack)
    }

    pub fn cancel(&mut self) -> Result<ContractEvent, TransitionError> {
        self.transition_to(ContractStatus::Cancelled)
    }
}

impl ContractStatus {
    pub fn valid_transitions(self) -> &'static [ContractStatus] {
        use ContractStatus::*;

        match self {
            Pending => &[Ready, Cancelled],
            Ready => &[Claimed, Cancelled],
            Claimed => &[Executing, Ready, Cancelled],
            Executing => &[Verifying, Failed, Cancelled],
            Verifying => &[Completed, Failed],
            Completed => &[],
            Failed => &[Ready, RollingBack, Cancelled],
            RollingBack => &[RolledBack, Failed],
            RolledBack => &[],
            Cancelled => &[],
        }
    }

    pub fn can_transition_to(self, target: ContractStatus) -> bool {
        self.valid_transitions().contains(&target)
    }

    pub fn transition_to(self, target: ContractStatus) -> Result<ContractStatus, TransitionError> {
        if self.can_transition_to(target) {
            Ok(target)
        } else {
            Err(TransitionError {
                from: self,
                to: target,
            })
        }
    }

    fn as_db_str(self) -> &'static str {
        match self {
            ContractStatus::Pending => "pending",
            ContractStatus::Ready => "ready",
            ContractStatus::Claimed => "claimed",
            ContractStatus::Executing => "executing",
            ContractStatus::Verifying => "verifying",
            ContractStatus::Completed => "completed",
            ContractStatus::Failed => "failed",
            ContractStatus::RollingBack => "rolling_back",
            ContractStatus::RolledBack => "rolled_back",
            ContractStatus::Cancelled => "cancelled",
        }
    }

    fn from_db_str(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(ContractStatus::Pending),
            "ready" => Some(ContractStatus::Ready),
            "claimed" => Some(ContractStatus::Claimed),
            "executing" => Some(ContractStatus::Executing),
            "verifying" => Some(ContractStatus::Verifying),
            "completed" => Some(ContractStatus::Completed),
            "failed" => Some(ContractStatus::Failed),
            "rolling_back" => Some(ContractStatus::RollingBack),
            "rolled_back" => Some(ContractStatus::RolledBack),
            "cancelled" => Some(ContractStatus::Cancelled),
            _ => None,
        }
    }
}

pub fn crate_identity() -> &'static str {
    "stead-contracts"
}

pub const CURRENT_SCHEMA_VERSION: i64 = 2;

#[derive(Debug, Clone)]
pub struct SqliteContractStore {
    db_path: PathBuf,
}

impl SqliteContractStore {
    pub fn open(db_path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let store = Self {
            db_path: db_path.as_ref().to_path_buf(),
        };

        let conn = store.connection()?;
        store.bootstrap_schema(&conn)?;

        Ok(store)
    }

    pub fn schema_version(&self) -> rusqlite::Result<i64> {
        let conn = self.connection()?;
        conn.query_row(
            "SELECT value FROM schema_meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
    }

    pub fn save_contract(&self, contract: &Contract) -> rusqlite::Result<()> {
        let conn = self.connection()?;
        let blocked_by = serde_json::to_string(&contract.blocked_by)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;

        conn.execute(
            "INSERT INTO contracts (id, status, blocked_by)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                blocked_by = excluded.blocked_by",
            params![contract.id, contract.status.as_db_str(), blocked_by],
        )?;

        Ok(())
    }

    pub fn load_contract(&self, id: &str) -> rusqlite::Result<Option<Contract>> {
        let conn = self.connection()?;

        conn.query_row(
            "SELECT id, status, blocked_by FROM contracts WHERE id = ?1",
            params![id],
            |row| {
                let id: String = row.get(0)?;
                let status_str: String = row.get(1)?;
                let blocked_by_str: String = row.get(2)?;

                let status = ContractStatus::from_db_str(&status_str)
                    .ok_or(rusqlite::Error::InvalidQuery)?;
                let blocked_by = serde_json::from_str(&blocked_by_str)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;

                Ok(Contract {
                    id,
                    status,
                    blocked_by,
                })
            },
        )
        .optional()
    }

    pub fn list_contracts(&self) -> rusqlite::Result<Vec<Contract>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, status, blocked_by
             FROM contracts
             ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], contract_from_row)?;
        rows.collect()
    }

    pub fn record_transition(
        &self,
        contract: &Contract,
        event: &ContractEvent,
    ) -> rusqlite::Result<()> {
        if contract.id != event.contract_id {
            return Err(rusqlite::Error::InvalidQuery);
        }

        let mut conn = self.connection()?;
        let tx = conn.transaction()?;

        let blocked_by = serde_json::to_string(&contract.blocked_by)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;

        let updated = tx.execute(
            "UPDATE contracts SET status = ?1, blocked_by = ?2 WHERE id = ?3",
            params![contract.status.as_db_str(), blocked_by, contract.id],
        )?;

        if updated == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        tx.execute(
            "INSERT INTO contract_events (contract_id, from_status, to_status, blocked_by_snapshot)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                event.contract_id,
                event.from.as_db_str(),
                event.to.as_db_str(),
                blocked_by,
            ],
        )?;

        tx.commit()
    }

    pub fn list_events(&self, contract_id: &str) -> rusqlite::Result<Vec<ContractEvent>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT contract_id, from_status, to_status
             FROM contract_events
             WHERE contract_id = ?1
             ORDER BY id ASC",
        )?;

        let rows = stmt.query_map(params![contract_id], |row| {
            let contract_id: String = row.get(0)?;
            let from_status: String = row.get(1)?;
            let to_status: String = row.get(2)?;

            let from =
                ContractStatus::from_db_str(&from_status).ok_or(rusqlite::Error::InvalidQuery)?;
            let to =
                ContractStatus::from_db_str(&to_status).ok_or(rusqlite::Error::InvalidQuery)?;

            Ok(ContractEvent {
                contract_id,
                from,
                to,
            })
        })?;

        rows.collect()
    }

    pub fn rebuild_contract_from_events(&self, id: &str) -> rusqlite::Result<Option<Contract>> {
        let snapshot = self.load_contract(id)?;
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT from_status, to_status, blocked_by_snapshot
             FROM contract_events
             WHERE contract_id = ?1
             ORDER BY id ASC",
        )?;
        let rows = stmt.query_map(params![id], |row| {
            let from_status: String = row.get(0)?;
            let to_status: String = row.get(1)?;
            let blocked_by_snapshot: String = row.get(2)?;

            let from =
                ContractStatus::from_db_str(&from_status).ok_or(rusqlite::Error::InvalidQuery)?;
            let to =
                ContractStatus::from_db_str(&to_status).ok_or(rusqlite::Error::InvalidQuery)?;
            let blocked_by = serde_json::from_str(&blocked_by_snapshot)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;

            Ok((from, to, blocked_by))
        })?;
        let events: Vec<(ContractStatus, ContractStatus, Vec<String>)> =
            rows.collect::<rusqlite::Result<_>>()?;

        if events.is_empty() {
            return Ok(snapshot);
        }

        let mut rebuilt = match snapshot {
            Some(contract) => contract,
            None => Contract::new(id, Vec::new()),
        };

        if let Some((first_from, _, first_blocked_by)) = events.first() {
            rebuilt.status = *first_from;
            rebuilt.blocked_by = first_blocked_by.clone();
        }

        for (_, to, blocked_by) in events {
            rebuilt.status = to;
            rebuilt.blocked_by = blocked_by;
        }

        Ok(Some(rebuilt))
    }

    pub fn create_decision(&self, contract_id: &str, summary: &str) -> rusqlite::Result<i64> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO decision_items (contract_id, summary, resolved)
             VALUES (?1, ?2, 0)",
            params![contract_id, summary],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_open_decisions(&self) -> rusqlite::Result<Vec<DecisionItem>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, contract_id, summary
             FROM decision_items
             WHERE resolved = 0
             ORDER BY id ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DecisionItem {
                id: row.get(0)?,
                contract_id: row.get(1)?,
                summary: row.get(2)?,
            })
        })?;

        rows.collect()
    }

    pub fn list_anomalies(&self) -> rusqlite::Result<Vec<Contract>> {
        self.list_by_attention_tier(AttentionTier::Anomaly)
    }

    pub fn list_by_attention_tier(&self, tier: AttentionTier) -> rusqlite::Result<Vec<Contract>> {
        let conn = self.connection()?;

        let sql = match tier {
            AttentionTier::NeedsDecision => {
                "SELECT DISTINCT c.id, c.status, c.blocked_by
                 FROM contracts c
                 JOIN decision_items d ON d.contract_id = c.id
                 WHERE d.resolved = 0
                 ORDER BY c.id ASC"
            }
            AttentionTier::Anomaly => {
                "SELECT id, status, blocked_by
                 FROM contracts
                 WHERE status IN ('failed', 'rolling_back', 'rolled_back')
                 ORDER BY id ASC"
            }
            AttentionTier::Completed => {
                "SELECT id, status, blocked_by
                 FROM contracts
                 WHERE status = 'completed'
                 ORDER BY id ASC"
            }
            AttentionTier::Running => {
                "SELECT id, status, blocked_by
                 FROM contracts
                 WHERE status IN ('executing', 'verifying')
                 ORDER BY id ASC"
            }
            AttentionTier::Queued => {
                "SELECT id, status, blocked_by
                 FROM contracts
                 WHERE status IN ('pending', 'ready', 'claimed')
                 ORDER BY id ASC"
            }
        };

        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], contract_from_row)?;
        rows.collect()
    }

    fn connection(&self) -> rusqlite::Result<Connection> {
        let conn = Connection::open(&self.db_path)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(conn)
    }

    fn bootstrap_schema(&self, conn: &Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_meta (
                key TEXT PRIMARY KEY,
                value INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS contracts (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                blocked_by TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS contract_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                contract_id TEXT NOT NULL,
                from_status TEXT NOT NULL,
                to_status TEXT NOT NULL,
                blocked_by_snapshot TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(contract_id) REFERENCES contracts(id)
            );

            CREATE TABLE IF NOT EXISTS decision_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                contract_id TEXT NOT NULL,
                summary TEXT NOT NULL,
                resolved INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(contract_id) REFERENCES contracts(id)
            );",
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO schema_meta (key, value) VALUES ('schema_version', ?1)",
            params![CURRENT_SCHEMA_VERSION],
        )?;

        let has_blocked_by_snapshot: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('contract_events') WHERE name = 'blocked_by_snapshot'",
            [],
            |row| row.get(0),
        )?;
        if has_blocked_by_snapshot == 0 {
            conn.execute(
                "ALTER TABLE contract_events ADD COLUMN blocked_by_snapshot TEXT NOT NULL DEFAULT '[]'",
                [],
            )?;
        }

        let version: i64 = conn.query_row(
            "SELECT value FROM schema_meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )?;
        if version < CURRENT_SCHEMA_VERSION {
            conn.execute(
                "UPDATE schema_meta SET value = ?1 WHERE key = 'schema_version'",
                params![CURRENT_SCHEMA_VERSION],
            )?;
        }

        Ok(())
    }
}

fn contract_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Contract> {
    let id: String = row.get(0)?;
    let status_str: String = row.get(1)?;
    let blocked_by_str: String = row.get(2)?;

    let status = ContractStatus::from_db_str(&status_str).ok_or(rusqlite::Error::InvalidQuery)?;
    let blocked_by =
        serde_json::from_str(&blocked_by_str).map_err(|_| rusqlite::Error::InvalidQuery)?;

    Ok(Contract {
        id,
        status,
        blocked_by,
    })
}
