# DX Review: stead CLI v0.2.0

**Date**: 2026-02-05
**Reviewer**: Product perspective (Steve Jobs role)
**Verdict**: Solid foundation with a handful of sharp edges to fix before v1

---

## Command Structure

```
stead
  run <task> --verify <cmd>   Create + execute + verify a contract
  list [--status <s>]         List contracts
  show <id>                   Show contract detail
  verify <id>                 Re-run verification
  session list [--cli] [--project] [--limit]
  session show <id> [--full]
```

**What works well:**
- Flat, predictable hierarchy (only `session` has sub-commands)
- Global `--json` flag for agent consumption — critical for the "stead is for agents" wedge
- `session list` groups by CLI type, shows relative time and project — exactly the "quick glance" the product vision describes

**Comparison to best-in-class CLIs:**

| Feature | stead | gh | cargo | turbo |
|---------|-------|----|-------|-------|
| Colored output | No | Yes | Yes | Yes |
| Shell completions | No | Yes | Yes | No |
| Alias support | No | Yes | No | No |
| `--help` quality | Basic | Excellent | Good | Good |
| Machine-readable output | `--json` | `--json` | `--message-format=json` | `--output-logs=json` |
| Exit codes | Partial | Full | Full | Full |

---

## Issues Found

### P0: Memory leak via `.leak()` in `session.rs:218`

```rust
.unwrap_or_else(|| format!("{:?}", call.tool).leak());
```

`String::leak()` converts a `String` into `&'static str` by leaking the allocation. For a CLI that exits quickly this is harmless, but once this code is shared with the SwiftUI app via FFI (which runs for hours), this becomes a real memory leak. The performance audit flagged this already.

**Fix**: Use `Cow<'_, str>` or just `String`.

### P1: Byte-slicing in `session.rs:293-294`

```rust
format!("{}...", &first_line[..max_len - 3])
```

This slices by byte index, not char boundary. A multi-byte UTF-8 character at the boundary will **panic**. The `list.rs:truncate()` function does this correctly with `.chars().take()` — the fix is to use the same pattern.

Contrast with the correct implementation at `list.rs:86-92`:
```rust
let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
```

### P1: Five copies of `truncate()`

Counted across the codebase:
1. `list.rs:85` — correct (chars-based)
2. `session.rs:288` — **buggy** (byte-based)
3. `usf/schema.rs:418` — buggy (byte-based)
4. Two more referenced in the performance audit

This violates DRY and ensures at least one copy will be wrong. Should be a single `util::truncate()` in stead-core.

### P2: `session list` shows "unknown" for most project paths

```
claude-19fc3bf8- │ unknown │ just now │ Session 19fc3bf8
```

Most Claude Code sessions show `unknown` as their project path and generate a fallback title like `Session 19fc3bf8`. This suggests the Claude adapter isn't parsing `project_path` from the session files correctly, or the session files lack this data. Either way, the user sees useless information — this is the **first thing someone will see** when they try stead.

### P2: IDs are unreadable

Contract IDs (`ml9uq2jl-1rp6h1f`) and session IDs (`claude-19fc3bf8-...`) are truncated in the table to 15-16 chars, making them impossible to use for `show` without copy-paste. Compare with `gh pr list` which shows human-readable `#123` numbers.

Options:
- Show a short alias (auto-incrementing number or first 6 chars)
- Support prefix matching (`stead show ml9u`)

### P2: No `init` command

`stead run` silently creates `.stead/` directory. There's no explicit `stead init` to set up a project. This means:
- No ceremony when adopting stead (good for quick starts)
- No opportunity to explain what stead is doing or configure it (bad for discoverability)
- Running any command in any directory creates a `.stead/` dir (could be surprising)

### P3: `run` command spawns `claude` unconditionally

```rust
fn spawn_claude(task: &str) -> Result<()> {
    let output = Command::new("claude")
        .args(["-p", task])
        ...
```

This hardcodes Claude Code as the only agent. The whole point of stead is multi-agent orchestration. This should be configurable or removed — the `run` command should be agent-agnostic (just create the contract, the agent is separate).

### P3: No color, no emoji status indicators

The output is plain text. No color for PASSED (green) / FAILED (red). No status indicators. Compare:
```
# Current
ml9uq2jl-1rp... passed test task                      2026-02-05 19:30

# Better (like gh pr list)
PASSED  ml9uq2  test task  5m ago
```

### P3: Error output goes to different places

- `show` with invalid ID: `Error: Contract not found: nonexistent` (anyhow error, stderr)
- `session list --cli invalid`: `Unknown CLI: invalid...` (eprintln, stderr) then exits 0
- `session show` with invalid ID: `Session not found:` (eprintln, stderr) then exits 0

Inconsistent: some errors return non-zero exit codes (via `bail!`), others print to stderr and return `Ok(())`. For agent consumption, every error should set a non-zero exit code.

---

## What's Good

1. **`--json` is global and consistent** — every command respects it. This is table stakes for agent integration and they got it right.

2. **Session discovery actually works** — Found 20 Claude Code sessions and Codex sessions. This is the product's wedge feature and it delivers.

3. **Contract lifecycle is complete** — create, run, verify, re-verify, list, filter. Simple and functional.

4. **Thin CLI wrapper** — `main.rs` is 46 lines. All logic lives in stead-core. This is the correct architecture for sharing with SwiftUI via FFI.

5. **Test coverage is solid** — Every command has unit tests with an in-memory SQLite backend. The test architecture (dependency injection via `execute_with_storage`) is clean.

6. **UTF-8 safe truncation exists** — They did it correctly in `list.rs`, just need to use it everywhere.

---

## Priority Fixes Before SwiftUI Integration

1. Fix `.leak()` memory bug (P0 — will cause real problems in long-running SwiftUI app)
2. Fix byte-slicing `truncate` in `session.rs` (P1 — will panic on non-ASCII)
3. Consolidate `truncate()` into one function (P1 — prevent future bugs)
4. Make error handling consistent (exit codes + JSON error format) (P2)
5. Fix Claude adapter to populate project_path correctly (P2)

---

## Not Urgent (Post-MVP)

- Add color output (use `colored` or `owo-colors` crate)
- Add shell completions (clap supports this natively)
- Add `stead init` command
- Make `run` agent-agnostic
- Support ID prefix matching for `show`/`verify`

---

## Addendum: Architecture Review (Linus Torvalds)

**Date:** 2026-02-05

### P1: clap/tokio/anyhow Do Not Belong in stead-core

`stead-core/Cargo.toml` depends on `clap`, `tokio`, and `anyhow`. These are CLI/binary concerns. The core library is supposed to be the shared brain consumed by CLI, FFI, and SwiftUI.

- `clap` -- CLI argument parsing lives in `stead-core/src/cli/mod.rs`. This entire module should move to `stead-cli`. The core library exposes functions, not argument structs.
- `tokio` -- No async code exists anywhere in stead-core. This is a phantom dependency adding 100+ transitive crates for nothing.
- `anyhow` -- Library code should use typed errors (`thiserror`), not erased errors. `anyhow` is for binaries. The FFI layer already uses `thiserror` correctly.

**Impact:** Every `cargo build -p stead-ffi` compiles clap and tokio for no reason. The SwiftUI app will link against argument parsing code it never calls.

### P1: `contract` Subcommand Grouping Needed Before M6

Current: `stead list`, `stead show`, `stead verify` are top-level.

M6 adds: `claim`, `unclaim`, `start`, `cancel`, `rollback`. That's 8 top-level contract commands. Namespace soup.

**Solution:** `stead contract list`, `stead contract show`, etc. Keep `stead run` as top-level convenience. This matches `gh pr list`, `gh issue list`, `cargo build` patterns.

Do this BEFORE M6 lands, not after. Changing command names after users exist is painful.

### P2: Hardcoded Version String

```rust
#[command(version = "0.2.0")]
```

Should be `#[command(version)]` which reads from `CARGO_PKG_VERSION` automatically. Hardcoded versions always drift.

### Note: FFI Layer is Clean

Reviewed `stead-ffi/src/lib.rs`. The UniFFI proc-macro approach (`#[derive(uniffi::Record)]`, `#[uniffi::export]`) is correct. Type conversions (DateTime -> String, usize -> u32) are handled properly. Both cdylib (2.6MB) and staticlib (108MB debug) build successfully.

No `build.rs` needed -- UniFFI 0.28 proc macros handle scaffolding via `uniffi::setup_scaffolding!()`.

All 88 workspace tests pass with the FFI crate in the workspace.
