# Documentation Authority Map

Date: 2026-02-11
Status: Active
Purpose: Prevent conflicting docs from acting as competing sources of truth.

## Authority Order

0. **Aggregated baseline (entry point)**
- `docs/plans/planning-baseline-2026-02-13.md`

1. **Concept and architecture constraints**
- `docs/plans/canonical-decisions-2026-02-11.md`

2. **Current shipped behavior (public-facing)**
- `README.md`
- `rust/README.md`

3. **Execution sequencing / milestones**
- `docs/plans/phase2-revised.md`

4. **Scope framing and UX direction**
- `docs/plans/mvp-scope.md`
- `docs/plans/ux-refined.md`

5. **Research and exploration**
- `docs/research/*` (non-authoritative for runtime behavior)
- Draft UX exploration docs (for example `docs/plans/control-room-ux.md`) are non-binding when they conflict with higher-order docs.

## Conflict Rule

If two docs disagree:
1. Canonical decisions win for concept-level direction.
2. README-level docs win for current behavior claims.
3. Milestone docs are interpreted as sequencing references, not runtime truth.
4. Historical/superseded docs are context only and never override active docs.

## Maintenance Rule

When a decision changes:
1. Update `docs/plans/planning-baseline-2026-02-13.md` and canonical decisions first.
2. Update README-level docs second.
3. Update milestone/scope docs third.
4. Mark stale statements as superseded/historical if not immediately updated.
