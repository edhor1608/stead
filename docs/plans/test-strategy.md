# Stead Test Strategy

**Created:** 2026-02-05
**Status:** Active
**Owner:** Testing Expert

## Current State

- **114 tests passing** (98 unit + 16 integration)
- Unit tests are `#[cfg(test)]` inline modules in each source file
- Integration tests in `rust/tests/integration.rs` using `assert_cmd` + `predicates`
- Dev dependencies: `tempfile`, `assert_cmd`, `predicates`
- CI pipeline exists at `.github/workflows/ci.yml` (fmt, clippy, tests, release build)
- No code coverage tooling
- No property-based testing
- No fuzz testing

## Test Architecture by Milestone

### stead-core (Library Tests)

After M2 library split, all logic lives in `stead-core`. Tests should live alongside the code they test (inline `#[cfg(test)]` modules).

#### Contract Module (`contracts/`)

**Unit tests (existing, migrate as-is):**
- Contract creation, lifecycle (pending -> running -> passed/failed)
- ID generation uniqueness and format
- Base36 conversion edge cases
- Serde round-trip (serialize/deserialize)
- Status display formatting

**New tests needed for M6 (10-state machine):**
- Valid state transitions: `Pending -> Ready -> Claimed -> Executing -> Verifying -> Completed`
- Invalid transitions rejected: e.g., `Pending -> Completed` should fail
- Rollback path: `Failed -> RollingBack -> RolledBack`
- Cancel from any non-terminal state
- Transition guards: `Claimed` requires owner field set
- `blockedBy`/`blocks` dependency resolution
- Property-based: arbitrary sequences of state transitions never reach invalid state

```rust
// Property-based test sketch (M6)
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    prop_compose! {
        fn arb_transition()(action in prop_oneof![
            Just(Action::Claim),
            Just(Action::Start),
            Just(Action::Complete),
            Just(Action::Fail),
            Just(Action::Cancel),
            Just(Action::Rollback),
        ]) -> Action {
            action
        }
    }

    proptest! {
        #[test]
        fn state_machine_never_panics(
            actions in prop::collection::vec(arb_transition(), 0..50)
        ) {
            let mut contract = Contract::new("task", "verify");
            for action in actions {
                // apply_transition should return Result, never panic
                let _ = contract.apply_transition(action);
            }
        }
    }
}
```

#### Storage Module (`storage/`)

**Unit tests (existing, migrate as-is):**
- Write and read contract round-trip
- List empty directory
- List multiple contracts (sorted by date)
- Update existing contract
- Update nonexistent contract returns `NotFound`
- Graceful corruption handling (skip bad JSONL lines)
- Initialization check
- JSONL format verification (one line per contract)

**New tests for M3 (SQLite):**
- SQLite storage implements same `Storage` trait as JSONL
- Schema creation on fresh database
- Migration from JSONL to SQLite preserves all contracts
- Concurrent read/write safety (multiple threads)
- Index performance: query by status, query by project
- WAL mode enabled
- Database file permissions (not world-readable)
- Graceful handling of locked database
- Empty vs. nonexistent database

```rust
// Suggested Storage trait for testability
pub trait Storage {
    fn write_contract(&self, contract: &Contract) -> Result<(), StorageError>;
    fn read_contract(&self, id: &str) -> Result<Option<Contract>, StorageError>;
    fn update_contract(&self, contract: &Contract) -> Result<(), StorageError>;
    fn list_contracts(&self) -> Result<Vec<Contract>, StorageError>;
}
```

Both `JsonlStorage` and `SqliteStorage` implement this trait. Tests can be parameterized:

```rust
#[test_case(JsonlStorage::new(tmp.path()) ; "jsonl")]
#[test_case(SqliteStorage::new(tmp.path()) ; "sqlite")]
fn test_write_and_read(storage: impl Storage) {
    let contract = Contract::new("task", "verify");
    storage.write_contract(&contract).unwrap();
    let loaded = storage.read_contract(&contract.id).unwrap();
    assert_eq!(loaded.unwrap().id, contract.id);
}
```

#### USF Module (`usf/`)

**Unit tests (existing, migrate as-is):**
- Session creation with correct ID prefix
- Title extraction from first user message
- Title truncation at 60 chars
- Message counting by role
- Tool name mapping (Claude, Codex, OpenCode -> Universal)
- CLI type display
- Session serialization round-trip
- Session summary from full session

**Adapter-specific tests (existing, migrate as-is):**
- Claude: content item parsing (text, tool_use, tool_result, thinking, unknown)
- Claude: truncation helper
- Codex: session_meta entry parsing
- Codex: truncation helper
- OpenCode: timestamp conversion
- OpenCode: session/part JSON parsing
- All adapters: creation (system-dependent, graceful skip)

**New tests needed:**
- End-to-end adapter tests with fixture files (synthetic JSONL/JSON)
- Full session parse from fixture -> verify timeline structure
- Edge cases: empty sessions, sessions with only tool calls, malformed lines
- Adapter discovery: `discover_all_sessions()` with mock filesystem

### stead-cli (Binary Tests)

The CLI is a thin wrapper around `stead-core`. Test at the binary level.

**Integration tests (existing, migrate as-is):**
- `--help` output contains expected text
- `--version` output matches Cargo.toml version
- `list` on empty directory shows "No contracts found"
- `run` + `list` round-trip
- `run` with failing verification
- `show` by contract ID
- `show` nonexistent contract
- `verify` re-runs verification
- `list --status` filter (passed/failed/invalid)
- `--json` output format
- `session list --help` flags
- `session list` runs without error
- `session list --json` returns JSON array
- `session show` nonexistent session
- `session list --cli unknown` error message

**New integration tests needed:**
- `run` with `--json` flag: verify JSON structure matches Contract schema
- Concurrent `run` commands in same directory (race condition testing for M3)
- Large contract lists (100+ entries) performance check
- Binary exit codes: 0 for success, non-zero for errors
- Stderr vs stdout separation (errors to stderr, data to stdout)

### M4: FFI Boundary Tests (`stead-ffi/`)

FFI is the riskiest boundary. Every public function exposed to Swift must be tested.

**Test strategy:**
1. **Rust-side FFI tests:** Test the FFI functions from Rust (no Swift needed)
2. **Round-trip tests:** Rust -> C ABI -> Rust to verify data survives the boundary
3. **Memory safety:** Ensure no leaks when Swift drops returned values
4. **Error propagation:** Verify Rust errors become Swift-consumable errors
5. **Null/empty handling:** All optional fields handled correctly across FFI

```rust
// FFI test sketch
#[test]
fn test_ffi_list_contracts() {
    // Set up test contracts in temp dir
    let contracts = ffi::list_contracts();
    assert!(!contracts.is_empty());
    // Verify the FFI-safe types convert correctly
}

#[test]
fn test_ffi_error_on_invalid_id() {
    let result = ffi::get_contract("nonexistent".to_string());
    assert!(result.is_none()); // or appropriate error type
}
```

### M5: SwiftUI Snapshot Tests

**Approach:** Use Swift's `XCTest` + snapshot testing library (e.g., `swift-snapshot-testing`).

**Views to snapshot:**
- `ContractListView` — empty state, populated state, mixed statuses
- `SessionListView` — empty, single CLI, multiple CLIs
- Attention priority ordering (Red > Yellow > Green > White > Gray)
- Menu bar icon states
- Dark mode / light mode variants

**Test setup:**
```swift
import SnapshotTesting
import XCTest

class ContractListViewTests: XCTestCase {
    func testEmptyState() {
        let view = ContractListView(contracts: [])
        assertSnapshot(of: view, as: .image(layout: .device(config: .iPhoneSe)))
    }

    func testPopulatedState() {
        let view = ContractListView(contracts: mockContracts())
        assertSnapshot(of: view, as: .image(layout: .device(config: .iPhoneSe)))
    }
}
```

**CI consideration:** Snapshot tests need a macOS runner with Xcode. Run separately from Rust tests.

### M6: Contract State Machine Property Tests

The 10-state contract lifecycle is the highest-risk feature. Property-based testing is mandatory.

**Properties to verify:**
1. **No invalid states:** From any reachable state, only valid transitions are allowed
2. **Terminal states are final:** `Completed`, `RolledBack`, `Cancelled` cannot transition further
3. **Rollback is safe:** Failed contracts can always be rolled back
4. **Dependencies respected:** `Pending` contracts with unresolved `blockedBy` cannot move to `Ready`
5. **Ownership invariant:** `Executing` contracts always have an owner
6. **Monotonic progress:** Contracts don't regress (except rollback path)

**Dependency:** Add `proptest` to dev-dependencies.

```toml
[dev-dependencies]
proptest = "1"
```

## Test Infrastructure

### CI/CD Pipeline

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo fmt --check

  rust-coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - uses: taiki-e/install-action@cargo-llvm-cov
      - run: cargo llvm-cov --workspace --lcov --output-path lcov.info
      - uses: codecov/codecov-action@v4
        with:
          files: lcov.info

  macos-tests:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace
      # SwiftUI tests when M5 is ready
      # - run: xcodebuild test -project macos/Stead.xcodeproj -scheme Stead
```

### Code Coverage Targets

| Module | Current | Target (M3) | Target (M6) |
|--------|---------|-------------|-------------|
| `schema/contract` | ~90% | 90% | 95% |
| `storage/jsonl` | ~85% | 85% | 85% (deprecated) |
| `storage/sqlite` | N/A | 90% | 90% |
| `usf/schema` | ~80% | 85% | 85% |
| `usf/adapters/*` | ~40% | 60% | 70% |
| `commands/*` | ~70% | 75% | 80% |
| `cli` | ~90% | 90% | 90% |
| **Overall** | **~75%** | **80%** | **85%** |

USF adapters have lower coverage because they depend on filesystem state (installed CLIs). Fixture-based tests will improve this.

### Test Naming Convention

```
test_{module}_{behavior}_{condition}
```

Examples:
- `test_contract_creation_sets_pending_status`
- `test_storage_update_nonexistent_returns_not_found`
- `test_session_title_truncates_at_60_chars`

### Test Data / Fixtures

Create `tests/fixtures/` directory for synthetic session files:

```
rust/tests/fixtures/
  claude/
    simple-session.jsonl      # Basic user/assistant exchange
    tool-heavy-session.jsonl   # Many tool calls
    empty-session.jsonl        # No messages
  codex/
    simple-session.jsonl
  opencode/
    session.json               # Session metadata
    messages/                  # Message files
    parts/                     # Part files
```

Fixture files enable deterministic adapter testing without requiring CLIs to be installed.

## Quality Gate Checklist

Before any milestone is merged to `main`:

- [ ] All existing tests pass: `cargo test --workspace`
- [ ] No new warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Formatting clean: `cargo fmt --check`
- [ ] New code has tests (no untested public functions)
- [ ] Integration tests cover the happy path
- [ ] Error paths have at least one test
- [ ] No `unwrap()` in library code (use `?` or proper error handling)
- [ ] Breaking changes documented

### M2 Gate (Library Split)

- [ ] `cargo test --workspace` passes (all 114 tests)
- [ ] `stead list` still works from compiled binary
- [ ] `stead session list` still works
- [ ] No public API regression (same functions available)
- [ ] Workspace structure correct (two crates)

### M3 Gate (SQLite)

- [ ] Storage trait tests pass for both backends
- [ ] Migration test: JSONL -> SQLite preserves data
- [ ] Concurrent access test passes
- [ ] Performance: 1000 contracts list < 100ms

### M4 Gate (FFI)

- [ ] All FFI functions tested from Rust side
- [ ] Memory leak check (Instruments or similar)
- [ ] Swift can call every exposed function
- [ ] Error cases return proper Swift errors

### M5 Gate (SwiftUI)

- [ ] Snapshot tests for all views
- [ ] Dark/light mode snapshots
- [ ] Empty state and populated state
- [ ] App launches and shows data from stead-core

### M6 Gate (State Machine)

- [ ] Property-based tests pass (1000+ cases)
- [ ] All 10 states reachable
- [ ] All invalid transitions rejected
- [ ] Dependency resolution tested

## Dev Dependencies (Planned)

```toml
[dev-dependencies]
tempfile = "3"           # Existing - temp directories
assert_cmd = "2"         # Existing - CLI testing
predicates = "3"         # Existing - assertion helpers
proptest = "1"           # New (M6) - property-based testing
test-case = "3"          # New (M3) - parameterized tests
```

## Running Tests Locally

```bash
# All tests
cargo test --workspace

# Unit tests only (fast)
cargo test --lib --workspace

# Integration tests only
cargo test --test integration

# Specific module
cargo test --lib storage

# With output (see println! in tests)
cargo test --workspace -- --nocapture

# Coverage report (requires cargo-llvm-cov)
cargo llvm-cov --workspace --html
open target/llvm-cov/html/index.html
```
