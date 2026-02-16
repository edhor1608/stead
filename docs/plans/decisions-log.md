# Decisions Log

> Note: This is a chronological historical log.
> Some entries are intentionally superseded by newer decisions.
> For current planning truth, start with `docs/plans/planning-baseline-2026-02-13.md`.

## 2026-02-02: Project Inception

**Context:** Theo's article about parallel project cognitive overhead resonated. Jonas experiences this daily with 5+ active repos.

**Decision:** Create `stead` repo to explore this problem space seriously.

**Rationale:** The problem is real, affects daily work, and no good solution exists. Worth investigating whether something buildable could help.

**Consequences:** Need to define scope - is this research only, or are we building something?

---

## 2026-02-02: Core Architecture Direction

**Context:** Brainstormed component ideas (terminal fork, browser fork, task tracking, git replacement). Evaluated each with honest positive/negative/radical takes.

**Decision:** Adopt the radical reframings as architectural principles:

| Component | Rejected Approach | Adopted Approach |
|-----------|-------------------|------------------|
| Terminal | Fork Ghostty | Execution daemon + optional terminal view |
| Browser | Fork Helium | Session proxy layer wrapping any browser |
| Tasks | Jira/Linear model | Contract-based execution (input/output/verify/rollback) |
| UI | Dashboard | Control room (air traffic control metaphor) |
| Version Control | Replace git | Transformation layer that compiles to git |
| Memory | Store/retrieve facts | Context generator that synthesizes relevant context |

**Rationale:** Each radical take solves the actual problem without inheriting massive maintenance burden. Fork the *concept*, not the *software*.

**Consequences:** See `architecture-principles.md` for full breakdown. Build order: contract engine → control room → execution daemon → session proxy → transformation layer.

---

## 2026-02-02: What is stead?

**Decision:** Actual tool — an operating environment for agent-driven development.

**Rationale:** The problem is real, daily, and unsolved. Research without building won't fix it.

---

## 2026-02-02: Target Scope

**Decision:** Start personal (Jonas's workflow), design for general.

**Rationale:** Dogfooding ensures it solves real problems. But architecture should not be Jonas-specific.

---

## 2026-02-02: Project Memory Architecture

**Context:** Agents need persistent knowledge across sessions. Current approaches (RAG, conversation history, knowledge graphs) all treat memory as a retrieval problem.

**Decision:** Don't build a memory store. Build a context generator.

**Rationale:** The project already HAS memory — code, docs, git history, contracts, decisions. The problem isn't storage, it's synthesis. Memory isn't facts to retrieve; it's understanding that shapes behavior. A context generator synthesizes relevant context for each specific task from everything that exists.

**Key insight:** Decisions become constraints, not stored facts. History becomes patterns, not logs. Memory is embodied in the agent's starting state, not queried from a database.

**Consequences:** No separate memory system to maintain. No schema to keep updated. The "mind" is a process, not a store.

---

## 2026-02-02: Agent SDK Language

**Context:** Contract Engine needs an interface for agents to claim contracts, report status, propose transformations. Question: what language/form should this take?

**Decision:** Protocol-first, not language-first. The "SDK" is a CLI tool (`stead`) with HTTP/JSON API underneath. No language-specific library as primary interface.

**Rationale:** Agents don't import libraries — they shell out. Claude Code uses bash tools, not `import` statements. A CLI works universally (any agent that can execute commands), outputs JSON for machine consumption, and matches how agents actually work. Language-specific SDKs solve the wrong problem.

**Key insight:** The bash tool IS the SDK from the agent's perspective.

**Consequences:** Build `stead` CLI as single binary with JSON output. HTTP API underneath enables control room and optional language bindings later. TypeScript types package (optional) for humans building tooling, not for agents.

See: `docs/plans/decisions/agent-sdk-language.md`

---

## 2026-02-02: Contract Schema Format

**Context:** Contracts are the core abstraction. Need to decide the format agents will consume and produce.

**Decision:** TypeScript-native schema with JSON serialization.
- Schema definition: TypeScript interfaces
- Contract instances: JSON conforming to interfaces
- Verification: TypeScript predicates (compiled to JS)
- Storage: JSONL (append-only, one contract per line)

**Rationale:** Agents already "think" in TypeScript. Claude Code naturally produces interface-shaped structures. Fighting this wastes effort. JSON is the data, TypeScript is the type system, JavaScript predicates are executable verification.

**Key insight:** Match how agents naturally represent structured data. Don't invent a format they need to learn.

**Consequences:** Tied to JS/TS ecosystem (acceptable given agent tooling landscape). Predicates need sandboxed execution. Schema validation via Zod/Valibot at runtime.

See: `docs/plans/decisions/contract-schema-format.md`

---

## 2026-02-02: First Implementation Target

**Decided:** See [decisions/first-slice.md](decisions/first-slice.md)

**Summary:** CLI that wraps Claude Code tasks in contracts with automated verification. `stead run "task" --verify "cmd"` — no daemon, no UI, just contracts + verification + persistence.

---

## 2026-02-03: First Slice Complete

**Context:** Implemented the CLI-only contract engine with verification as the first slice.

**Decision:** First slice is complete. PR created: https://github.com/edhor1608/stead/pull/1

**What was built:**
- `stead run "task" --verify "cmd"` - Create and execute contract
- `stead list [--status=X]` - List contracts
- `stead show <id>` - Show contract details
- `stead verify <id>` - Re-run verification
- YAML storage in `.stead/contracts/`
- 64 tests passing
- Compiles to single binary

**Rationale:** Minimal viable contract engine to dogfood agent-driven development workflows.

**Consequences:** Ready to use for real tasks. Next: merge PR, then expand (daemon, UI, or session format work).

---

## 2026-02-03: Universal Session Format Exploration

**Context:** Research into AI CLI session storage (Claude Code, Codex CLI, OpenCode) revealed they all store sessions in incompatible formats. This causes vendor lock-in, fragmented visibility, and no cross-CLI workflows.

**Decision:** Explore Universal Session Format as a potential stead component.

**Rationale:** This directly addresses the Control Room vision — you can't have unified visibility without unified data. Also enables:
- Session browser across all CLIs
- Cross-CLI resume (start in Claude, continue in Codex)
- Session forking and linking
- A/B testing across models

**Connection to NORTH_STAR:**
- Reduces *ding* problem: know which CLI finished, restore context instantly
- Enables Control Room: unified view of agent state across tools
- Extends Context Generator: sessions as project memory that persists

**Key finding:** Claude Code and OpenCode share `ses_*` ID convention, making conversion between them most feasible.

**Consequences:** New research doc at `docs/research/ai-cli-session-formats.md`, spec at `docs/plans/universal-session-format.md`. Decision on priority vs other stead components TBD.

See: `docs/plans/universal-session-format.md`

---

## 2026-02-03: Execution Layer Strategy

**Context:** The NORTH_STAR daemon concept imagined building execution from scratch. But agents need to actually run somewhere, and existing CLIs (Claude Code, Codex, OpenCode) are complete agent runtimes tied to subscriptions. Building our own runtime means reimplementing API integration, tool execution, session management — all of which these CLIs already do.

**Options considered:**

| Option | Description | Effort | Trade-offs |
|--------|-------------|--------|------------|
| **A: Build own runtime** | Fork execution from scratch, handle APIs directly, implement all tools | Huge | Full control, but reinvents the wheel. Could be valuable long-term if CLIs become limiting. |
| **B: Orchestrate existing CLIs** | CLIs are execution engines, stead is control plane. USF is the adapter layer. | Medium | Dependent on CLI stability, but leverages existing work. Matches "fork the concept, not the software" principle. |

**Decision:** Option B — Orchestrate existing CLIs via Universal Session Format.

**Rationale:**
- CLIs already ARE the daemon. They execute tasks, manage state, handle tools.
- Stead's value isn't execution — it's orchestration, project-scoping, contracts, visibility.
- "Fork the concept, not the software" — don't rebuild what exists.
- Practical: CLIs are tied to subscriptions already being paid for.

**Key reframe:** The architecture becomes:
```
┌─────────────────────────────────────────────────┐
│              stead (control plane)              │
│  Contracts, orchestration, visibility, projects │
└───────────────────────┬─────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────┐
│       Universal Session Format (adapter)        │
└───────────────────────┬─────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
   Claude Code      Codex CLI       OpenCode
   (execution)      (execution)     (execution)
```

**Preserved for future:** Option A (build own runtime) remains viable if:
- CLIs become too limiting or unstable
- We need deeper control over execution
- API-direct access becomes more practical

**Consequences:**
- USF moves from "tangential" to "foundational"
- Daemon concept reframes from "build" to "orchestrate"
- Next step: build USF adapters for CLI integration

## 2026-02-03: Tech Stack - Rust

**Context:** First slice was built in TypeScript/Bun. Evaluated language options for the full implementation: TypeScript (Bun or Node), Go, Rust. Key constraint: verification predicates were originally designed as JS functions, which would lock us to a JS runtime.

**Decision:** Rust for the core implementation. Verification changes from "JS predicates" to "shell commands + expression assertions."

**Rationale:**
- Verification doesn't dictate the stack — it's not the hot path, runs occasionally
- Shell commands handle 90% of verification (same as CI/CD)
- Expression language (cel-rust or rhai) handles output inspection
- Complex verification → external scripts in any language
- Rust benefits: fast CLI startup (~2ms), single binary, strict compiler catches errors
- AI coding agents work well with Rust — compiler acts as reviewer

**Consequences:**
- Rewrite first slice from TypeScript to Rust
- Need Rust ecosystem
- Expression evaluator needed for assertions (cel-rust or similar)
- Longer compile times (acceptable tradeoff)

---

## 2026-02-03: Verification Approach Revised

**Context:** Original contract-schema-format decision specified "TypeScript predicates compiled to JavaScript." This locked us to a JS runtime.

**Decision:** Replace JS predicates with shell commands + expression assertions.

**Rationale:** Commands handle behavior checks (tests pass, server responds). Expressions handle output inspection (agent followed constraints). External scripts handle complex logic. No embedded JS runtime needed.

**Consequences:** Update contract-schema-format.md to reflect new approach.

---

## 2026-02-04: Monolith Architecture (No HTTP API)

**Context:** Phase 2 plan proposed HTTP API layer with server + CLI as separate processes. Questioned whether HTTP abstraction is needed when everything runs locally on one device with one user.

**Options considered:**

| Option | Description | Trade-offs |
|--------|-------------|------------|
| HTTP Server | CLI and Control Room talk to central server | Standard pattern, but server to manage, overhead for local data |
| Tauri IS Stead | Control Room app contains the engine, CLI reads/writes same storage | Simpler, no server, but need coordination on storage |
| SQLite + File Watcher | Shared SQLite, Control Room watches for changes | No server, but polling/watching |
| Unix Sockets | Local IPC instead of HTTP | Lower overhead, but still daemon to manage |

**Decision:** Option 2 — Tauri IS Stead (Monolith).

**Rationale:**
- Everything runs on one machine, one user — why serialize over HTTP?
- No server to start/stop/manage
- Tauri apps are Rust backend + web frontend — the backend IS the contract engine
- Simpler architecture, fewer moving parts
- "Is the server running?" stops being a failure mode

**Architecture:**
```
┌─────────────────────────────────────────────────────────────┐
│              Control Room (Tauri App = Stead)               │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Rust Backend (stead-core library)                     │ │
│  │  • Contract Engine  • USF Adapters  • SQLite Storage   │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Web Frontend (attention-prioritized Control Room)     │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

┌──────────────┐
│    CLI       │ ──── uses same stead-core library ────▶ SQLite
└──────────────┘
```

**Consequences:**
- M2 (HTTP API) is removed from plan
- Storage moves from JSONL to SQLite (better concurrent access)
- CLI and Control Room share stead-core library
- Simpler to reason about, fewer processes

---

## 2026-02-04: Native UI Strategy (Rust Library + Platform UIs)

**Context:** Decided on monolith approach. Next question: what UI framework? Tauri uses web UI (HTML/CSS/JS in webview). But Mac users care about native feel.

**Options considered:**

| Option | Description | Trade-offs |
|--------|-------------|------------|
| Tauri (Web UI) | One codebase, all platforms | Never truly native feel |
| Native per platform | SwiftUI (Mac), WinUI (Win), GTK (Linux) | Best UX, but 3 codebases |
| Native Mac, Tauri others | SwiftUI for daily driver, web for others | Best where it matters, pragmatic |

**Decision:** Rust library + Native UIs per platform (starting with SwiftUI for Mac).

**Rationale:**
- Rust compiles to a library any language can call (FFI)
- SwiftUI is delightful for Mac apps, feels native
- You're on Mac — optimize for your daily driver
- Architecture enables cross-platform later without rewrite
- "Fork the concept, not the software" — the library IS the concept, UI is just a view

**Architecture:**
```
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│   macOS App     │  │  Windows App    │  │   Linux App     │
│   (SwiftUI)     │  │  (WinUI 3)      │  │    (GTK)        │
└────────┬────────┘  └────────┬────────┘  └────────┬────────┘
         │                    │                    │
         └────────── FFI (swift-bridge/UniFFI) ────┘
                              │
                    ┌─────────▼─────────┐
                    │   stead-core      │
                    │  (Rust library)   │
                    └───────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │       CLI         │
                    └───────────────────┘
```

**Tools:**
- **swift-bridge** or **UniFFI** for Rust → Swift bindings
- Same stead-core library used by CLI and all UIs

**Consequences:**
- Project structure splits into: stead-core (lib), stead-cli (binary), stead-ffi (bindings), macos/ (SwiftUI app)
- Start with Mac app, add other platforms if/when needed
- No Tauri for now (may add later for Linux/Windows)

---

## 2026-02-04: Storage Migration (JSONL → SQLite)

**Context:** With monolith architecture, CLI and Control Room both access storage. JSONL is append-only and requires file rewrites for updates. Multiple processes accessing same file = coordination issues.

**Decision:** Migrate from JSONL to SQLite.

**Rationale:**
- SQLite handles concurrent access safely (built-in locking)
- ACID transactions for data integrity
- Query capability (filter by status, project, date)
- Still single-file storage (`.stead/stead.db`)
- Battle-tested (SQLite is everywhere)

**Consequences:**
- Rewrite storage layer
- Migration path for existing JSONL data
- Can use `rusqlite` crate

---

## 2026-02-14: Rewrite Branch (`rewrite/v1`) M0-M4 Lock

**Context:** We restarted implementation on a clean modular Rust workspace with strict TDD checkpoints per slice and explicit goal to make subsystems exportable as standalone GitHub projects.

**Decision:** Keep canonical concepts, but enforce them as isolated crates with hard test boundaries:
- `stead-contracts`: 10-state lifecycle engine + typed transition errors + actor permissions + SQLite snapshot/event store.
- `stead-daemon`: versioned command envelope, typed error contract, cursor-based event replay.
- `stead-resources`: standalone resource claim/release registry with deterministic negotiation and escalation-only failure surfacing.

**Rationale:**
- Preserves concept truth (lifecycle, supervision projection, agent coordination) while making each subsystem reusable.
- Maintains a strict "engine truth first, adapters second" layering for CLI/macOS/app clients.
- Allows incremental replacement of legacy paths without blocking testable progress.

**Key policy lock for resource negotiation:**
- Port conflicts are resolved silently by assigning the **lowest available next port** (`requested+1` upward) within configured range.
- If no port is available, emit explicit escalation event (`port_range_exhausted`) for attention surfaces.

**Consequences:**
- M0-M4 slices are implemented with failing tests first and crate/workspace green checkpoints.
- Daemon now supports built-in resource contention handling and escalation event streaming.
- Modular crates are now package-metadata-complete for separate publication paths.

---

## 2026-02-14: Rewrite Branch (`rewrite/v1`) M5 USF Adapter Contract

**Context:** The rewrite requires USF as a reusable abstraction layer, decoupled from concrete CLIs while preserving conformance across Claude Code, Codex CLI, and OpenCode.

**Decision:** Implement `stead-usf` as a standalone adapter + query crate with locked fixture conformance tests per CLI.

**Rationale:**
- Keeps CLI/tool specifics behind adapter boundaries.
- Makes cross-tool control room consumption deterministic and testable.
- Supports exporting USF as an independent open-source module.

**Locked behavior:**
- Each adapter maps its source fixture shape to a canonical `SessionRecord`.
- Unified query contract supports deterministic sort (`updated_at desc`, `id asc`) plus CLI/text filters.
- Corrupt/partial inputs return typed errors (`invalid_json` or `invalid_format`) instead of panics.

**Consequences:**
- `stead-usf` now ships fixture-backed tests for conformance, listing contract, and resilience.
- The crate can be consumed independently from `stead-core` legacy USF code.

---

## 2026-02-14: Rewrite Branch (`rewrite/v1`) M6-M7 Optional Modules

**Context:** Session proxy and context generator are optional modules but must be enabled by default and must not weaken core guarantees.

**Decision:** Implement both in `stead-module-sdk` with explicit contracts and deterministic fallback behavior.

**Rationale:**
- Keeps module behavior reusable outside the main runtime.
- Gives a stable SDK boundary for daemon/client integration later.
- Preserves "core remains stable even when optional modules are toggled."

**Locked behavior (M6):**
- Session identities are project-scoped and unique per creation.
- Tokens are valid only in their original project boundary.
- Destroying an identity invalidates only that identity.
- Module manager enables `session_proxy` and `context_generator` by default.
- Disabling `session_proxy` gates that module only; core operations remain unaffected.

**Locked behavior (M7):**
- Context assembly is deterministic from sorted source fragments.
- Provider contract supports primary + fallback providers.
- Generated output includes provenance citations and confidence score.
- When providers are unavailable/failed, deterministic local fallback is returned.

**Consequences:**
- `stead-module-sdk` now has fixture-free deterministic tests for isolation, lifecycle, toggling, provider fallback, provenance, and backend-unavailable fallback.
- Optional-module semantics are formalized before daemon/CLI wiring.

---

## 2026-02-14: Rewrite Branch (`rewrite/v1`) M8-S1 Status-First CLI Entry

**Context:** Canonical decision requires bare `stead` to act as supervision entry point. Existing CLI required a subcommand and defaulted to help/error.

**Decision:** Add status-first default path in `stead-cli`:
- `stead` prints a status overview.
- `stead --json` prints machine-readable status JSON.
- Both call daemon `Health` endpoint (daemon-backed default path).

**Rationale:**
- Aligns CLI entry behavior with control-room supervision loop.
- Keeps subcommand workflows intact while enabling low-friction "what needs attention now?" checks.

**Consequences:**
- New integration tests lock default text/JSON behavior.
- CLI now depends on `stead-daemon` for default status path.
- Existing command suites remain green.

---

## 2026-02-15: Rewrite Branch (`rewrite/v1`) M8-S2 Grouped CLI Surface Cutover

**Context:** The rewrite direction is no backward-facing command surface. Status-first entry (M8-S1) was in place, but top-level legacy verbs were still scaffolded.

**Decision:** Replace CLI parser/dispatch with grouped command families only:
- `contract`
- `session`
- `resource`
- `attention`
- `context`
- `module`
- `daemon`

**Rationale:**
- Aligns CLI shape with concept-level architecture (daemon-backed control plane + modular capabilities).
- Removes accidental coupling to older command model.
- Gives stable machine-facing surface per family for further iteration.

**Implementation notes:**
- Contract list + attention counts promoted into daemon API (`ListContracts`, `AttentionStatus`).
- Resource lease state made durable across CLI invocations (`resources.json`) so negotiation semantics hold outside a long-lived process.
- Session parse path now routes through standalone `stead-usf`.
- Context generation path now routes through `stead-module-sdk` contracts.
- Module enable/disable/list now managed via `.stead/modules.json`.

**Consequences:**
- New grouped CLI integration tests are canonical for command-family behavior.
- Old top-level verbs are removed from CLI parser.
- Full workspace fmt/tests are green after cutover.

---

## 2026-02-16: Parallel Track Added - Rust-Native Named Localhost Broker (`portless` Concept)

**Context:** We identified `vercel-labs/portless` as strong conceptual overlap with stead's resource negotiation and session-proxy goals. We want the concept, but not an external runtime dependency, and M9 is already in progress in a parallel implementation thread.

**Decision:** Add a **parallel** Rust-native implementation track for a named-localhost broker module (portless-style behavior) that is reusable as a standalone crate and integrated through existing daemon/resource/module boundaries.

**Rationale:**
- Preserves rewrite principle: from-scratch, concept-forward, no backward coupling.
- Keeps core behavior in Rust and exportable as independent GitHub module(s).
- Directly targets real pain: stable project URLs + deterministic port conflict handling.

**Plan placement and execution rule:**
- This is added as a new parallel track, **non-blocking for active M9 work**.
- Integration should happen through existing seams (`stead-resources`, `stead-daemon`, `stead-module-sdk`, `stead-cli`) instead of introducing a new architecture layer.

**Planned TDD slices (new track):**
- **M11-S1:** Domain contract tests for named endpoint leases (claim/release/ownership, deterministic naming rules).
- **M11-S2:** Negotiation tests for name+port conflicts with deterministic next-port assignment and escalation on range exhaustion.
- **M11-S3:** Daemon API tests for endpoint claim/list/release envelopes and typed errors.
- **M11-S4:** CLI tests for endpoint workflows and stable JSON output.
- **M11-S5:** Session-proxy integration tests validating project-bound stable URL mapping and module on/off behavior.

**Consequences:**
- `portless` concept is now explicitly on the roadmap without changing current M9 critical path.
- Future implementation must follow existing strict TDD checkpoint protocol per slice.
- Detailed execution plan lives in `docs/plans/rewrite-parallel-track-m11-named-localhost.md`.

---

## Open Decisions

### Naming
- "stead" as project name — keep it?
- What does stead stand for? (or is it just a word?)

---

## 2026-02-16: Rewrite Branch (`rewrite/v1`) Session Surface Parity + M10 SLO Test Gates

**Context:** The grouped CLI rewrite still lacked `session list`, which blocked CLI/UI parity expectations and the M10 session-list latency gate.

**Decision:**
- Add `stead session list` with:
  - workspace-local discovery from `.stead/sessions/{claude,codex,opencode}`
  - deterministic recency ordering via USF query contract
  - `--cli` and `--query` filters
  - corrupt/partial file tolerance (skip invalid files, keep command successful)
- Add explicit SLO tests:
  - `session list` under target load (<200ms)
  - state propagation latency gate (<5s)
  - ding-to-context restoration scenario (<10s)
  - concurrent client soak stability gate

**Rationale:**
- Restores a key canonical session workflow without re-introducing legacy surface coupling.
- Locks SLO requirements as executable tests instead of aspirational docs.

**Consequences:**
- CLI now has both `session parse` and `session list`.
- M10 gates are represented in test suites and run in normal workspace test passes.

---

## 2026-02-16: M11 Named Localhost Broker Implemented (S1-S5)

**Context:** The parallel M11 plan was defined but not yet executed. We needed executable, reusable implementation slices with strict TDD checkpoints and no legacy coupling.

**Decision:** Implement M11 end-to-end using the recommended decision-gate defaults:
- CLI namespace: `stead resource endpoint ...`
- First URL format: `http://<name>.localhost:<port>`
- Crate boundary: new reusable crate `stead-endpoints`
- Persistence scope: workspace-local (`.stead/endpoints.json` through daemon storage pathing)

**Implemented slices:**
- **M11-S1/S2** (`stead-endpoints`):
  - Domain tests for claim/release/idempotency/import-export.
  - Deterministic negotiation tests and range-exhaustion escalation events.
- **M11-S3** (`stead-daemon`):
  - API requests/responses for endpoint claim/list/release.
  - Typed errors for `not_found`, `not_owner`, `conflict`, `endpoint_range_exhausted`.
- **M11-S4** (`stead-cli`):
  - `resource endpoint claim|list|release` flows.
  - Stable JSON payload tests and typed JSON error output (`error.code`, `error.message`).
- **M11-S5** (`stead-module-sdk`):
  - Session-proxy endpoint mapping tests for deterministic project mapping, module-disable fallback, and project scoping.

**Rationale:**
- Delivers the `portless` concept in a Rust-native, exportable module path.
- Keeps integration via existing seams (daemon, CLI, module SDK) instead of adding architecture layers.
- Locks behavior with tests before expanding features.

**Consequences:**
- Named localhost endpoint brokering is now a real, test-backed capability in rewrite/v1.
- Core reusable building blocks are in standalone crates suited for separate GitHub export.
- Remaining M11 future work is enhancement-level (proxy/no-port UX, multi-machine/global coordination), not foundation work.

---

## 2026-02-17: CI Quality Gates Hardened (Coverage + macOS UI Tests)

**Context:** Test-strategy targets and milestone expectations required enforceable quality gates, but CI still lacked strict domain coverage thresholds and direct macOS Control Room test execution.

**Decision:** Extend `.github/workflows/ci.yml` with:
- `coverage` job using `cargo-llvm-cov`
- hard line-coverage gates (`--fail-under-lines 90`) for:
  - `stead-contracts`
  - `stead-resources`
  - `stead-usf`
- `macos-ui-tests` job running:
  - `xcodebuild -project macos/Stead/Stead.xcodeproj -scheme Stead -destination 'platform=macOS' test`
- build job now depends on both `coverage` and `macos-ui-tests`

**Rationale:**
- Converts coverage and UI quality expectations into merge-blocking checks.
- Prevents regressions in both domain correctness and SwiftUI behavior.

**Consequences:**
- PR pipeline now enforces coverage policy and macOS app test health by default.
- Added workflow guard tests in `stead-test-utils` to lock CI contract shape.

---

## 2026-02-17: Session Proxy Endpoint Mapping Exposed in CLI

**Context:** Endpoint mapping existed in module/domain layers, but there was no direct user-facing session command to exercise the session-proxy endpoint concept in real workflows.

**Decision:** Add `stead session endpoint` command (daemon-backed):
- `stead --json session endpoint --project <path> --owner <owner>`
- respects module toggles from `.stead/modules.json`
  - when `session_proxy=false`: returns JSON `null` and exits successfully
- when enabled:
  - derives deterministic endpoint name from project path
  - claims endpoint via daemon `ClaimEndpoint`
  - returns stable JSON envelope from endpoint claim result

**Rationale:**
- Turns M11/M6 concept into practical operator-facing functionality.
- Keeps behavior aligned with module enable/disable controls.

**Consequences:**
- Real usage now exists for session->endpoint flow without leaving CLI.
- CLI integration tests lock deterministic same-project behavior and cross-project negotiation behavior.

---

## 2026-02-17: Documentation Drift Guardrails for Rewrite Surface

**Context:** Authority docs and README-level behavior docs drifted from shipped rewrite behavior. Public docs still described legacy top-level commands and monolith/no-daemon runtime claims.

**Decision:**
- Add executable documentation guard tests in `rust/stead-test-utils/tests/docs_consistency.rs` to lock:
  - grouped command-family CLI claims in `/README.md`
  - daemon-backed Rust workspace description in `/rust/README.md`
  - daemon-backed runtime statement in `/docs/plans/planning-baseline-2026-02-13.md`
  - canonical decision text aligned to grouped command families in `/docs/plans/canonical-decisions-2026-02-11.md`
- Update the above docs to match current rewrite reality.

**Rationale:**
- README-level docs are authoritative for shipped behavior; drift here creates implementation confusion and planning churn.
- Locking docs with tests keeps future slices from accidentally reintroducing stale architecture claims.

**Consequences:**
- `cargo test --workspace` now includes a docs-consistency gate for rewrite command-surface and architecture claims.
- Planning baseline/canonical docs now align with grouped daemon-backed CLI behavior.
