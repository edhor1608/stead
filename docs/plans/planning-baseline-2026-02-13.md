# Planning Baseline (Aggregated)

Date: 2026-02-13
Status: Active
Purpose: Single entry point for current planning truth and conflict resolution.

## 1) Canonical Source Set

Use these docs in this order:

1. Concept constraints:
   - `docs/plans/canonical-decisions-2026-02-11.md`
2. Current shipped behavior:
   - `README.md`
   - `rust/README.md`
3. Execution sequencing:
   - `docs/plans/phase2-revised.md`
4. Scope and UX direction:
   - `docs/plans/mvp-scope.md`
   - `docs/plans/ux-refined.md`

If two docs conflict, higher order wins.

## 2) Locked Planning Decisions

### 2.1 Contract lifecycle model

- Canonical engine model is the 10-state lifecycle (`Pending` .. `Cancelled`).
- Human supervision views are projections of that engine truth.

Source:
- `docs/plans/canonical-decisions-2026-02-11.md`

### 2.2 Storage model

- SQLite is canonical runtime storage for all app surfaces.
- JSONL is compatibility tooling only (migration/import/export), not runtime truth.

Source:
- `docs/plans/canonical-decisions-2026-02-11.md`

### 2.3 MVP transport architecture

- Current MVP architecture is local monolith behavior: no HTTP API and no daemon requirement.
- CLI and FFI should call shared Rust core directly.

Source:
- `docs/plans/phase2-revised.md`
- Current implementation in `rust/stead-cli` and `rust/stead-core`

### 2.4 FFI bridge

- UniFFI is the active bridge choice for Rust-to-Swift bindings.

Source:
- `README.md`
- `rust/stead-ffi/*`
- `docs/plans/ux-refined.md`

### 2.5 Execution strategy

- stead orchestrates existing CLIs (Claude/Codex/OpenCode) as execution runtimes in MVP.
- Building a standalone execution daemon is deferred.

Source:
- `docs/plans/mvp-scope.md`
- `docs/plans/phase2-revised.md`

## 3) Open Planning Decisions (Not Locked Yet)

These are explicitly still open and should be decided one-by-one:

1. Contract semantics: should `stead run` be allowed to end `completed` if agent execution failed but verification passed?
2. State-machine strictness: current pragmatic command behavior vs stricter transition enforcement.
3. Control-room scope: project-local contract view vs cross-project contract view now.
4. CLI default interaction: canonical says bare `stead` should show status overview, but current CLI still defaults to help text.

Reference:
- `docs/plans/planning-first-roadmap-2026-02-11.md`

## 4) Superseded Legacy Decision Docs

The following documents are still useful for historical rationale but are not authoritative for current architecture:

1. `docs/plans/decisions/agent-sdk-language.md`
   - Superseded for transport model (HTTP/daemon assumptions are obsolete for current MVP architecture).
2. `docs/plans/decisions/contract-schema-format.md`
   - Superseded for storage model (JSONL-as-runtime assumptions are obsolete).
3. `docs/plans/decisions/first-slice.md`
   - Historical first-slice scope snapshot.

## 5) Change Protocol

When a planning decision changes:

1. Update this baseline and `docs/plans/canonical-decisions-2026-02-11.md` first.
2. Update `README.md` / `rust/README.md` next (if behavior claims changed).
3. Update sequencing/scope docs (`phase2-revised`, `mvp-scope`, `ux-refined`) after.
4. Mark stale decision docs as superseded instead of leaving competing active statements.
