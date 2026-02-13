# Step-by-Step Decision 01: API Transport Architecture

Date: 2026-02-13
Status: Resolved (Option A locked in planning baseline)
Decision owner: Jonas

Update:
- This decision is now reflected in `docs/plans/planning-baseline-2026-02-13.md`.
- Current MVP baseline remains library-first / no HTTP API / no daemon.

## Scope of this step

Decide one thing only:
- Should MVP stay **library-only / no HTTP API**, or
- should we introduce a **local HTTP API + daemon** now?

---

## 1) Current state in code (what is actually implemented)

### 1.1 CLI execution model

The CLI directly calls Rust command modules (no HTTP client layer):
- `rust/stead-cli/src/main.rs:11-13`
- `rust/stead-cli/src/main.rs:17-56`

### 1.2 Command surface shape

CLI is subcommand-based and local (`run`, `list`, `show`, etc.), not `contract/*` HTTP-backed routes:
- `rust/stead-core/src/cli/mod.rs:20-80`

### 1.3 Dependencies

No HTTP/server stack found in Cargo manifests (`axum`, `warp`, `hyper`, etc. absent):
- `rust/stead-core/Cargo.toml:9-29`
- `rust/Cargo.toml:1-8`

### 1.4 Practical conclusion from code

Implemented architecture today is:
- local monolith behavior,
- CLI -> stead-core direct calls,
- no daemon process,
- no HTTP transport layer.

---

## 2) Current state in docs (where they agree and where they conflict)

### 2.1 Docs that say "no HTTP / monolith"

- `docs/plans/phase2-revised.md:19` -> "Monolith — no HTTP API, no separate server"
- `docs/plans/phase2-revised.md:301` -> "No server — monolith, direct library calls"
- `docs/plans/phase2-revised.md:307-314` -> says old HTTP plan was superseded

### 2.2 Docs that still say "HTTP underneath + daemon"

- `docs/plans/decisions/agent-sdk-language.md:24` -> CLI with HTTP API underneath
- `docs/plans/decisions/agent-sdk-language.md:71-76` -> CLI wraps HTTP calls
- `docs/plans/decisions/agent-sdk-language.md:176` -> HTTP API runs in daemon

### 2.3 Important nuance

The core reasoning in `agent-sdk-language.md` ("agents use CLI, not imports") is still valid.
Only the transport/daemon assumption is outdated vs current architecture.

---

## 3) The single decision you need to make now

## Option A (Recommended): Lock MVP to library-only / no HTTP API

Definition:
- Keep current architecture: CLI + FFI call shared Rust core directly.
- Defer transport layer until real external-client need appears.

Pros:
1. Matches current implementation and existing momentum.
2. Lower operational complexity (no daemon lifecycle, no local network surface).
3. Keeps "single binary / local-first" product feel.

Costs:
1. External integrations via HTTP are delayed.
2. If API transport is needed later, ADR + adapter layer must be added then.

Immediate follow-up if chosen:
1. Mark `docs/plans/decisions/agent-sdk-language.md` as superseded (or rewrite transport sections).
2. Add canonical ADR for "API transport = deferred" with clear revisit triggers.

## Option B: Introduce local HTTP API + daemon now

Definition:
- Add daemon service, HTTP endpoints, and make CLI a thin client.

Pros:
1. Stable integration surface for future tools/plugins.
2. Clear protocol boundary early.

Costs:
1. Significant complexity increase now (daemon process mgmt, health, startup, failure modes).
2. More work before resolving current core inconsistencies.
3. Architectural churn against current code direction.

Immediate follow-up if chosen:
1. Add server crate and endpoint spec.
2. Rework CLI calls to HTTP.
3. Update phase roadmap and testing strategy for daemon lifecycle.

---

## 4) Recommendation basis

Recommendation: **Option A**.

Why:
1. It aligns with implemented reality (no rewrite needed to match plan).
2. It best serves current MVP goal: fast, reliable local supervision loop.
3. It removes a major docs contradiction with minimal risk.

---

## 5) Checkmark (decision output)

Use one of these as your explicit checkmark:

- `[ ] A` Lock MVP to library-only/no HTTP API.
- `[ ] B` Introduce local HTTP API + daemon now.

When you pick one, this step is complete.

---

## 6) Clarifying question for this step

Do you expect a real need in the next 1-2 milestones for third-party tools to call stead over a stable local API (not CLI subprocess)?
- If **no**, Option A is the right lock.
- If **yes**, Option B may be justified now.
