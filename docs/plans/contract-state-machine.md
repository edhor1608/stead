# Contract State Machine Specification (M6)

**Created:** 2026-02-05
**Status:** Draft
**Author:** Design spec for M6 implementation

---

## 1. State Diagram

```
                          cancel()
              +---------------------------------+
              |                                 |
              |         cancel()                |
              |    +------------------+         |
              |    |                  |         |
              v    v                  |         |
         +---------+  deps_met  +---------+    |
  new()  | Pending |----------->|  Ready  |----+---->+-----------+
         +---------+            +---------+         | Cancelled |
                                    |               +-----------+
                                    | claim(owner)        ^
                                    v                     |
                               +---------+  cancel()      |
                               | Claimed |----------------+
                               +---------+
                                    |
                                    | start()
                                    v
                              +-----------+  cancel()
                              | Executing |----------->+-----------+
                              +-----------+            | Cancelled |
                                    |                  +-----------+
                                    | verify()
                                    v
                              +-----------+
                              | Verifying |
                              +-----------+
                                   / \
                        pass /       \ fail
                            v         v
                   +-----------+  +---------+
                   | Completed |  | Failed  |
                   +-----------+  +---------+
                                      |
                                      | rollback()
                                      v
                                +-------------+
                                | RollingBack |
                                +-------------+
                                      |
                                      | rollback_done()
                                      v
                                +------------+
                                | RolledBack |
                                +------------+
```

### Compact Transition Table

| From         | To           | Trigger             |
|--------------|--------------|---------------------|
| Pending      | Ready        | `deps_met` (system) |
| Pending      | Cancelled    | `cancel`            |
| Ready        | Claimed      | `claim(owner)`      |
| Ready        | Cancelled    | `cancel`            |
| Claimed      | Executing    | `start`             |
| Claimed      | Ready        | `unclaim`           |
| Claimed      | Cancelled    | `cancel`            |
| Executing    | Verifying    | `verify`            |
| Executing    | Cancelled    | `cancel`            |
| Verifying    | Completed    | `pass` (system)     |
| Verifying    | Failed       | `fail` (system)     |
| Failed       | RollingBack  | `rollback`          |
| RollingBack  | RolledBack   | `rollback_done`     |

**Terminal states:** Completed, RolledBack, Cancelled

---

## 2. Invalid Transitions (Explicitly Rejected)

Any transition not in the table above is invalid. Notable cases to guard against:

| Attempted                    | Why Invalid                                       |
|------------------------------|---------------------------------------------------|
| Pending -> Claimed           | Must pass through Ready (deps must be met first)  |
| Pending -> Executing         | Cannot skip claim                                 |
| Ready -> Executing           | Must be claimed first                             |
| Executing -> Completed       | Must pass through Verifying                       |
| Executing -> Failed          | Must pass through Verifying                       |
| Completed -> anything        | Terminal state                                    |
| RolledBack -> anything       | Terminal state                                    |
| Cancelled -> anything        | Terminal state                                    |
| Failed -> Ready              | Cannot retry directly; rollback first or create new contract |
| Verifying -> Cancelled       | Verification started, must complete               |
| RollingBack -> Cancelled     | Rollback started, must complete                   |

---

## 3. Transition Guards

### Pending -> Ready (`deps_met`)
- **Trigger:** System-initiated. Checked when any contract completes.
- **Guard:** All contracts in `blocked_by` list have status `Completed`.
- **If `blocked_by` is empty:** Contract starts as Ready directly (skip Pending).

### Ready -> Claimed (`claim`)
- **Trigger:** Agent or human calls `claim(contract_id, owner)`.
- **Guard:** `owner` must be a non-empty string.
- **Side effect:** Sets `claimed_at = now()`, `owner = owner`.

### Claimed -> Executing (`start`)
- **Trigger:** The owner (agent or human) calls `start(contract_id)`.
- **Guard:** Caller must match `owner`. Only the owner starts their own contract.
- **Side effect:** Sets `started_at = now()`.

### Claimed -> Ready (`unclaim`)
- **Trigger:** Owner releases the contract.
- **Guard:** Caller must match `owner`.
- **Side effect:** Clears `owner`, `claimed_at`.

### Executing -> Verifying (`verify`)
- **Trigger:** Owner signals work is done, system runs verification command.
- **Guard:** Caller must match `owner`.
- **Side effect:** System begins executing `verification` command.

### Verifying -> Completed (`pass`)
- **Trigger:** System-initiated. Verification command exits 0.
- **Side effect:** Sets `completed_at = now()`. Triggers `deps_met` check on contracts where this ID is in their `blocked_by`.

### Verifying -> Failed (`fail`)
- **Trigger:** System-initiated. Verification command exits non-zero.
- **Side effect:** Sets `completed_at = now()`, `failed_reason = verification output`.

### Failed -> RollingBack (`rollback`)
- **Trigger:** Owner or human calls `rollback(contract_id)`.
- **Guard:** Contract has a `rollback` command defined. If no rollback command, this transition is unavailable.

### RollingBack -> RolledBack (`rollback_done`)
- **Trigger:** System-initiated. Rollback command exits (any exit code).
- **Side effect:** Sets `rolled_back_at = now()`.

### * -> Cancelled (`cancel`)
- **Trigger:** Human calls `cancel(contract_id)`.
- **Guard:** Contract is NOT in Verifying or RollingBack (in-flight operations must complete). Contract is NOT in a terminal state.
- **Who:** Human only. Agents cannot cancel.
- **Side effect:** Sets `cancelled_at = now()`.

---

## 4. Actor Permissions

| Transition       | System | Agent (owner) | Human |
|------------------|--------|---------------|-------|
| deps_met         | X      |               |       |
| claim            |        | X             | X     |
| unclaim          |        | X             | X     |
| start            |        | X             | X     |
| verify           |        | X             | X     |
| pass             | X      |               |       |
| fail             | X      |               |       |
| rollback         |        | X             | X     |
| rollback_done    | X      |               |       |
| cancel           |        |               | X     |

"System" means stead-core itself, triggered by internal logic (verification exit code, dependency resolution).

---

## 5. Data Model Changes

### Current Contract struct

```rust
pub struct Contract {
    pub id: String,
    pub task: String,
    pub verification: String,
    pub status: ContractStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: Option<String>,
}
```

### New Contract struct

```rust
pub struct Contract {
    // --- Existing (unchanged) ---
    pub id: String,
    pub task: String,
    pub verification: String,
    pub status: ContractStatus,
    pub created_at: DateTime<Utc>,
    pub output: Option<String>,

    // --- New fields ---
    pub owner: Option<String>,           // Who claimed it (agent name or human)
    pub blocked_by: Vec<String>,         // Contract IDs that must complete first
    pub blocks: Vec<String>,            // Contract IDs waiting on this one
    pub rollback_cmd: Option<String>,    // Shell command to undo work
    pub failed_reason: Option<String>,   // Why verification failed

    // --- Timestamps for each phase ---
    pub claimed_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub rolled_back_at: Option<DateTime<Utc>>,
}
```

### New ContractStatus enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
```

### SQLite Schema Migration (4-state -> 10-state)

Current M3 schema (from phase2-revised.md):

```sql
CREATE TABLE contracts (
    id TEXT PRIMARY KEY,
    task TEXT NOT NULL,
    verify_cmd TEXT NOT NULL,
    status TEXT NOT NULL,
    output TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    project_path TEXT NOT NULL
);
```

Migration to add M6 fields:

```sql
-- New columns
ALTER TABLE contracts ADD COLUMN owner TEXT;
ALTER TABLE contracts ADD COLUMN rollback_cmd TEXT;
ALTER TABLE contracts ADD COLUMN failed_reason TEXT;
ALTER TABLE contracts ADD COLUMN claimed_at TEXT;
ALTER TABLE contracts ADD COLUMN started_at TEXT;
ALTER TABLE contracts ADD COLUMN completed_at TEXT;
ALTER TABLE contracts ADD COLUMN cancelled_at TEXT;
ALTER TABLE contracts ADD COLUMN rolled_back_at TEXT;

-- Dependency tracking (separate table - many-to-many)
CREATE TABLE contract_dependencies (
    contract_id TEXT NOT NULL,
    depends_on_id TEXT NOT NULL,
    PRIMARY KEY (contract_id, depends_on_id),
    FOREIGN KEY (contract_id) REFERENCES contracts(id),
    FOREIGN KEY (depends_on_id) REFERENCES contracts(id)
);

CREATE INDEX idx_deps_contract ON contract_dependencies(contract_id);
CREATE INDEX idx_deps_depends_on ON contract_dependencies(depends_on_id);

-- Status migration: map old states to new states
-- pending -> pending (or ready, if no dependencies)
-- running -> executing
-- passed  -> completed
-- failed  -> failed
UPDATE contracts SET status = 'executing' WHERE status = 'running';
UPDATE contracts SET status = 'completed' WHERE status = 'passed';
-- 'pending' and 'failed' keep their names

-- New index for owner queries
CREATE INDEX idx_contracts_owner ON contracts(owner);
```

Note: `blocked_by` and `blocks` are not stored as columns on `contracts`. They are derived from the `contract_dependencies` join table. The struct fields are populated at read time.

---

## 6. API Surface

### Core Transition Functions

All return `Result<Contract, ContractError>`. Each validates the current state before transitioning.

```rust
/// Create a new contract. Starts as Ready if blocked_by is empty, Pending otherwise.
fn create(task: &str, verification: &str, opts: CreateOpts) -> Result<Contract, ContractError>;

/// System: check if a Pending contract's dependencies are all Completed.
/// If so, transition to Ready. Called after any contract reaches Completed.
fn resolve_dependencies(contract_id: &str) -> Result<Contract, ContractError>;

/// Agent/Human: claim a Ready contract.
fn claim(contract_id: &str, owner: &str) -> Result<Contract, ContractError>;

/// Agent/Human: release a Claimed contract back to Ready.
fn unclaim(contract_id: &str, owner: &str) -> Result<Contract, ContractError>;

/// Agent/Human: start executing a Claimed contract.
fn start(contract_id: &str, owner: &str) -> Result<Contract, ContractError>;

/// Agent/Human: signal work is done, trigger verification.
fn verify(contract_id: &str, owner: &str) -> Result<Contract, ContractError>;

/// System: mark verification passed. Triggers dependency resolution on downstream contracts.
fn pass(contract_id: &str, output: Option<&str>) -> Result<Contract, ContractError>;

/// System: mark verification failed.
fn fail(contract_id: &str, reason: &str, output: Option<&str>) -> Result<Contract, ContractError>;

/// Agent/Human: trigger rollback on a Failed contract.
fn rollback(contract_id: &str) -> Result<Contract, ContractError>;

/// System: mark rollback complete.
fn rollback_done(contract_id: &str) -> Result<Contract, ContractError>;

/// Human only: cancel a contract (if not in Verifying/RollingBack/terminal).
fn cancel(contract_id: &str) -> Result<Contract, ContractError>;
```

### CreateOpts

```rust
pub struct CreateOpts {
    pub blocked_by: Vec<String>,     // Contract IDs to depend on
    pub rollback_cmd: Option<String>, // Optional rollback command
}
```

### ContractError

```rust
#[derive(Debug, Error)]
pub enum ContractError {
    #[error("contract not found: {0}")]
    NotFound(String),

    #[error("invalid transition: {from} -> {to}")]
    InvalidTransition {
        from: ContractStatus,
        to: ContractStatus,
    },

    #[error("not owner: expected {expected}, got {actual}")]
    NotOwner {
        expected: String,
        actual: String,
    },

    #[error("dependency not found: {0}")]
    DependencyNotFound(String),

    #[error("circular dependency detected")]
    CircularDependency,

    #[error("no rollback command defined")]
    NoRollbackCommand,

    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
}
```

### CLI Commands (stead-cli)

```
stead contract create "task" --verify "cmd" [--blocked-by ID,...] [--rollback "cmd"]
stead contract claim ID [--owner NAME]     # defaults to hostname or $USER
stead contract unclaim ID
stead contract start ID
stead contract verify ID
stead contract rollback ID
stead contract cancel ID
stead contract show ID                     # existing, updated for new fields
stead contract list                        # existing, updated for new states
```

---

## 7. Test Matrix

### Valid Transitions (13 tests)

| # | Test                                      |
|---|-------------------------------------------|
| 1 | create with no deps -> Ready              |
| 2 | create with deps -> Pending               |
| 3 | Pending -> Ready (deps_met)               |
| 4 | Ready -> Claimed (claim)                  |
| 5 | Claimed -> Executing (start)              |
| 6 | Claimed -> Ready (unclaim)                |
| 7 | Executing -> Verifying (verify)           |
| 8 | Verifying -> Completed (pass)             |
| 9 | Verifying -> Failed (fail)                |
| 10| Failed -> RollingBack (rollback)          |
| 11| RollingBack -> RolledBack (rollback_done) |
| 12| Ready -> Cancelled (cancel)               |
| 13| Executing -> Cancelled (cancel)           |

### Invalid Transitions (15 tests minimum)

| # | Test                                          |
|---|-----------------------------------------------|
| 1 | Pending -> Claimed (skip Ready)               |
| 2 | Pending -> Executing                          |
| 3 | Ready -> Executing (skip Claimed)             |
| 4 | Executing -> Completed (skip Verifying)       |
| 5 | Executing -> Failed (skip Verifying)          |
| 6 | Completed -> anything (terminal)              |
| 7 | RolledBack -> anything (terminal)             |
| 8 | Cancelled -> anything (terminal)              |
| 9 | Verifying -> Cancelled (in-flight)            |
| 10| RollingBack -> Cancelled (in-flight)          |
| 11| Failed -> Ready (no direct retry)             |
| 12| claim with wrong state (e.g., Executing)      |
| 13| start by non-owner                            |
| 14| verify by non-owner                           |
| 15| rollback without rollback_cmd defined         |

### Guard Condition Tests (8 tests)

| # | Test                                                       |
|---|------------------------------------------------------------|
| 1 | deps_met only fires when ALL blocked_by are Completed      |
| 2 | deps_met does NOT fire if any blocked_by is not Completed  |
| 3 | claim sets owner and claimed_at                            |
| 4 | unclaim clears owner and claimed_at                        |
| 5 | start sets started_at                                      |
| 6 | pass sets completed_at and triggers downstream deps_met    |
| 7 | fail sets failed_reason and completed_at                   |
| 8 | circular dependency rejected at create time                |

### Integration / Lifecycle Tests (4 tests)

| # | Test                                                         |
|---|--------------------------------------------------------------|
| 1 | Full happy path: create -> claim -> start -> verify -> pass  |
| 2 | Failure path: create -> claim -> start -> verify -> fail -> rollback -> rollback_done |
| 3 | Dependency chain: A blocks B, complete A, B becomes Ready    |
| 4 | Cancel mid-flight: create -> claim -> cancel                 |

### Total: ~40 tests

- 13 valid transitions
- 15 invalid transitions
- 8 guard conditions
- 4 integration/lifecycle

---

## 8. Implementation Notes

### Dependency Resolution Strategy

When a contract reaches `Completed`, scan all `Pending` contracts where this contract's ID appears in their `blocked_by`. For each, check if ALL `blocked_by` entries are now `Completed`. If yes, transition to `Ready`.

This is an O(n) scan over pending contracts. Fine for expected scale (tens to low hundreds of contracts per project). If this becomes a bottleneck, add a reverse index, but don't optimize prematurely.

### Verification Execution

`verify()` transitions to Verifying, then the system runs the verification command. On completion, the system calls either `pass()` or `fail()`. The Verifying state exists so the UI can show "verification running" distinctly from "agent working."

This should be synchronous within a single `verify()` call from the CLI perspective: `stead contract verify ID` runs the command and reports pass/fail. Internally, the state machine goes Executing -> Verifying -> Completed/Failed in one user-facing operation.

For the library API, expose both the combined `verify_and_complete()` convenience and the granular `verify()` + `pass()`/`fail()` for the SwiftUI app where you want to show the intermediate state.

### Rollback Execution

Same pattern as verification. `rollback()` transitions to RollingBack, runs the rollback command, then `rollback_done()` transitions to RolledBack. Exit code of rollback command is logged but doesn't affect the transition -- rollback always completes (you can't fail a rollback; you deal with it manually if it goes wrong).

### Status Serialization

Use `snake_case` for serialization to match SQLite storage:
- `rolling_back` not `rollingback`
- `rolled_back` not `rolledback`

This avoids ambiguity in the TEXT column.

### Migration Path from Current 4-State

The current code uses `start()` and `complete(passed, output)` directly on the struct. The new API replaces this with the state machine functions above. Existing code (CLI commands `run`, `verify`) will call the appropriate sequence.

Map current states:
- `Pending` -> `Ready` (current contracts have no dependencies)
- `Running` -> `Executing`
- `Passed` -> `Completed`
- `Failed` -> `Failed`

The `Running` -> `Executing` rename reflects that "running" is ambiguous (running what?), while "executing" clearly means "the agent is doing the work."
