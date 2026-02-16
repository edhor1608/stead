# Rewrite Parallel Track: M11 Named Localhost Broker

Date: 2026-02-16  
Status: Planned (parallel, non-blocking)  
Purpose: Define an execution-ready TDD plan for a Rust-native `portless`-style endpoint broker while M9 continues.

## 1) Scope

Build a reusable Rust module path for deterministic local endpoint allocation:

- Stable endpoint naming (`*.localhost`)
- Deterministic port assignment and conflict handling
- Typed daemon/CLI contracts
- Session-proxy integration point

This track is explicitly parallel to M9 and must not block M9 delivery.

## 2) Non-Goals (for this track)

- Internet tunnels / public URL exposure
- TLS certificate automation
- Cross-machine/global coordination
- Replacing current resource and module systems

## 3) Milestone Definition Standard (used by all M11 slices)

Each slice follows strict TDD:

1. Add failing tests first.
2. Add minimal implementation to pass.
3. Refactor without behavior change.
4. Verify:
   - targeted tests pass
   - relevant crate tests pass
   - `cargo test --workspace` passes
5. Publish checkpoint with:
   - failing tests added
   - code added
   - refactors
   - remaining risk + next slice

## 4) M11 Slices

### M11-S1: Endpoint Lease Domain

Objective:
- Define canonical endpoint lease types and rules.

Tests first:
- Claim new endpoint returns lease with name, owner, assigned port.
- Re-claim by same owner is idempotent.
- Release succeeds for owner and fails for non-owner with typed error.
- Lease export/import round-trip preserves state.

Exit criteria:
- Domain tests pass with deterministic behavior and typed errors.

### M11-S2: Deterministic Negotiation + Escalation

Objective:
- Handle name/port contention deterministically.

Tests first:
- Conflict resolves to lowest available next port within configured range.
- Exhausted range emits escalation event (`endpoint_range_exhausted`).
- Negotiation result ordering is deterministic.

Exit criteria:
- Negotiation + escalation semantics are fully test-locked.

### M11-S3: Daemon API Contract

Objective:
- Expose endpoint operations through versioned daemon envelope.

Tests first:
- Request/response envelope version + stable JSON shape.
- Typed errors for `not_found`, `not_owner`, `conflict`/`exhausted`.
- End-to-end claim/list/release via daemon handle API.

Proposed command family:
- `ClaimEndpoint`
- `ListEndpoints`
- `ReleaseEndpoint`

Exit criteria:
- API contract tests and daemon crate tests pass.

### M11-S4: CLI Contract

Objective:
- Add machine-stable CLI for endpoint flows.

Tests first:
- Claim/list/release user flows.
- `--json` output schema snapshots remain stable.
- Error paths produce non-zero exit code + typed JSON (when `--json`).

Exit criteria:
- CLI integration tests pass and workspace remains green.

### M11-S5: Session Proxy Integration

Objective:
- Wire endpoint broker into session proxy module behavior.

Tests first:
- Enabled session-proxy module yields deterministic project endpoint mapping.
- Disabled module falls back cleanly without core regressions.
- Identity/project boundaries remain enforced.

Exit criteria:
- Module SDK + daemon integration tests pass with no lifecycle regressions.

## 5) Risks and Guardrails

Primary risks:
- Accidental overlap with current M9 files/workstreams.
- API drift between daemon and CLI JSON contracts.
- Underspecified naming policy causing non-determinism.

Guardrails:
- Keep M11 changes in Rust crates + docs; avoid M9 UI file churn.
- Lock response schemas with tests before implementation details expand.
- Keep behavior deterministic first; optimization later.

## 6) Decision Gates (owner choice required before M11-S1 implementation)

### DG-1: CLI namespace for endpoints

1. `stead resource endpoint ...` (recommended): keeps all allocators in `resource`.
2. `stead endpoint ...`: cleaner discoverability, but adds a new top-level family.
3. Support both: flexible, but duplicates surface area.

### DG-2: URL format in first implementation

1. `http://<name>.localhost:<port>` only: simplest, no local proxy process.
2. `http://<name>.localhost` by default with reverse proxy: best UX, more moving parts.
3. Staged (recommended): ship `name+port` first, add no-port proxy as follow-up.

### DG-3: Crate boundary

1. Extend `stead-resources` directly: least moving parts.
2. New crate (recommended): `stead-endpoints` built on `stead-resources` for exportability.
3. Implement only in `stead-daemon`: fastest local path, weakest reuse.

### DG-4: Persistence scope

1. Workspace-local (`.stead/`) only (recommended): matches current architecture.
2. Machine-global endpoint table: better global uniqueness, higher complexity.
3. Dual scope (workspace default + optional global): flexible, but larger surface.
