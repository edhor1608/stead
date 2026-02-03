# Decisions Log

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

## Open Decisions

### Naming
- "stead" as project name — keep it?
- What does stead stand for? (or is it just a word?)

### First Implementation Target

**Decided:** See [decisions/first-slice.md](decisions/first-slice.md)

**Summary:** CLI that wraps Claude Code tasks in contracts with automated verification. `stead run "task" --verify "cmd"` — no daemon, no UI, just contracts + verification + persistence.
