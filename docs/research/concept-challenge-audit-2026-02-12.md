# Stead Concept Challenge Audit (Step-by-Step)

Date: 2026-02-12
Scope: whole repo (`rust/`, `macos/`, `docs/`)
Mode: concept challenge and docs/implementation alignment check (not deep code-style review)

## Step 1 - Define the authority baseline

I used `docs/plans/docs-authority-map.md` as the source for doc precedence:

1. Concept constraints: `docs/plans/canonical-decisions-2026-02-11.md`
2. Current shipped behavior: `README.md`, `rust/README.md`
3. Milestone sequencing: `docs/plans/phase2-revised.md`
4. Scope/UX direction: `docs/plans/mvp-scope.md`, `docs/plans/ux-refined.md`, `docs/plans/control-room-ux.md`

Why this matters: several docs conflict, so matching implementation must be judged against a stable hierarchy.

## Step 2 - Confirm what is working now

### Runtime and quality checks

- `cargo fmt --all --check` passes.
- `cargo clippy --workspace -- -D warnings` passes.
- `cargo test --workspace` passes with 114 tests (98 unit + 16 integration).

### Implemented core capabilities

- 10-state contract model exists in Rust (`Pending` .. `Cancelled`).
- CLI commands exist for `run`, `create`, `list`, `show`, `verify`, `claim`, `cancel`, `session list`, `session show`.
- SQLite is the default runtime storage path for CLI commands, with JSONL migration support.
- USF adapters exist for Claude, Codex, and OpenCode.

## Step 3 - Check docs vs implementation (direct mismatches)

### High-impact mismatches

1. FFI/macOS contract reads are JSONL, not SQLite
   - Canonical decision says SQLite is runtime truth for CLI + FFI + app.
   - `rust/stead-ffi/src/lib.rs` reads via `stead_core::storage::list_contracts` and `read_contract` (JSONL path).
   - Impact: CLI and app can diverge on contract data.

2. Documented macOS build command currently fails
   - `README.md` recommends `cd ../macos && ./build.sh`.
   - Repro: script fails at UniFFI step with `error running cargo metadata`.
   - Root cause: `macos/build.sh` runs bindgen from the wrong working directory for this setup.
   - Note: manual `xcodegen --quiet && xcodebuild ... build` in `macos/Stead` succeeds.

3. `stead run` can report success when agent execution failed
   - In `rust/stead-core/src/commands/run.rs`, Claude failure is captured as warning text but final status is still derived from verification command result.
   - Repro: running with `PATH` that hides `claude` still returned status `completed` when verify command was `true`.
   - Impact: contract truth can say "completed" even when no agent work ran.

4. Canonical CLI entry behavior is not implemented
   - Canonical decision requires bare `stead` to show status overview and support `stead status`.
   - Current behavior: bare `stead` prints help and exits non-zero; no `status` command exists.

5. Attention priority order differs between docs and app
   - `README.md` says: `Failed > Executing > Verifying > Claimed > Ready > Pending > Completed`.
   - `macos/Stead/Sources/Stead/Models/SteadStore.swift` currently prioritizes `Completed` above `Executing`.

6. macOS contract scope is project-local by cwd, not cross-project
   - `SteadStore.loadContracts()` calls FFI with `FileManager.default.currentDirectoryPath`.
   - This conflicts with "one view across all projects" framing and creates brittle behavior based on launch cwd.

### Active docs drift (lower impact but confusing)

7. `docs/plans/mvp-scope.md` still frames lifecycle as current 4-state and defers 10-state, but implementation is already 10-state.

8. `docs/plans/phase2-revised.md` and `docs/plans/test-strategy.md` still reference old test counts (88) and stale implementation checkpoints.

## Step 4 - Additional concept challenge (non-code-review)

This section challenges whether the implementation still protects the North Star loop: "ding -> restore context quickly with trustworthy state".

1. Truth split risk
   - If CLI truth is SQLite but app truth is JSONL, the supervision surface loses trust.
   - A control room that can disagree with execution history increases cognitive overhead instead of reducing it.

2. Verification-over-execution ambiguity
   - Current run semantics allow "verified" even if execution never happened.
   - For supervision, this is dangerous: "green" state no longer means "agent successfully executed task".

3. Context restoration quality risk
   - Session summaries in current environment were mostly `/unknown` with empty messages and fallback timestamps.
   - That weakens the primary value proposition: quickly restoring real project context.

4. Product model fragmentation
   - Docs simultaneously describe: strict canonical engine rules, pragmatic CLI behavior, and advanced control-room UX.
   - This fragmentation is now a bigger risk than raw implementation bugs.

## Step 5 - What is broken right now

- `macos/build.sh` (as documented in README) is broken in current setup.
- `stead run` can produce false-positive completion if `claude` is unavailable but verification passes.

## Step 6 - Suggested first alignment batch

1. Make FFI contract APIs use SQLite default backend (same path as CLI).
2. Fix `macos/build.sh` bindgen invocation context.
3. Decide and enforce `run` semantics: fail hard when agent execution fails (recommended).
4. Decide whether canonical CLI default (`stead` status overview + `stead status`) is in scope now; if not, mark decision as deferred.
5. Align one authoritative attention order across README + app.

## Step 7 - Questions for you (requested checkpoints)

1. Should I treat `docs/plans/canonical-decisions-2026-02-11.md` as strict implementation target now, or as near-term direction?
2. Do you want `stead run` to fail when Claude execution fails, even if verification command passes? (recommended: yes)
3. Should the macOS app be cross-project now, or explicitly project-local for current milestone?
4. Should I do a follow-up patch limited to the 3 critical mismatches first: FFI SQLite parity, build script, run semantics?
