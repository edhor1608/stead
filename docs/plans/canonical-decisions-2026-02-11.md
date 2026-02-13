# Canonical Decisions

Date: 2026-02-11
Status: Accepted (planning baseline)
Purpose: Lock the concept-level foundation before further implementation challenges.

## Decision 1: Contract Lifecycle Model

### Decision

Use a **10-state engine lifecycle** as canonical system truth, and expose a **collapsed supervision view** for humans.

- Canonical engine states:
  - `Pending`
  - `Ready`
  - `Claimed`
  - `Executing`
  - `Verifying`
  - `Completed`
  - `Failed`
  - `RollingBack`
  - `RolledBack`
  - `Cancelled`
- Human supervision view is a projection (attention tiers), not a replacement of engine truth.

### Why

1. Agents and automation need precise state semantics.
2. Humans need fast attention-oriented summaries during interruptions.
3. One model can satisfy both by separating machine truth from human projection.

### Accepted Alternative

- **Accepted:** Dual-layer model (engine truth + supervision projection).

### Rejected Alternatives

1. **Rejected:** 4-state canonical model.
   - Reason: insufficient for ownership, verification, rollback, and dependency semantics.
2. **Rejected:** Expose 10 states directly as primary human model.
   - Reason: adds cognitive load and harms supervision speed.

### Implementation Direction

1. Keep 10-state transitions as canonical behavior.
2. Define explicit mapping rules from engine state to attention tier.
3. Document both layers in architecture and UX docs.

---

## Decision 2: Canonical Storage Model

### Decision

Use **SQLite as canonical storage** for all runtime surfaces (CLI, FFI, macOS app), with **append-only event history inside SQLite**.

- SQLite is the single source of truth.
- JSONL is compatibility tooling only (migration, import/export), not runtime truth.

### Why

1. Concurrent CLI + app access needs transactional safety.
2. Control-room and reporting need queryable state.
3. Event history preserves auditability without split-brain storage.

### Accepted Alternative

- **Accepted:** SQLite snapshot tables + append-only event tables.

### Rejected Alternatives

1. **Rejected:** JSONL as canonical runtime storage.
   - Reason: weak concurrency model and poor multi-client consistency.
2. **Rejected:** Dual canonical stores (SQLite + JSONL runtime writes).
   - Reason: divergence risk and long-term maintenance burden.

### Implementation Direction

1. Route CLI, FFI, and macOS reads/writes through SQLite paths.
2. Add/maintain event-log table(s) for lifecycle and audit events.
3. Keep JSONL as explicit import/export command surface only.

---

## Decision 3: CLI Default Interaction

### Decision

Make bare `stead` show a **status overview** (supervision entry point). Keep explicit subcommands for detailed operations. Support `stead status` as explicit equivalent.

### Why

1. The primary product loop is supervision and context restoration.
2. Default overview lowers friction and matches control-room concept.
3. Agent/script workflows remain stable via explicit commands and JSON output.

### Accepted Alternative

- **Accepted:** Status-first default + subcommand model.

### Rejected Alternatives

1. **Rejected:** Help-first default only.
   - Reason: optimized for command discovery, not supervision loop.
2. **Rejected:** Remove explicit subcommands in favor of only default views.
   - Reason: harms scriptability and operational precision.

### Implementation Direction

1. Add a status command output model usable by both bare `stead` and `stead status`.
2. Ensure `--json` status output is machine-stable.
3. Keep existing detailed subcommands (`run`, `list`, `show`, etc.).

---

## Authority and Scope

These three decisions are concept-level constraints for Phase 1 (planning lock). Any conflicting statements in other docs should be updated, marked historical, or archived.

This document does not define exact implementation tickets. It defines non-negotiable conceptual direction.

