# stead Architecture Principles

Established: 2026-02-02

## Core Paradigm

**Agents are the primary operators. Humans are supervisors.**

This isn't "tools that help agents" — it's infrastructure where agents do the work and humans oversee.

---

## Architectural Decisions

### 1. Execution Daemon, Not Terminal

**Reject:** Forking a terminal emulator (Ghostty, etc.)

**Instead:** Build a task execution daemon with optional terminal visualization.

**Rationale:** The terminal is a human interface to a shell. Agents don't need the interface — they need the shell. The terminal is a *view* into execution, not the execution itself.

**Implications:**
- Core is a daemon that executes tasks
- Sessions are project-scoped by default
- Structured output channels (not just stdout/stderr — agent metadata streams)
- Terminal UI is one possible frontend, not the primary interface

---

### 2. Session Proxy Layer, Not Browser Fork

**Reject:** Forking a browser (Helium, etc.)

**Instead:** Build a session proxy layer that wraps any browser with isolated identity contexts.

**Rationale:** Browsers are impossibly complex. The actual problem is identity (cookies, localStorage, OAuth sessions). Solve identity isolation without owning the rendering engine.

**Implications:**
- Each project gets an isolated identity context
- Context can be injected into Chrome/Arc/Safari/whatever
- Agent browser automation gets the same identity isolation
- No web compatibility maintenance burden

---

### 3. Contracts, Not Tasks

**Reject:** Task tracking (Jira/Linear model)

**Instead:** Contract-based execution model.

**Rationale:** Agents don't need human-readable task descriptions. They need precise specifications. This is closer to database transactions than project management.

**Contract structure:**
```
contract {
  id: unique identifier

  input: {
    # What the agent receives
    # Typed, validated, complete
  }

  output: {
    # What the agent must produce
    # Typed, verifiable
  }

  verification: {
    # How to know it's done
    # Automated checks, not human approval
  }

  rollback: {
    # What to do if it fails
    # Compensating actions
  }

  dependencies: [contract_ids]
  resources: [what it needs access to]
}
```

**Implications:**
- State machine lifecycle, not kanban columns
- Preconditions/postconditions, not "acceptance criteria"
- Programmatic verification, not human review (unless verification requires human)
- Dependency graphs, not sprint boards

---

### 4. Control Room, Not Dashboard

**Reject:** Project management dashboard (cards, boards, lists)

**Instead:** Control room for supervising autonomous systems.

**Rationale:** You're not managing tasks. You're supervising autonomous agents. Think air traffic control, not Jira.

**Control room shows:**
- What's running right now (live execution state)
- What's blocked and why (dependency/resource issues)
- What needs human decision (escalation queue)
- What finished and needs review (verification results)
- Resource utilization (tokens, compute, time, cost)
- Previews (screenshots, deployment links, diffs)

**Implications:**
- Real-time state, not async updates
- Organized by status/urgency, not by project hierarchy
- Human attention is a scarce resource — UI optimizes for it
- One view across all projects (the whole "stead")

---

### 5. Transformation Layer Above Git

**Reject:** Replacing git or forking git

**Instead:** Build a layer above git that compiles transformations to commits.

**Rationale:** Git is entrenched. Fighting it means fighting every CI/CD, hosting platform, and IDE. But git's model (human-paced, manual merge resolution) doesn't fit agent workflows.

**Approach:**
- Agents work in a structured workspace (not raw files)
- Changes expressed as transformations, not diffs
- Layer compiles down to git commits for interop
- Merge conflicts resolved by re-running transformations, not manual resolution

**Analogy:** Git is the assembly language of version control. Agents need a higher-level language that compiles to git.

**Implications:**
- Full git compatibility (push to GitHub, normal CI/CD)
- Agent-native change representation internally
- Programmatic conflict resolution
- Transformation history alongside commit history

---

## What This System Is

An **operating environment for agent-driven development**.

Not an OS in the kernel sense, but in the "environment that coordinates resources for workers" sense.

- Workers: agents (Claude, subagents, background processes)
- Resources: compute, code, data, identity, external services
- Coordination: contracts, dependencies, state machines
- Supervision: human control room

---

## Build Order

1. **Contract engine** — core execution model, state machine, API
2. **Control room UI** — human supervision interface
3. **Execution daemon** — task runner with structured output
4. **Session proxy** — identity isolation layer
5. **Transformation layer** — git abstraction for agents

Each layer is useful independently. Each integrates with existing tools until replaced.
