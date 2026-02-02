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

## Open Decisions

### Naming
- "stead" as project name — keep it?
- What does stead stand for? (or is it just a word?)

### First Implementation Target
- Contract engine seems foundational
- But what's the minimum slice that's useful tomorrow?

*Awaiting decision*
