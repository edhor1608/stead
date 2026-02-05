# Performance Audit: stead CLI v0.2.0

**Date**: 2026-02-05
**Auditor**: ThePrimeagen (simulated)
**Codebase**: `rust/src/` - 19 files, ~2700 LOC

---

## 1. Unnecessary Dependencies

### tokio: DEAD WEIGHT. DROP IT.

```toml
tokio = { version = "1", features = ["full"] }
```

**Zero usage.** Grepped the entire `src/` tree for `tokio`, `async`, `.await`, `#[tokio`. Nothing. Not a single async function in the codebase. The comment says "for future HTTP API" but the architecture decision explicitly says NO HTTP API.

**Impact**: tokio with `features = ["full"]` pulls in:
- tokio itself (async runtime, I/O reactor, thread pool, timers, signal handling)
- bytes, mio, parking_lot, socket2, signal-hook-registry, tokio-macros
- Transitive proc-macro deps: syn, quote, proc-macro2

This is **the single largest dependency** in the project. Every other dep is doing real work. tokio is doing nothing.

**Action**: Remove it. One line in Cargo.toml. Binary will likely shrink 100-300KB.

### chrono: Legitimate but heavy

chrono is used everywhere (timestamps, formatting, relative time). It's doing real work. However, `chrono` with `serde` feature pulls in a decent amount of code. No action needed now, but if binary size becomes critical, `time` crate is lighter.

### Everything else: Justified

- clap (derive): CLI parsing, no way around it
- serde + serde_json: JSON parsing is the core workload
- dirs: Home directory expansion, tiny
- thiserror + anyhow: Error handling, minimal overhead

---

## 2. Allocation Audit

### CRITICAL: Memory leak via `.leak()`

**File**: `commands/session.rs:218`

```rust
.unwrap_or_else(|| format!("{:?}", call.tool).leak());
```

This **leaks memory on every call**. `String::leak()` converts a `String` into a `&'static str` by intentionally leaking the heap allocation. It will never be freed. Called once per tool call in a session timeline display. If a session has 500 tool calls, that's 500 leaked strings.

In practice, the process exits right after, so it's not a runtime problem. But it's a code smell that screams "I didn't know how to satisfy the borrow checker." The fix is to own the string instead of leaking it.

### Duplicated `run_verification` function

**Files**: `commands/run.rs:98` and `commands/verify.rs:60`

Identical function, copy-pasted. 30 lines of duplicated code. Should be extracted to a shared module. Not a performance issue, but it's technical debt that leads to bugs when one copy gets updated and the other doesn't.

### Duplicated `truncate` function - FIVE COPIES

Found in:
- `commands/list.rs:73` (UTF-8 safe, uses `.chars()`)
- `commands/session.rs:288` (byte-level, NOT UTF-8 safe)
- `usf/schema.rs:418` (byte-level, NOT UTF-8 safe)
- `usf/adapters/claude.rs:416` (byte-level, NOT UTF-8 safe)
- `usf/adapters/codex.rs:473` (byte-level, NOT UTF-8 safe)
- `usf/adapters/opencode.rs:407` (byte-level, NOT UTF-8 safe)

Six implementations. Two different behaviors. The one in `list.rs` is correct (uses `.chars().count()` for UTF-8 safety). The other five slice at byte boundaries (`&first_line[..max_len - 3]`), which will **panic on multi-byte UTF-8 characters**. This is a latent bug, not just a perf issue.

### Unnecessary clones in adapters

The USF adapters (claude.rs, codex.rs, opencode.rs) are clone-heavy. Example from `claude.rs`:

```rust
// Lines 155-160: Three clones of the same id for one tool call
let tool_call_id = id.clone();
pending_tool_calls.insert(
    id.clone(),
    (id.clone(), tool, input.clone()),
);
```

That's `id` cloned 3 times when 1 would suffice (clone for the HashMap key, move the original into the timeline entry). The `input: serde_json::Value` clone is also heavy - it deep-copies the entire JSON tree.

Similar patterns throughout:
- `entry.uuid.clone().unwrap_or_default()` called multiple times per loop iteration in claude.rs (lines 108, 124, 147)
- `result_content.clone()` called twice (lines 128, 130) when once + a reference would do
- `text.clone()` on every content item in every message

These clones aren't individually expensive for small session files, but for a Claude session with thousands of messages (common for long coding sessions), you're doing thousands of unnecessary heap allocations.

### `truncate` returns `String` when `Cow<str>` would avoid allocation

Every `truncate` call allocates a new String even when the input is already short enough. Returning `Cow<'_, str>` would avoid allocation in the common case (string already fits).

### `format!` allocations in display code

43 `format!()` calls across the codebase. Most are in display/print code, so they're unavoidable. But some could be eliminated:

```rust
// session.rs:131 - format for every tool call display
format!("{:?}", call.tool)
```

This should use the `Display` impl that already exists on `UniversalTool`.

---

## 3. Startup Time

### `stead list`: ~0ms (fast)

```
0.00s user 0.00s system
```

Reads `.stead/contracts.jsonl` (or finds it doesn't exist) and exits. This is as fast as it can be.

### `stead session list`: ~220ms (concerning)

```
0.19s user 0.03s system 99% cpu 0.228 total
```

For a CLI that should feel instant, 220ms is noticeable. Here's what's happening:

1. **Constructs all three adapters** (claude, codex, opencode) - checks if dirs exist
2. **Claude adapter**: Walks `~/.claude/projects/`, enters every project subdirectory, finds every `.jsonl` file, opens and parses the first 100+ lines of each to build summaries
3. **Codex adapter**: Recursively walks `~/.codex/sessions/` year/month/day directories
4. **OpenCode adapter**: Walks `~/.local/share/opencode/storage/session/`, loads JSON files, then for EACH session loads project info AND walks messages dir AND counts files

The OpenCode adapter is the worst offender. `build_session_summary()` (line 285) does:
- Load project info (disk I/O for each session)
- If no title, walk the messages directory, load each message, load its parts (!), find first user text
- Count messages by `read_dir().count()` (another dir walk)

That's potentially 3+ directory reads and multiple JSON parses PER SESSION just for a summary.

### File I/O patterns

**JSONL is read-scan-everything on every operation.** `read_contract()` calls `list_contracts()` which reads the ENTIRE file, deserializes EVERY line, sorts them, then does `find()`. For `update_contract()`, it reads everything, finds the contract, then REWRITES THE ENTIRE FILE.

With 100 contracts, that's:
- `list`: Parse 100 JSON lines, sort, display
- `show`: Parse 100 JSON lines, sort, linear search
- `update`: Parse 100 JSON lines, find, serialize 100 lines, write

This is O(n) for reads and O(n) for writes. It works fine for <1000 contracts but will degrade.

---

## 4. Binary Size

**Current**: 1.2 MB (release, stripped, LTO)

This is actually decent for a Rust CLI with clap + serde + chrono + tokio. But remember, tokio is dead weight.

**Estimated breakdown** (approximate):
- clap (derive + parsing): ~300-400KB
- serde + serde_json: ~200-250KB
- tokio (full, unused): ~200-300KB
- chrono: ~100-150KB
- Application code: ~50-100KB
- Standard library: ~100-150KB

**Removing tokio should get you to ~900KB-1MB.**

Further options:
- `opt-level = "z"` instead of default (optimize for size vs speed) - saves ~5-10%
- `panic = "abort"` - saves ~10-20KB by removing unwind tables

---

## 5. Architecture Performance

### JSONL will not scale (but you know this)

The current JSONL storage has these problems:
1. **Full scan on every read** - no index, no seeking
2. **Full rewrite on every update** - not append-only despite the name
3. **No concurrent access safety** - two `stead` processes writing simultaneously = corruption
4. **Sort on every list** - deserialize everything, sort in memory

SQLite (planned M3) fixes all of these. Good call.

### USF adapters are doing too much work on `session list`

`discover_all_sessions()` in `adapters/mod.rs`:
1. Creates all three adapters
2. Each adapter walks its entire directory tree
3. Each adapter parses files to build summaries
4. Results are combined and re-sorted

For `session list --cli claude`, it still constructs and checks codex and opencode adapters. The filter happens AFTER all the I/O in `session.rs:19-28`. This should short-circuit.

### The Claude adapter reads 100+ lines per session for a summary

`parse_session_summary()` reads up to 100 lines of each session file. Claude Code sessions can have thousands of lines. The 100-line cap helps, but for a directory with 50 sessions, that's 50 file opens + 5000 lines parsed just to show a list.

Better approach: Cache summaries, or read only the first 5-10 lines (session metadata is typically in the first entry).

### OpenCode adapter is the perf worst case

`build_session_summary()` does multiple filesystem operations per session:
1. `load_project_info()` - opens and parses a JSON file
2. Title extraction - walks messages dir, loads message files, loads part files
3. Message counting - `read_dir().count()`

For 20 OpenCode sessions, that's potentially 60+ filesystem operations.

---

## 6. Recommendations

### Priority 1: Quick Wins (do now, M2)

| Change | Impact | Effort |
|--------|--------|--------|
| **Remove tokio** | ~200KB binary reduction, fewer deps | 1 min |
| **Fix `.leak()` in session.rs** | Memory correctness | 5 min |
| **Short-circuit adapter filtering** | Faster `session list --cli X` | 10 min |

### Priority 2: Code Quality (M2-M3)

| Change | Impact | Effort |
|--------|--------|--------|
| **Extract shared `truncate`** | Fix UTF-8 panic bug, DRY | 15 min |
| **Extract shared `run_verification`** | DRY | 10 min |
| **Reduce clones in adapters** | Fewer allocations on large sessions | 30 min |

### Priority 3: Architecture (M3+, with SQLite)

| Change | Impact | Effort |
|--------|--------|--------|
| **SQLite for contracts** | O(1) reads, concurrent access, indexing | Already planned |
| **Cache session summaries** | Fast `session list` on repeat | Medium |
| **Lazy adapter loading** | Only init adapters that pass filter | Small |
| **Read fewer lines for summary** | Reduce I/O per session file | Small |

### Priority 4: Nice to Have (later)

| Change | Impact | Effort |
|--------|--------|--------|
| `panic = "abort"` in release | ~10-20KB saved | 1 min |
| `opt-level = "z"` | ~5-10% binary reduction (trades speed) | 1 min |
| `Cow<str>` returns from truncate | Avoid allocations in common case | 15 min |

---

## Summary

The codebase is clean and well-structured for a v0.2.0. The biggest problem is **tokio sitting there doing nothing** - rip it out. The `.leak()` call is a correctness bug that should be fixed immediately. The USF adapter I/O patterns are the real performance concern for `session list`, but SQLite + caching in M3 should address that.

The clone-heavy adapter code is not a bottleneck today (sessions are parsed once then thrown away), but it would matter if sessions were held in memory or parsed repeatedly (e.g., a GUI refreshing a list).

**TL;DR**: Remove tokio, fix the leak, short-circuit adapter filtering. Everything else is fine for now and gets solved by the SQLite migration.
