# Brainstorm: Agent-First Development Stack

Date: 2026-02-02

## Jonas's Initial Vision

- Own terminal (Ghostty fork)
- Own browser (Helium fork)
- Task and project tracking made for AI agents, not for managing human teams
- Totally built for subagents, multi-agents, background agents, async work, overnight work, ralph-loops
- Task/project tracking can be an underlying non-human-readable structure
- Extra UI for humans to track projects, state, done work, planned steps, review queues, previews
- Own code organization (git for agents)
- Optimized for agents under the hood, optimized for humans in the frontend

## Core Insight

Flip the paradigm: **Agents are the primary users of the dev stack. Humans are supervisors with a dashboard.**

---

## Analysis

Discussion led to radical reframings of each component. All adopted.

See: [Architecture Principles](../plans/architecture-principles.md)

### Summary of Reframings

| Original Idea | Reframed To |
|---------------|-------------|
| Own terminal (Ghostty fork) | Execution daemon — agents need the shell, not the terminal UI |
| Own browser (Helium fork) | Session proxy — solve identity isolation without owning a browser |
| Task tracking for agents | Contracts — input/output/verification/rollback, not descriptions |
| Non-human-readable + human UI | Control room — supervise autonomous systems, not manage tasks |
| Git for agents | Transformation layer — compile to git, don't replace it |

### Key Insight Adopted

**Agents are the primary operators. Humans are supervisors.**

This changes everything about tool design.
