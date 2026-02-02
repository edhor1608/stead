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

## Open Decisions

### Naming
- "stead" as project name — keep it?
- What does stead stand for? (or is it just a word?)

### First Implementation Target

**Decided:** See [decisions/first-slice.md](decisions/first-slice.md)

**Summary:** CLI that wraps Claude Code tasks in contracts with automated verification. `stead run "task" --verify "cmd"` — no daemon, no UI, just contracts + verification + persistence.
