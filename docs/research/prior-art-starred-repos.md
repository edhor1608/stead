# Prior Art: Jonas's Starred Repos

**Date:** 2026-02-02
**Source:** GitHub starred repositories analysis

---

## High Relevance

### 1. JasonDocton/lucid-memory
**Cognitive memory system implementing ACT-R architecture.**

- **What:** Local, reconstructive memory (not retrieval). 2.7ms retrieval, $0/query.
- **Memory model:** Three activation sources:
  - Base-level (recency/frequency)
  - Probe-trace similarity (relevance)
  - Spreading activation (associations amplify)
- **Key insight:** "Memory evolves, details fade, essence persists" = stead's "memory is alive, not a log"
- **Relevance:** Validates context generator approach. Activation model is exactly what we need.

---

### 2. obra/superpowers
**Agent-driven development workflow framework.**

- **What:** Skills that auto-trigger during coding. Brainstorm → spec → plan → execute → review → merge.
- **Key patterns:**
  - Skills as markdown files (`SKILL.md` + YAML frontmatter)
  - Subagent-driven-development (fresh agent per task)
  - Git worktrees for parallel isolation
  - Two-stage review (spec compliance → code quality)
- **Relevance:** We're already using this. Plans are close to contracts but lack formal I/O specs.

---

### 3. opral/lix
**Embeddable version control as a library.**

- **What:** VCS using SQL databases. Semantic change tracking (not line diffs).
- **Key patterns:**
  - Sees `price: 10 → 12` not "line 4 changed"
  - Plugin architecture for format understanding
  - SQL-queryable change history
- **Relevance:** Directly relevant to transformation layer. But doesn't compile to git (we need that).
- **Gap:** lix is for embedding in apps, not git compatibility. Stead must compile to git.

---

### 4. stravu/crystal
**Desktop app for parallel Claude Code sessions in git worktrees.**

- **What:** Each session isolated, changes committed per iteration, squashed to main.
- **Relevance:** Validates parallel agent management. Control room UI inspiration.
- **Gap:** No execution daemon, no session proxy, no contracts.

---

### 5. steveyegge/beads
**Git-backed graph issue tracker for AI agents.** 14.3k stars.

- **What:** Issues as JSONL in `.beads/`, synced via git.
- **Key patterns:**
  - Content-addressed IDs (prevent merge collisions): `bd-a1b2`
  - `bd ready` - lists tasks with no open blockers (agent-optimized)
  - **Gates:** `AwaitType` (gh:run, gh:pr, timer, human) + `AwaitID` for async
  - Three-layer model: SQLite (fast) + JSONL (git) + remote (distributed)
  - Compaction: semantic "memory decay" summarizes old tasks
- **Relevance:** HIGH. Solves many same problems. Gates are excellent.
- **Gap:** Still issue-centric, not contract-centric. No verification/rollback primitives.

---

### 6. Effect-TS/effect
**TypeScript framework with functional effect system.** 13.1k stars.

- **What:** Production framework for managing side effects, errors, dependencies.
- **Key patterns:**
  - `Effect<A, E, R>` - computation returning A, failing with E, requiring R
  - Durable workflows with activities and compensation (saga pattern)
  - `DurableClock.sleep()` - pausable execution
  - `DurableDeferred` - async coordination between workflows
  - `WorkflowEngine` - registration, execution, polling, interrupt/resume
- **Relevance:** Could be runtime foundation for contract engine.
  - Activities = execution steps with rollback
  - Compensation = stead's rollback model
  - WorkflowEngine = execution daemon internals

---

### 7. BloopAI/vibe-kanban
**Multi-agent orchestration platform.** 20.2k stars.

- **What:** Supports Claude Code, Codex, Amp, Cursor, Gemini. Worktree isolation per task.
- **Key patterns:**
  - Executor abstraction for agent agnosticism
  - SQLite + React frontend
  - Human review loop with line-by-line diffs
- **Relevance:** Validates worktree isolation at scale. Good executor pattern.
- **Gap:** Still task-centric. Human review is always-on (stead wants verification-first).

---

### 8. ClaytonFarr/ralph-playbook
**Documents the "Ralph" autonomous agent loop methodology.**

- **What:** Dumb bash loop (`while :; do cat PROMPT.md | claude ; done`) with file-based state.
- **Key patterns:**
  - Three phases: Define → Planning Loop → Building Loop
  - Agents maintain state via files between iterations
  - Context management critical (~40-60% utilization)
  - `AGENTS.md` = operational context
- **Relevance:** HIGH. Stead formalizes Ralph's ad-hoc patterns:
  - `IMPLEMENTATION_PLAN.md` → Contract system
  - `PROMPT.md` + `AGENTS.md` → Context generator
  - `loop.sh` → Execution daemon with observability
- **Gap:** No UI, no visibility, manual editing. Stead adds control room.

---

## Medium Relevance

### SuperClaude-Org/SuperClaude_Framework
- Confidence checking before execution
- SelfCheckProtocol against hallucinations
- Token budget allocation by task complexity

### git-ai-project/git-ai
- Git Notes for metadata (non-invasive)
- Attribution tracking through all git operations
- Shows git can be extended without forking

### cloudflare/moltworker
- Gateway pattern: single control plane, multiple execution contexts
- Container sandboxing for safe agent execution
- Device pairing model for auth

---

## Low Relevance

| Repo | Why Low |
|------|---------|
| supermemoryai/claude-supermemory | Cloud RAG, retrieval-focused |
| supermemoryai/supermemory | Same - retrieval, not synthesis |
| vercel-labs/agent-skills | Domain expertise (React rules), not workflow |
| better-auth/skills | Product-specific docs |
| 21st-dev/magic-mcp | UI generation, different problem |

---

## Synthesis: What None of Them Have

| Concept | Best Existing | Gap for Stead |
|---------|--------------|---------------|
| Verification-first | Human review gates | Programmatic verification before completion |
| Rollback as primitive | Effect compensation | First-class compensating actions in contracts |
| Contract vs task | beads issues, vibe-kanban tasks | Typed I/O, not descriptions |
| Context generation | lucid-memory activation | Active synthesis from project state |
| Transformation layer | lix semantic diffs | Compile to git for interop |
| Session proxy | None | Project-scoped identity isolation |

---

## Architecture Implications

### Contract Engine
Build on Effect's workflow primitives:
- Contracts = workflows with payload (input), success (output)
- Activities = atomic steps with verification
- DurableClock for "wait for CI"
- DurableDeferred for "wait for human"

### Execution Daemon
Combine patterns:
- vibe-kanban's executor abstraction
- beads' three-layer sync (SQLite + JSONL + git)
- Worktree isolation per contract

### Control Room
Combine:
- vibe-kanban's UI patterns
- beads' `ready` semantics
- crystal's diff viewer

### Transformation Layer
Extend beads' approach:
- Contracts produce transformations
- Compile to git commits
- Content-addressed for parallel work

### Context Generator
Build on lucid-memory's activation model:
- Recency + frequency + relevance + associations
- Synthesize, don't retrieve

---

## Key Repos to Study Further

1. **Effect-TS/effect** - Workflow primitives for contract engine
2. **steveyegge/beads** - Git-backed state, gates, sync model
3. **opral/lix** - Semantic change patterns (adapt for git compatibility)
4. **JasonDocton/lucid-memory** - Activation model for context generator
