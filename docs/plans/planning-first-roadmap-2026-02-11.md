# Planning-First Roadmap

Date: 2026-02-11
Status: Active
Last Updated: 2026-02-13

## Goal

Lock planning, concept boundaries, and tech-selection decisions first.
Only after that: challenge open implementation issues.

## Canonical Baseline

Primary baseline for Phase 1 decisions:
- `docs/plans/canonical-decisions-2026-02-11.md`
- `docs/plans/docs-authority-map.md`
- `docs/plans/planning-baseline-2026-02-13.md`

## Sequence

### Phase 1: Planning and Concept Alignment (Now)

1. Define one authoritative planning baseline.
2. Resolve concept scope (MVP vs post-MVP) without ambiguity.
3. Resolve core technical direction decisions.
4. Freeze terminology and success criteria.

Output:
- A clear source-of-truth doc set
- A resolved decisions list
- A bounded “open questions” list for implementation

### Phase 2: Challenge Open Items (After Phase 1)

1. Challenge unresolved architecture and behavior gaps.
2. Validate risks against agreed planning baseline.
3. Convert accepted challenges into actionable change tasks.

Output:
- Prioritized challenge backlog
- Approved implementation changes

## Decisions To Resolve In Phase 1

Progress update (2026-02-13):
- Authority and aggregation baseline: locked
- MVP transport baseline (no HTTP/daemon for current MVP): locked
- Remaining items below are still open unless explicitly marked otherwise.

1. Contract semantics:
- Should a contract be allowed to complete if agent execution fails but verify passes?

2. State machine strictness:
- Enforce strict M6 transitions now, or keep pragmatic command behavior for current phase?

3. Control Room scope:
- Project-local contracts for now, or cross-project contracts now?

4. Storage contract:
- Must all surfaces (CLI + FFI + macOS app) read/write SQLite now?

## Definition of “Planning Locked”

Planning is considered locked when:

1. One source-of-truth doc set is explicitly selected.
2. The five decisions above are decided.
3. Conflicting statements are either removed or marked historical.
4. MVP acceptance criteria are measurable and consistent across docs.

## Next Step

Use this doc as the control checklist for Phase 1 decisions.
After these are resolved, start the challenge pass on open implementation items.
