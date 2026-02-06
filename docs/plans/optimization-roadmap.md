# Optimization Roadmap: stead CLI

**Date:** 2026-02-05
**Synthesized from:** Architecture Review (Linus), Performance Audit (ThePrimeagen), FFI Comparison, UX Design, Agent Orchestration Research, Test Strategy, Visionary Concepts

---

## Research Synthesis

### Architecture Review (Linus) -- Verdict: REJECT with fixable issues

**What needs fixing:**
1. **`run_verification()` duplicated** in `commands/run.rs` and `commands/verify.rs` -- identical 30-line functions
2. **`truncate()` duplicated 5+ times** -- 1 correct (UTF-8 safe in `list.rs`), 4 buggy (byte-slicing in `session.rs`, `claude.rs`, `codex.rs`, `opencode.rs`) that will **panic on multi-byte UTF-8**
3. **`generate_id()` duplicated** -- `schema/contract.rs` (good, has randomness) vs `usf/schema.rs` (no randomness, millisecond collisions possible)
4. **Memory leak** -- `session.rs:218` calls `.leak()` on a `format!()` string. Leaks heap memory per tool call displayed
5. **Library API doesn't exist** -- `stead-core` still exports `cli` and `commands` modules that print to stdout. Commands mix I/O with logic. M2 split happened structurally but the API boundary is still wrong
6. **`clap` in stead-core** -- CLI parsing dependency has no business in the library crate
7. **tokio dead weight** -- full async runtime compiled, zero async code

**What's good (keep as-is):**
- Module structure: clean, no circular deps
- Contract type: solid state machine
- Storage layer: proper error types, graceful corruption handling
- USF schema + adapter trait: well-designed
- Error handling pattern (thiserror for lib, anyhow for commands): correct
- Test coverage: 88 tests, good edge cases

### Performance Audit (ThePrimeagen) -- Three priorities

**Priority 1 -- Do now:**
- Remove tokio (~200KB binary reduction, 1 minute)
- Fix `.leak()` memory bug (5 minutes)
- Short-circuit adapter filtering in `discover_all_sessions()` -- currently constructs ALL adapters even with `--cli claude` filter (10 minutes)

**Priority 2 -- M3 timeframe:**
- Fix UTF-8 truncation bug and consolidate to one function
- Extract shared `run_verification()`
- Reduce unnecessary `.clone()` calls in adapters (3x `id.clone()` when 1 suffices, deep-copying `serde_json::Value`)

**Priority 3 -- Architecture (SQLite migration):**
- JSONL is O(n) reads, O(n) writes, full rewrite on update, no concurrent safety
- SQLite fixes all of these (already planned for M3)
- Session summary caching to avoid re-parsing adapter files
- Lazy adapter loading (only init adapters matching CLI filter)

**Binary size:** Currently 1.2MB. Removing tokio -> ~900KB-1MB. `panic = "abort"` saves another 10-20KB.

### FFI Comparison -- UniFFI recommended

**Performance implications:**
- UniFFI has slightly more overhead than swift-bridge (serialization vs direct layout) but **irrelevant for stead's use case** -- infrequent calls (list contracts, get session), not high-frequency
- The real perf concern is what data crosses the FFI boundary. Don't send full `UniversalSession` with 1000 timeline entries when a summary suffices
- DateTime handled as ISO 8601 strings (no native chrono bridging) -- fine, parsing cost is negligible

### UX Design -- Performance-critical features

**Must be fast (<100ms):**
- Menu bar icon state calculation (aggregate contract status)
- Popover appearance (only "needs attention" items)
- Contract list with attention priority ordering

**Must be responsive (<16ms per frame):**
- State transition animations (300ms spring animations)
- List scrolling with contract rows (36pt compact density)
- Section collapse/expand

**Performance requirements from UX:**
- "App launch: skeleton UI for ~200ms while SQLite loads" -- SQLite must return contract list in <200ms
- "Session discovery: progressive, show results as each adapter completes" -- adapters must not block each other
- "New contract creation: optimistic, appears immediately" -- write must be non-blocking from UI perspective
- Real-time updates via filesystem watching (kqueue/FSEvents) -- must not poll

### Agent Orchestration Research -- Positioning implications

No perf implications directly, but confirms:
- Cross-CLI session visibility is the wedge. `session list` must be FAST
- Contract engine is the differentiator. Contract CRUD must be O(1) with SQLite
- Local-first architecture means no network latency excuse -- everything must feel instant

---

## Quick Wins for M3 (Apply Now)

These are changes that take minutes to hours and have immediate impact.

| # | Change | Impact | Effort | File(s) |
|---|--------|--------|--------|---------|
| 1 | **Remove tokio from Cargo.toml** | ~200KB binary, faster compile | 1 min | `stead-core/Cargo.toml` |
| 2 | **Fix `.leak()` in session.rs:218** | Memory correctness | 5 min | `commands/session.rs` |
| 3 | **Short-circuit adapter filtering** | Faster `session list --cli X` | 15 min | `commands/session.rs`, `usf/adapters/mod.rs` |
| 4 | **Fix UTF-8 truncation bug** | Prevent panic on multi-byte chars | 15 min | `commands/session.rs`, adapter files |
| 5 | **Extract shared `run_verification()`** | DRY, prevent future divergence | 10 min | `commands/run.rs`, `commands/verify.rs` |
| 6 | **Move `clap` to stead-cli only** | Clean library API surface | 30 min | Both Cargo.toml files, restructure cli module |
| 7 | **`panic = "abort"` in release profile** | ~10-20KB binary saved | 1 min | `rust/Cargo.toml` |

### Fix details:

**#2 -- `.leak()` fix:**
```rust
// Before (leaks memory):
.unwrap_or_else(|| format!("{:?}", call.tool).leak());

// After (owned string, use Display trait):
let tool_name = call.original_tool.as_deref()
    .unwrap_or(&format!("{}", call.tool));
```

**#3 -- Short-circuit adapters:**
```rust
// In discover_all_sessions(), accept optional CliType filter:
pub fn discover_all_sessions(cli_filter: Option<CliType>) -> Vec<SessionSummary> {
    let mut sessions = Vec::new();
    if cli_filter.is_none() || cli_filter == Some(CliType::Claude) {
        // only then construct and query the adapter
    }
    // ...
}
```

**#4 -- UTF-8 truncation consolidation:**
One correct implementation using `char_indices()` (or the existing `.chars()` version in `list.rs`), placed in a shared utils module or at the crate root. Remove all 5 duplicates.

---

## Medium-Term Optimizations (M5/M6)

These require more design work and align with upcoming milestones.

### SQLite Performance (M3 foundation, tuned for M5)

| Change | Why | Notes |
|--------|-----|-------|
| WAL mode enabled at DB creation | Concurrent reads during writes (UI + CLI) | `PRAGMA journal_mode=WAL` |
| Prepared statements cached | Avoid re-parsing SQL on each call | `rusqlite::CachedStatement` |
| Index on `status` column | Fast attention-priority grouping | `CREATE INDEX idx_status ON contracts(status)` |
| Index on `project_path` | Fast project filtering | `CREATE INDEX idx_project ON contracts(project_path)` |
| Batch reads for contract list | Single query with ORDER BY vs sort in Rust | Let SQLite do the sorting |
| Session summary cache table | Avoid re-parsing adapter files on every `session list` | Invalidate on file mtime change |

**Target:** `stead list` < 10ms, `stead session list` < 100ms (down from 220ms).

### Adapter I/O Reduction

| Change | Current Cost | Improved Cost |
|--------|-------------|---------------|
| Claude: read 5-10 lines instead of 100 for summary | 50 sessions x 100 lines = 5000 lines | 50 x 10 = 500 lines |
| OpenCode: cache project info per session dir | 3+ fs ops per session | 1 fs op + cache lookup |
| Lazy adapter construction | All 3 adapters always constructed | Only construct matching adapter |
| Progressive discovery | Block until all adapters finish | Return results as each completes |

### Clone Reduction in Adapters

The Claude adapter clones `id` 3 times per tool call when 1 suffices. The `input: serde_json::Value` clone deep-copies the JSON tree. For long sessions (1000+ messages), this means thousands of unnecessary heap allocations.

Fix: clone for HashMap key, move original. Use references where possible. Consider `Rc<str>` for repeated IDs if profiling shows it matters.

### FFI Boundary Optimization (M4)

- Bridge summary types, not full sessions. `ContractSummary` with 6 flat fields, not `UniversalSession` with nested Vecs
- Lazy load session detail -- full timeline only when user expands in UI
- Batch contract list into single FFI call, not one-per-contract
- Consider `uniffi::Object` for stateful `SteadBridge` that caches results

### Library API Cleanup (ongoing from M2)

Commands should return data, not print. Current:
```rust
pub fn execute(status_filter: Option<&str>, json_output: bool) -> Result<()>
// prints to stdout, returns nothing
```

Should be:
```rust
pub fn list_contracts(status: Option<ContractStatus>) -> Result<Vec<Contract>>
// returns data, caller decides how to display
```

This is required for FFI (M4) and SwiftUI (M5) to work without reimplementing all logic.

---

## Long-Term Performance Architecture

### For the Control Room (M5+)

**State aggregation must be O(1):**
The menu bar icon needs instant state: "anything need me?" This should be a single SQLite query:
```sql
SELECT status, COUNT(*) FROM contracts
WHERE status IN ('Failed', 'Completed', 'Running')
GROUP BY status
```
Pre-compute and cache. Update on contract state change via trigger or notification.

**Filesystem watching strategy:**
- Use `kqueue` (macOS) via `notify` crate for `.stead/` directory changes
- FSEvents for broader session directory watching
- Debounce: batch filesystem events, process at most every 500ms
- Don't re-scan entire adapter trees on every change -- detect which file changed and update incrementally

**Animation performance:**
- SwiftUI's declarative rendering handles most of this
- Ensure contract list uses `LazyVStack` not `VStack` for large lists
- State diff calculation must happen off-main-thread if >100 contracts

### For Scale (M6+)

**Contract dependency resolution:**
The 10-state machine with `blockedBy`/`blocks` creates a DAG. Resolving "what's ready?" must be efficient:
- Maintain a ready queue in SQLite (trigger-updated)
- Don't walk the entire graph on every status query
- Index: `CREATE INDEX idx_blocked ON contract_deps(blocked_id)`

**Session memory/caching:**
- LRU cache for recently accessed sessions (avoid re-parsing)
- Lazy timeline loading -- summary first, timeline on demand
- Consider memory-mapped JSONL for Claude sessions (read without copying)

### Compile-Time Optimizations

| Setting | Current | Recommended | Impact |
|---------|---------|-------------|--------|
| `lto` | `true` | `true` | Keep (good for binary size) |
| `codegen-units` | `1` | `1` | Keep (better optimization, slower compile) |
| `strip` | `true` | `true` | Keep (smaller binary) |
| `panic` | default (unwind) | `"abort"` | ~10-20KB saved, faster panics |
| `opt-level` | default (`3`) | `3` for speed, `"z"` for size | Only switch to `"z"` if binary size is critical |

**Dev compile time:** With tokio removed and the workspace split, incremental builds of `stead-cli` should be <2s. Full rebuild ~15s on M2 MacBook Air.

---

## Priority Order

```
NOW (M3 prep):
  1. Remove tokio                    [1 min]
  2. Fix .leak() memory bug          [5 min]
  3. Fix UTF-8 truncation + consolidate [15 min]
  4. Extract shared run_verification [10 min]
  5. Short-circuit adapter filtering [15 min]
  6. panic = "abort" in release      [1 min]
  7. Move clap out of stead-core     [30 min]

M3 (SQLite):
  8. SQLite with WAL mode + indexes
  9. Session summary cache table
  10. Storage trait (enables testing both backends)

M4 (FFI):
  11. Library API returns data, not prints
  12. Summary types for FFI boundary
  13. Lazy session detail loading

M5 (SwiftUI):
  14. O(1) state aggregation for menu bar
  15. Filesystem watching with debounce
  16. LazyVStack for contract lists

M6 (Full lifecycle):
  17. Ready queue for dependency resolution
  18. LRU session cache
  19. Property-based testing for state machine
```

---

## Codebase Issues Found (Beyond tokio)

Scanning the actual code after M2 split, these issues exist:

1. **`clap` in stead-core** (`stead-core/Cargo.toml:11`) -- CLI parsing dependency in the library. The `cli/mod.rs` module with Clap derive types should move to `stead-cli`.

2. **Comment lie** (`stead-core/Cargo.toml:24`) -- "for future HTTP API" but the architecture decision explicitly says NO HTTP API. The comment justifies dead code.

3. **`discover_all_sessions()` swallows errors** (`usf/adapters/mod.rs:55-57`) -- `if let Ok(...)` silently drops adapter failures. A Claude adapter that fails due to permissions gives no feedback. Should at minimum log to stderr.

4. **`show_session` swallows errors** (`commands/session.rs:62-63`) -- `Err(_) => { eprintln!(...) }` then returns `Ok(())`. Callers can't distinguish "session found" from "session not found" from "I/O error".

5. **No `BufWriter`** (`storage/jsonl.rs:64-81`) -- writes use unbuffered `File`. For single-line appends this is fine, but `rewrite_contracts()` writes N lines without buffering.

6. **`read_contract` is O(n)** (`storage/jsonl.rs:132-134`) -- reads ALL contracts, sorts them, then does linear search. For `show <id>`, you pay the full cost of `list`.

All of these get resolved by the SQLite migration (M3) and library API cleanup, so they're not separate work items -- they're part of the plan.
