# Stead Concept Audit: Docs vs Implementation

Date: 2026-02-11
Scope: Whole repo (`rust/`, `macos/`, `docs/`)
Style: Concept-level challenge (not a deep line-by-line bug review)

## Step 1: What I treated as source-of-truth

I split docs into three buckets before comparing implementation:

1. Current product-facing claims
- `README.md`
- `CLAUDE.md`
- `rust/README.md`

2. Active planning docs
- `docs/plans/mvp-scope.md` (**Status: Active**)
- `docs/plans/phase2-revised.md` (**Status: Active**)
- `docs/plans/test-strategy.md` (**Status: Active**)
- `docs/plans/design-language.md` (**Status: Active**)

3. Draft/proposal/vision docs
- `docs/plans/contract-state-machine.md` (**Draft**)
- `docs/plans/control-room-ux.md` (**Draft**)
- `docs/plans/swiftui-components.md` (**Draft**)
- `docs/plans/universal-session-format.md` (**Proposal**)
- `docs/plans/visionary-concepts.md` (**Provocations**)
- `docs/research/*` (research context, not binding requirements)

## Step 2: What works today (validated)

1. Rust checks are healthy
- `cargo fmt --all --check` passes
- `cargo clippy --workspace -- -D warnings` passes
- `cargo test --workspace` passes
- Test count matches docs: 114 tests (98 unit + 16 integration)

2. CLI core is functional
- `run/create/list/show/verify/claim/cancel/session list/session show` all implemented
- Global `--json` flag exists and works

3. Storage migration path exists
- SQLite backend with migration from JSONL is implemented
- Default command path uses SQLite (`open_default`)

4. USF adapters are present
- Claude, Codex, OpenCode adapters exist and are wired to session commands

## Step 3: Mismatches between docs and implementation

### A. Highest-impact mismatches

1. FFI/macOS app reads JSONL contracts, not SQLite
- Docs say SQLite is the shared source (`README.md:91`, `docs/plans/mvp-scope.md:24`)
- FFI reads JSONL APIs: `rust/stead-ffi/src/lib.rs:135-146`
- Result: CLI and macOS app can diverge on contract data

2. macOS build script currently breaks
- Running `macos/build.sh` fails at UniFFI step: `error running cargo metadata`
- Script call site: `macos/build.sh:14-19`
- Result: documented app build path is not currently reliable

3. `stead run` can report `completed` even if Claude execution fails
- `run` captures Claude error but still uses verification result to set final state: `rust/stead-core/src/commands/run.rs:47-79`
- Repro run with `claude` hidden from `PATH` returned exit 0 and status `"completed"` while output contained `"[Claude failed: ...]"`
- Concept issue: contract can be “successful” without agent execution

### B. Contract model mismatches

4. State machine implementation differs from the M6 spec doc
- Spec rejects `Executing -> Failed`, `Failed -> Ready`, `RollingBack -> Failed`: `docs/plans/contract-state-machine.md:92-98`
- Code allows them: `rust/stead-core/src/schema/contract.rs:43-47`

5. `verify` command bypasses lifecycle transitions
- `verify` directly calls `contract.complete(...)` without entering `Verifying`: `rust/stead-core/src/commands/verify.rs:42-47`
- This weakens the 10-state model semantics

6. Dependency guard from state-machine doc is not enforced
- Spec says empty `blocked_by` should start Ready: `docs/plans/contract-state-machine.md:107`
- `Contract::new` always starts Pending: `rust/stead-core/src/schema/contract.rs:145-159`

### C. Product UX mismatches

7. CLI default behavior differs from onboarding doc
- Doc says bare `stead` should show status overview and `stead status` alias should exist: `docs/plans/developer-onboarding.md:145-153`
- Actual CLI requires subcommand: `rust/stead-core/src/cli/mod.rs:15-16`

8. `stead list` is not attention-priority sorted
- Doc claim: attention-priority list (`docs/plans/mvp-scope.md:34`, `:127`)
- Implementation lists whatever storage returns (created-desc), then optional status filter: `rust/stead-core/src/commands/list.rs:27-33`

9. Control Room UX in docs is ahead of implementation
- Docs specify badge states, `circle.circle` icon, focus mode, keyboard workflow (`docs/plans/ux-refined.md`, `docs/plans/mvp-scope.md:45-72`)
- Current app uses static icon `square.stack.3d.up`: `macos/Stead/Sources/Stead/SteadApp.swift:8`
- Menu content is basic summary; no focus mode, no badge logic, no decision/anomaly UX: `macos/Stead/Sources/Stead/Views/MenuBarView.swift:24-85`
- Only keyboard shortcut in app is refresh (`Cmd+R`): `macos/Stead/Sources/Stead/Views/ContentView.swift:29`

10. App contract scope is current working directory only
- Store calls `listContracts(cwd: FileManager.default.currentDirectoryPath)`: `macos/Stead/Sources/Stead/Models/SteadStore.swift:169-172`
- This conflicts with “one view across all projects” framing in docs

### D. Internal doc consistency mismatches

11. Project docs disagree on lifecycle scope
- `CLAUDE.md:65` says M6 complete (10-state)
- `docs/plans/mvp-scope.md:22` says current lifecycle is 4-state and `:80` says 10-state is deferred
- This makes “matching docs” ambiguous

12. Documented attention order differs from app order
- README order: `Failed > Executing > Verifying > Claimed > Ready > Pending > Completed` (`README.md:73`)
- App order puts `Completed` above `Executing`: `macos/Stead/Sources/Stead/Models/SteadStore.swift:51-56`

## Step 4: Broken or risky behaviors

Broken now:
1. `macos/build.sh` fails during UniFFI generation (`error running cargo metadata`)
2. Contract can be marked completed even when Claude never ran

Risky now:
1. Session detail output leaks memory with `String::leak()`: `rust/stead-core/src/commands/session.rs:220`
2. Docs/implementation drift is large enough to confuse contributors on what “done” means

## Step 5: Concept-level challenge (strategic)

The core concept is strong: contracts + unified session visibility + control-room supervision.
The current risk is not technical feasibility. It is product-definition fragmentation.

Today there are effectively three products described at once:
1. A strict 10-state contract engine
2. A pragmatic CLI-first MVP with direct verification paths
3. A rich control-room UX with attention semantics and keyboard supervision

Implementation is between (1) and (2), while docs often describe (3). This gap is now the main drag on momentum.

## Step 6: Questions for you (needs your decision)

1. Which doc family should be authoritative for current behavior?
- Option A: `README.md` + `CLAUDE.md`
- Option B: `docs/plans/mvp-scope.md`
- Option C: `docs/plans/phase2-revised.md`

2. Should `stead run` fail hard if Claude execution fails (even when verification passes)?
- I recommend yes, if contracts are meant to represent real agent execution.

3. Do you want strict M6 state-machine enforcement now?
- If yes, we should align transitions + command behavior to `contract-state-machine.md`.

4. Should the macOS app be treated as project-local for now, or must it already be cross-project?
- Current implementation is effectively project-local for contracts.

5. Do you want me to do a follow-up alignment pass that only targets high-impact fixes first?
- Suggested first batch: FFI SQLite parity, `run` failure semantics, build script fix, docs authority cleanup.
