# Stead Phase 2: Revised Implementation Plan

**Created:** 2026-02-04
**Status:** Active

## Context

**Completed:**
- Rust CLI v2 with run/list/show/verify commands
- USF adapters for Claude Code, Codex CLI, OpenCode
- `stead session list` and `stead session show` commands
- 88 tests passing (72 lib + 16 integration)

**Architecture Decision (2026-02-04):**
- **Monolith** — no HTTP API, no separate server
- **Rust library + Native UIs** — stead-core as shared brain
- **SwiftUI for Mac** — native feel on primary platform
- **SQLite storage** — replaces JSONL for concurrent access

See: `decisions-log.md` for full rationale.

## Revised Architecture

```
┌─────────────────┐
│   macOS App     │  (SwiftUI - native Mac UI)
│  Control Room   │
└────────┬────────┘
         │ FFI (swift-bridge)
         │
┌────────▼────────┐
│   stead-core    │  (Rust library - the brain)
│                 │
│ • Contracts     │
│ • USF Adapters  │  ← done!
│ • SQLite        │
└────────┬────────┘
         │
┌────────▼────────┐
│   stead-cli     │  (Rust binary - uses stead-core)
└─────────────────┘
```

## Project Structure (Target)

```
stead/
├── rust/
│   ├── stead-core/           # Library (all logic)
│   │   ├── src/
│   │   │   ├── lib.rs        # Public API
│   │   │   ├── contracts/    # Contract engine
│   │   │   ├── usf/          # Session adapters (done!)
│   │   │   └── storage/      # SQLite
│   │   └── Cargo.toml
│   │
│   ├── stead-cli/            # CLI binary
│   │   ├── src/main.rs
│   │   └── Cargo.toml
│   │
│   └── stead-ffi/            # Swift bindings
│       ├── src/lib.rs
│       └── Cargo.toml
│
├── macos/                    # SwiftUI app
│   ├── Stead/
│   │   ├── SteadApp.swift
│   │   ├── Views/
│   │   │   ├── ContractListView.swift
│   │   │   ├── SessionListView.swift
│   │   │   └── ...
│   │   └── Models/
│   └── Stead.xcodeproj
│
└── docs/
```

---

## Milestones

### M1: USF Read Adapters ✅ COMPLETE

- [x] USF schema
- [x] Claude Code adapter
- [x] Codex CLI adapter
- [x] OpenCode adapter
- [x] `stead session list` command
- [x] `stead session show` command
- [x] Unit + integration tests

---

### M2: Restructure to Library + CLI (Foundation)

Split current monolithic CLI into library + binary. This enables FFI later.

| Task | Description |
|------|-------------|
| 2.1 | Create Cargo workspace with stead-core and stead-cli |
| 2.2 | Move all logic to stead-core/src/ |
| 2.3 | stead-cli becomes thin wrapper calling stead-core |
| 2.4 | Define clean public API in lib.rs |
| 2.5 | All tests still pass |

**Files:**
```
rust/
├── Cargo.toml              # Workspace manifest
├── stead-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # pub fn list_contracts(), etc.
│       ├── contracts/
│       ├── usf/
│       └── storage/
└── stead-cli/
    ├── Cargo.toml
    └── src/main.rs         # Calls stead_core::*
```

**Verification:**
```bash
cargo test --workspace
stead list  # Still works
stead session list  # Still works
```

---

### M3: SQLite Storage

Replace JSONL with SQLite for concurrent access.

| Task | Description |
|------|-------------|
| 3.1 | Add rusqlite dependency |
| 3.2 | Define schema (contracts table, sessions cache) |
| 3.3 | Implement Storage trait with SQLite backend |
| 3.4 | Migration: JSONL → SQLite on first run |
| 3.5 | Update all commands to use SQLite |
| 3.6 | Tests pass |

**Schema:**
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

CREATE INDEX idx_contracts_status ON contracts(status);
CREATE INDEX idx_contracts_project ON contracts(project_path);
```

**Verification:**
```bash
cargo test --workspace
stead run "test" --verify "true"
sqlite3 .stead/stead.db "SELECT * FROM contracts"
```

---

### M4: Swift FFI Bindings

Enable Swift to call stead-core.

| Task | Description |
|------|-------------|
| 4.1 | Add stead-ffi crate |
| 4.2 | Set up swift-bridge |
| 4.3 | Expose core functions: list_contracts, list_sessions, etc. |
| 4.4 | Generate Swift bindings |
| 4.5 | Test from Swift playground |

**Example FFI (stead-ffi/src/lib.rs):**
```rust
#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        type ContractSummary;
        fn list_contracts() -> Vec<ContractSummary>;
        fn get_contract(id: String) -> Option<Contract>;

        type SessionSummary;
        fn list_sessions() -> Vec<SessionSummary>;
    }
}
```

**Verification:**
```swift
import Stead
let contracts = list_contracts()
print(contracts.count)
```

---

### M5: SwiftUI Control Room MVP

Native Mac app with attention-prioritized view.

| Task | Description |
|------|-------------|
| 5.1 | Create Xcode project |
| 5.2 | Integrate stead-ffi as Swift package |
| 5.3 | Contract list view (grouped by status) |
| 5.4 | Session list view |
| 5.5 | Attention priority ordering |
| 5.6 | System tray / menu bar presence |
| 5.7 | Basic styling |

**Attention Priority Order:**
1. 🔴 Needs Decision (blocked on human)
2. 🟡 Anomalies (unexpected state)
3. 🟢 Completed (awaiting review)
4. ⚪ Running (active)
5. ⬚ Queued (waiting)

**Verification:**
- App launches
- Shows contracts from all projects
- Shows sessions from Claude/Codex/OpenCode
- Grouped by attention priority

---

### M6: Full Contract Lifecycle

Expand from 4 states to 10 states.

| Task | Description |
|------|-------------|
| 6.1 | Expand ContractStatus enum |
| 6.2 | Add owner, blockedBy, blocks fields |
| 6.3 | State transition guards |
| 6.4 | CLI: claim, unclaim, start, complete, fail commands |
| 6.5 | Update SwiftUI views for new states |

**10 States:**
```rust
enum ContractStatus {
    Pending,     // Waiting for dependencies
    Ready,       // Can be claimed
    Claimed,     // Agent owns it
    Executing,   // Work in progress
    Verifying,   // Running verification
    Completed,   // Success
    Failed,      // Failed
    RollingBack, // Rollback in progress
    RolledBack,  // Rollback done
    Cancelled,   // Manually cancelled
}
```

---

## Execution Order

```
M2 (Library Split)
    ↓
M3 (SQLite)
    ↓
M4 (FFI)
    ↓
M5 (SwiftUI MVP)
    ↓
M6 (Full Lifecycle)
```

Each milestone is independently useful. M2-M3 improve the CLI. M4-M5 add the UI. M6 adds power features.

---

## Deferred

- Windows/Linux native apps (use stead-core + their native UI later)
- Tauri fallback (web UI for non-Mac if needed)
- Session Proxy (browser isolation)
- Execution Daemon (we orchestrate existing CLIs instead)
- Transformation Layer (git abstraction)

---

## Key Principles

1. **Library first** — all logic in stead-core, UIs are views
2. **Native feel** — SwiftUI for Mac, not web-in-a-box
3. **No server** — monolith, direct library calls
4. **SQLite** — safe concurrent access, queryable
5. **Incremental** — each milestone is useful alone

---

## Supersedes

This plan replaces the original Phase 2 plan which proposed:
- HTTP API layer with axum
- Tauri for Control Room
- Server + CLI as separate processes

Those approaches were reconsidered in favor of the simpler monolith architecture.
