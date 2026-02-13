# Deep Audit: Plans, Concepts, and Tech Selection

Date: 2026-02-11
Scope: `docs/plans/*`, `docs/plans/decisions/*`, selected `docs/research/*` context
Mode: Planning and concept challenge (not implementation bug review)

---

## Step 1: Audit Method and Decision Basis

This audit challenges the planning layer, not just summarizes it.

Method:
1. Build a full planning corpus inventory.
2. Classify docs by authority and status.
3. Trace concept chain from problem -> product -> architecture -> milestones -> UX behavior.
4. Revalidate major tech selections against explicit criteria.
5. Identify contradictions, stale assumptions, and missing decisions.
6. Produce a lock package: what to decide now, what to defer, and what to rewrite.

Decision criteria used for revalidation:
- Fit to the product problem (*ding* -> context restoration).
- Operational complexity for a solo local-first tool.
- Reliability and data integrity.
- Implementation and maintenance cost.
- Future extensibility without premature complexity.
- Cross-doc coherence (can this choice be documented consistently now).

---

## Step 2: Documentation Governance Reality

### 2.1 What is now strong

1. A canonical baseline exists:
   - `docs/plans/canonical-decisions-2026-02-11.md`
2. Precedence rules exist:
   - `docs/plans/docs-authority-map.md`
3. Planning-first sequencing exists:
   - `docs/plans/planning-first-roadmap-2026-02-11.md`

This is a big improvement. Before this, multiple docs behaved like equal truth.

### 2.2 Why governance is still not locked

The authority model exists, but several active docs still carry conflicting architecture claims.

P0 examples:
1. API transport model conflict:
   - HTTP + daemon model in `docs/plans/decisions/agent-sdk-language.md:24`, `docs/plans/decisions/agent-sdk-language.md:71`, `docs/plans/decisions/agent-sdk-language.md:176`
   - No-HTTP monolith in `docs/plans/phase2-revised.md:19`
2. Canonical storage conflict:
   - SQLite canonical in `docs/plans/canonical-decisions-2026-02-11.md:55`
   - JSONL architectural storage in `docs/plans/decisions/contract-schema-format.md:29`, `docs/plans/decisions/contract-schema-format.md:61`
3. FFI bridge conflict:
   - `swift-bridge` in `docs/plans/phase2-revised.md:33`, `docs/plans/phase2-revised.md:181`
   - UniFFI preference in `docs/plans/ux-refined.md:794`

Assessment:
- Governance is structurally present.
- Enforcement is incomplete.
- This is the main remaining planning risk.

---

## Step 3: Concept Coherence (Problem -> Product -> Architecture)

### 3.1 Problem and product framing coherence

Strong and coherent across core docs:
- Problem framing in `docs/plans/product-vision.md:11-16`.
- Supervision-over-execution framing in `docs/plans/product-vision.md:43-55`.
- Attention-priority control-room framing in `docs/plans/product-vision.md:47-49`.
- Agent-first architecture paradigm in `docs/plans/architecture-principles.md:7-10`.

Verdict: `PASS`

### 3.2 Horizon boundary drift (MVP vs future pillars)

Conflict pattern:
- Architecture pillars present execution daemon/session proxy as decisions:
  - `docs/plans/architecture-principles.md:15-27`
  - `docs/plans/architecture-principles.md:31-44`
- MVP explicitly defers those:
  - `docs/plans/mvp-scope.md:86-87`

This is acceptable only if horizon tags are explicit. Today, they are not consistently tagged.

Verdict: `PARTIAL FAIL`

Recommendation:
- Add explicit horizon tags to each major concept:
  - `MVP-CANONICAL`
  - `POST-MVP-CANDIDATE`
  - `VISIONARY-EXPLORATION`

### 3.3 Architecture sequence coherence

Conflict pattern:
- `architecture-principles` build order says control room before execution daemon (`docs/plans/architecture-principles.md:239-247`).
- `phase2-revised` sequence is M2 -> M3 -> M4 -> M5 -> M6 and defers daemon (`docs/plans/phase2-revised.md:269-293`).

Not fatal, but sequence intent is split across docs.

Verdict: `PARTIAL FAIL`

---

## Step 4: Detailed Revalidation of Tech Selections

## 4.1 Runtime and language (Rust)

Evidence:
- Rust as core in `docs/plans/phase2-revised.md:20`.
- Rust as MVP core in `docs/plans/mvp-scope.md:22`.
- Rust-native schema decision in `docs/plans/decisions/contract-schema-format.md:23`.

Challenge:
- Is Rust still the best fit if we optimize for solo usage, fast CLI, shared core, and future native UI?

Revalidation result:
- Yes. Rust remains the strongest fit for current constraints.

Why (basis):
1. Single binary + fast startup aligns with CLI wedge.
2. Shared core library for CLI and macOS UI aligns with architecture.
3. Memory safety and strict typing reduce long-term maintenance risk.

Verdict: `PASS`

## 4.2 Storage model (SQLite vs JSONL)

Evidence:
- SQLite canonical decision in `docs/plans/canonical-decisions-2026-02-11.md:55-58`.
- JSONL as storage decision still active in old decision docs:
  - `docs/plans/decisions/contract-schema-format.md:29`, `docs/plans/decisions/contract-schema-format.md:260`
  - `docs/plans/decisions/first-slice.md:51-53`, `docs/plans/decisions/first-slice.md:144`
- Phase plan affirms SQLite migration in `docs/plans/phase2-revised.md:135-145`.

Challenge:
- Is SQLite still correct if transparency and git-trackability were original JSONL advantages?

Revalidation result:
- SQLite is still the right canonical runtime choice.
- JSONL should stay only as import/export and migration surface.

Why (basis):
1. Concurrent CLI + app access requires transactional semantics.
2. Queryability is required for control-room behavior and filtering.
3. Dual runtime stores create divergence risk.

Missing piece:
- Canonical decisions promise append-only event history in SQLite (`docs/plans/canonical-decisions-2026-02-11.md:55`, `docs/plans/canonical-decisions-2026-02-11.md:68`), but there is no authoritative event schema doc yet.

Verdict: `PARTIAL PASS`

## 4.3 Transport and API shape (library-only vs HTTP daemon)

Evidence:
- HTTP-under-CLI model in `docs/plans/decisions/agent-sdk-language.md:24`, `docs/plans/decisions/agent-sdk-language.md:73-76`, `docs/plans/decisions/agent-sdk-language.md:176`.
- No-HTTP monolith model in `docs/plans/phase2-revised.md:19`.
- Decisions log contains both historical directions:
  - HTTP framing `docs/plans/decisions-log.md:70-77`
  - no-HTTP monolith `docs/plans/decisions-log.md:236-280`

Challenge:
- Should MVP include a local HTTP layer for future clients, or avoid it until clearly needed?

Revalidation result:
- For MVP and current scope, library-first/no-HTTP is the better decision.

Why (basis):
1. Reduces moving parts for local single-user operation.
2. Preserves strong shared-core architecture with CLI + FFI.
3. Avoids daemon lifecycle and observability overhead too early.
4. HTTP can be introduced later behind a stable core API if multi-client needs become concrete.

Required action:
- Mark `agent-sdk-language.md` as superseded or rewrite it to current architecture.

Verdict: `FAIL` until supersession is explicit.

## 4.4 FFI bridge (UniFFI vs swift-bridge)

Evidence:
- `swift-bridge` in phase sequencing `docs/plans/phase2-revised.md:181`.
- UniFFI preference in UX spec `docs/plans/ux-refined.md:794`.
- Ambiguous references in architecture and decisions log:
  - `docs/plans/architecture-principles.md:243`, `docs/plans/architecture-principles.md:263`
  - `docs/plans/decisions-log.md:325`

Challenge:
- Which bridge better fits long-term maintainability and type boundary stability?

Revalidation result:
- UniFFI is the better default recommendation for this project state.

Why (basis):
1. Better alignment with multi-language expansion path (future-proofing).
2. Stronger boundary generation model and ecosystem maturity for Rust-first projects.
3. Better fit with “shared Rust core, native UI layers” direction.

Counterpoint:
- `swift-bridge` can be simpler in narrow Rust<->Swift-only cases. If this project will remain Mac-only forever, that argument gets stronger.

Required action:
- Create one canonical FFI decision record and update `phase2-revised.md` accordingly.

Verdict: `FAIL` until explicit lock.

## 4.5 UI architecture and behavior coherence

Evidence of conflicting active specs:
1. Layout conflict:
   - No sidebar in MVP: `docs/plans/ux-refined.md:128-133`, `docs/plans/ux-refined.md:792`
   - Three-column split view baseline: `docs/plans/design-language.md:171`
2. Motion conflict:
   - Animated running indicator in draft: `docs/plans/control-room-ux.md:355`
   - Never animate running state: `docs/plans/design-language.md:198`
3. Loading-state conflict:
   - Skeleton on app launch: `docs/plans/control-room-ux.md:365`
   - No loading skeletons: `docs/plans/design-language.md:201`
4. Visual semantics conflict:
   - Completed blue in draft: `docs/plans/control-room-ux.md:91`, `docs/plans/control-room-ux.md:357`
   - Completed green in refined/design docs: `docs/plans/ux-refined.md:147`, `docs/plans/design-language.md:40`

Challenge:
- Can a contributor implement UI confidently from the current docs without second-guessing?

Revalidation result:
- No. UI canon is currently split.

Recommendation:
1. Make `ux-refined.md` the single MVP interaction source.
2. Keep `design-language.md` as style constraints only.
3. Mark `control-room-ux.md` as exploratory draft or archive it.

Verdict: `FAIL`

## 4.6 Session normalization (USF) status coherence

Evidence:
- USF still marked proposal with unchecked phases: `docs/plans/universal-session-format.md:3`, `docs/plans/universal-session-format.md:182-204`.
- Phase2 marks read adapters complete in M1: `docs/plans/phase2-revised.md:86-95`.

Challenge:
- Is USF a delivered MVP component, or still concept work?

Revalidation result:
- Partially delivered. Documentation does not separate delivered read-adapter scope from future round-trip/orchestration scope.

Recommendation:
- Split USF status by phase:
  - Phase 1: Implemented
  - Phase 2-4: Proposed

Verdict: `PARTIAL FAIL`

## 4.7 Test strategy as planning input quality

Evidence:
- Current state says 88 tests and no CI: `docs/plans/test-strategy.md:9`, `docs/plans/test-strategy.md:13`.
- Strategy references old integration path: `docs/plans/test-strategy.md:11`.

Challenge:
- Can planning decisions rely on this doc's "current state" section?

Revalidation result:
- Not fully. Strategic direction is useful, baseline facts are stale.

Verdict: `PARTIAL FAIL`

---

## Step 5: Decision Matrix (Revalidation Basis)

### 5.1 API transport options

| Option | Strengths | Risks | Fit for current scope |
|---|---|---|---|
| Library-only (no HTTP in MVP) | Simplest ops, lowest local complexity, direct core reuse | Harder external integrations until adapter added | High |
| Local HTTP daemon now | Standard external interface, easy tool integration | Adds process lifecycle/observability complexity early | Medium |
| Local socket IPC now | Lower overhead than HTTP | Still introduces daemon and IPC surface complexity | Low-Medium |

Recommendation: Library-only in MVP; define trigger criteria for introducing API transport later.

### 5.2 Storage options

| Option | Strengths | Risks | Fit for current scope |
|---|---|---|---|
| SQLite canonical + event tables | Concurrency-safe, queryable, robust | Needs schema discipline/migrations | High |
| JSONL canonical | Human-readable, line-diff friendly | Weak concurrent updates, harder querying | Low |
| Dual canonical (SQLite + JSONL writes) | Appears flexible | Split-brain risk, higher maintenance | Very Low |

Recommendation: SQLite canonical only; keep JSONL import/export.

### 5.3 FFI options

| Option | Strengths | Risks | Fit for current scope |
|---|---|---|---|
| UniFFI | Better long-term boundary model, strong Rust ecosystem support | Learning/setup overhead | High |
| swift-bridge | Swift-specific simplicity in narrow use cases | More constrained long-term expansion story | Medium |
| Manual C ABI wrappers | Maximum control | Highest maintenance and safety burden | Low |

Recommendation: UniFFI, unless project scope is explicitly fixed to Mac-only forever.

### 5.4 UI structure options (MVP)

| Option | Strengths | Risks | Fit for current scope |
|---|---|---|---|
| Single-column attention-priority | Strong supervision model clarity, low complexity | Less spatial navigation early | High |
| 3-column split from day one | Familiar macOS app shell for complex browsing | Competes with priority-first mental model | Medium |

Recommendation: Single-column MVP; revisit split layout after data complexity grows.

---

## Step 6: Contradiction and Drift Register

### P0 (must resolve before claiming planning lock)

1. API model: HTTP daemon vs monolith/no-HTTP.
2. Storage model: JSONL architectural language still active vs SQLite canonical.
3. FFI bridge: `swift-bridge` vs UniFFI.

### P1 (resolve before UX implementation hardening)

4. UI layout canon split (single-column vs 3-column split).
5. Running animation semantics conflict.
6. Loading skeleton policy conflict.
7. Completed-state color semantics conflict.

### P2 (planning hygiene and truthfulness)

8. USF proposal status vs M1-complete claims.
9. Test strategy baseline facts stale.
10. Historical decisions mixed with active decisions in ways that look currently binding.

---

## Step 7: Completeness Gaps (Missing Decisions and Specs)

1. Missing canonical ADR: API transport strategy and revisit triggers.
2. Missing canonical ADR: FFI bridge selection and migration path.
3. Missing canonical schema: SQLite event-history model promised in canonical decisions.
4. Missing horizon metadata across major docs (MVP vs post-MVP vs exploratory).
5. Missing supersession markers in older decisions docs.

---

## Step 8: What Is Working Well (Do Not Reopen Unnecessarily)

1. Product thesis is coherent and differentiated (attention-first supervision).
2. Rust core direction is strong.
3. SQLite canonical direction is strong.
4. Planning-first governance setup (canonical decisions + authority map) is the right control move.

---

## Step 9: Recommended Lock Package (Concrete)

## 9.1 Lock now (required)

1. `ADR-API-TRANSPORT`
   - Choose library-only MVP.
   - Define explicit triggers for adding local API (examples: external plugins requiring stable IPC, multi-process supervision requirements).
2. `ADR-FFI-BRIDGE`
   - Choose UniFFI or `swift-bridge` explicitly.
   - Document acceptance criteria and migration policy.
3. `ADR-STORAGE-EVENT-MODEL`
   - Define snapshot tables + append-only events schema.
   - Define JSONL import/export responsibilities.

## 9.2 Normalize docs immediately after lock

1. Rewrite or mark superseded:
   - `docs/plans/decisions/agent-sdk-language.md`
   - JSONL-core claims in `docs/plans/decisions/contract-schema-format.md`
   - JSONL-core claims in `docs/plans/decisions/first-slice.md`
2. Align phase roadmap with chosen FFI bridge:
   - `docs/plans/phase2-revised.md`
3. Set one MVP UX source and demote conflicting docs:
   - keep one canonical spec, mark others as non-authoritative.

## 9.3 Status hygiene pass

1. Update `docs/plans/universal-session-format.md` phase checkboxes/status labels to reflect delivered Phase 1 parts.
2. Refresh `docs/plans/test-strategy.md` current-state facts.
3. Add "last verified" metadata to active planning docs.

---

## Step 10: Final Assessment

### Planning confidence by area

- Product concept: `High`
- Runtime choice (Rust): `High`
- Storage direction (SQLite): `High`, but spec gap remains
- API architecture lock: `Low` (until supersession action is done)
- FFI architecture lock: `Low` (until one bridge is locked)
- UX architecture lock: `Medium-Low` (canon split)
- Planning docs factual hygiene: `Medium`

### Bottom line

The concept is strong and technically viable. The main risk is not bad architecture; it is unresolved documentation authority in a few high-impact decisions.

Once transport, FFI, and UX canon are explicitly locked and superseded docs are marked, planning becomes reliable enough for the next implementation challenge round.

---

## Questions Requiring Your Explicit Decision

1. API for MVP: keep library-only/no-HTTP, or add a local API now?
2. FFI: lock UniFFI, or lock `swift-bridge`?
3. UX canonical source: keep `ux-refined.md` as MVP truth, or consolidate into a new single spec?
4. Supersession policy: archive old decision docs, or keep them with a mandatory "Superseded by" header?
