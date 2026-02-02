# North Star

**Check this document before making any architectural or implementation decision.**

This document captures the full chain: Problem → Vision → Reframings → Principles

---

## Layer 1: The Original Pain (Theo's *ding*)

Source: [Theo's article](https://x.com/theo/article/2018091358251372601)

```
*ding*
You hear a notification from a Claude Code workflow finishing.
Which terminal tab was it?
Hop around terminal windows and tabs, finally find it. Project B.
Which browser was that in...
Oh, localhost:3001 — auth redirects broken.
Which terminal is using :3000?
*ding*
Another workflow finished. Briefly grabs attention — just long enough to lose track.
```

**The core problems:**

1. **Context fragmentation** — Projects split across terminal/browser/IDE with no grouping
2. **Interrupt without restoration** — *ding* grabs attention but doesn't restore context
3. **Resource collisions** — Ports, auth redirects, sessions conflict between projects
4. **Invisible agent state** — What's running? What finished? What needs me?

**Theo's non-solutions (confirmed):**
- tmux — Helps terminal, adds another thing to context-switch into
- Agent orchestration GUIs — Same problem, different app
- IDE with built-in browser — Doesn't solve github, auth, external services
- Docker — Solves ports, adds massive overhead
- Background agents — Makes it worse: invisible work completing

---

## Layer 2: Jonas's Vision (The Bullet Points)

Response to Theo's problem — what if we built from scratch for this reality?

- **Own terminal** — not constrained by terminal-as-human-interface assumptions
- **Own browser** — project-scoped identity, not tab chaos
- **Task tracking for agents, not human teams** — not Jira/Linear designed for human coordination
- **Built for subagents, multi-agents, background agents, async/overnight work, ralph-loops**
- **Non-human-readable structure underneath** — optimize for agent consumption
- **Human UI for tracking state, reviews, previews** — separate presentation layer
- **Own code organization (git for agents)** — not human-paced, human-sized commits
- **Project memory that persists** — knowledge survives sessions, not just conversation context
- **Optimized for agents under the hood, optimized for humans in the frontend**

**The paradigm shift:**

> **Old model:** Human works, agent assists.
> **New model:** Agent works, human supervises.

---

## Layer 3: The Reframings (Radical Takes)

Each bullet point challenged and reframed to solve the actual problem without inheriting maintenance burden.

### Own Terminal → Execution Daemon

**Original idea:** Fork Ghostty, add agent features.

**Problem with that:** Terminal emulation is deceptively complex. Years of edge cases. Ghostty is Zig (limited contributors). Most agent work doesn't need a visible terminal at all.

**Reframing:** Agents don't need terminals — they need a task execution daemon with optional terminal visualization. The terminal is a human interface to a shell. Agents need the shell, not the interface.

**What this means:**
- Fork the *shell*, not the terminal
- Daemon with structured output channels (not just stdout/stderr)
- Project-scoped sessions with isolated env, auto port allocation
- Terminal becomes a *view* into execution, not the foundation

---

### Own Browser → Session Proxy

**Original idea:** Fork Helium, add project-scoped contexts.

**Problem with that:** Browsers are the most complex software after operating systems. Web compat issues will eat you alive. The browser isn't the problem — the *identity* layer is.

**Reframing:** Don't fork a browser. Build a session proxy layer that wraps any browser with isolated identity contexts. Each project gets isolated cookies, localStorage, auth tokens.

**What this means:**
- Solve identity isolation without owning the rendering engine
- Context can be injected into Chrome/Arc/Firefox/whatever
- Agent automation gets the same isolation
- No web compatibility maintenance burden

---

### Task Tracking for Agents → Contracts

**Original idea:** Build task tracking designed for agents, not human teams.

**Problem with that:** You're building a workflow engine, not a task tracker. "Task" is the wrong abstraction — human-readable descriptions are actually harmful for agents.

**Reframing:** Tasks aren't the right abstraction. Contracts are. An agent needs:
- Input specification (what it receives)
- Output specification (what it must produce)
- Verification criteria (how to know it's done)
- Rollback procedure (what to do if it fails)

**What this means:**
- Closer to database transactions than project management
- Preconditions/postconditions, not "acceptance criteria"
- Programmatic verification, not human review (unless required)
- State machines, not kanban columns

---

### Human UI → Control Room

**Original idea:** Dashboard for humans to track state, reviews, previews.

**Problem with that:** Dashboards are for managing work. You're not managing — you're supervising autonomous systems.

**Reframing:** The UI is a control room, not a dashboard. Think air traffic control, not Jira board.

**What this means:**
- Organize by attention priority, not by project
- `Needs Decision > Anomalies > Completed > Running > Queued`
- Default state is calm/empty — earn screen presence
- One view across all projects (the whole "stead")
- Human attention is scarce — optimize for it

---

### Git for Agents → Transformation Layer

**Original idea:** Replace git with something designed for agent workflows.

**Problem with that:** Git is the most entrenched tool in existence. Any alternative must be git-compatible or it's dead on arrival.

**Reframing:** Don't replace git. Build a layer above it. Agents work in structured workspaces. Changes are transformations, not diffs. The layer compiles down to git commits.

**What this means:**
- Git is the "assembly language" — agents need higher-level language
- Transformations capture intent: `rename(function, old, new)` not line diffs
- Merge conflicts resolved by re-running transformations
- Full GitHub/CI compatibility maintained

---

### Project Memory → Context Generator

**Original idea:** Build a persistent memory/knowledge store for the project that agents can query — decisions, patterns, history that survives sessions.

**Problem with that:** Memory systems are notoriously hard. RAG is lossy. Knowledge graphs are rigid. Conversation history blows tokens. All treat memory as a retrieval problem — store facts, query facts. But that's not how humans use project knowledge.

**Reframing:** Don't build a memory store. Build a context generator.

The project already HAS memory — code, docs, git history, contracts, decisions. The problem isn't storage, it's synthesis. A "project mind" isn't a database you query. It's a process that generates relevant context for the current task.

**What this means:**
- No separate memory system to maintain
- Context generator synthesizes what matters from everything that exists
- Task-specific briefings, not one-size-fits-all docs
- Decisions become constraints that shape behavior, not facts to retrieve
- Memory is alive — a lens through which agents see the project, not a log they search

**The frame:**
> Decisions aren't stored as facts. They become constraints.
> History isn't a log. It's patterns that influence choices.
> Memory isn't retrieved. It's embodied in the agent's starting state.

---

## Decision Filter

Before ANY decision (tech choice, architecture, feature), ask:

### 1. Does this reduce the *ding* problem?
- Does it help find context faster?
- Does it prevent resource collisions?
- Does it restore state when attention shifts?

### 2. Is this agent-first?
- Is the agent the primary user of this component?
- Is the human interface a VIEW into agent state, not the source of truth?

### 3. Is this the simplest solution?
- Are we adding complexity because it's interesting, or because it solves the pain?
- Could a convention solve this instead of code?
- Could we use an existing tool instead of building?

### 4. Does this trace back?
- Can we connect this to one of the reframings above?
- Can we connect that reframing to Jonas's bullet points?
- Can we connect those bullet points to Theo's *ding*?

If the chain breaks, question the decision.

---

## What Success Looks Like

```
*ding*
Notification: "qwer-q: memory fix complete. 3 files changed. Tests pass."
One click/keystroke → full context restored:
  - Terminal session for qwer-q
  - Browser with qwer-q tabs (already authenticated)
  - IDE at the right files
  - Port 3200 (auto-assigned, no collision)
You review, approve, continue — or dismiss and stay in current project.
```

That's it. That's the goal. Everything else is means to this end.

---

## Anti-Patterns to Avoid

- **Research rabbit holes** — Interesting findings that don't connect to the pain
- **Technology tourism** — Picking tools because they're cool, not because they solve the problem
- **Premature abstraction** — Building for hypothetical future needs
- **Scope expansion** — "While we're at it, let's also..."
- **Decision momentum** — Making choices because we're in decision-making mode
- **Losing the chain** — Decisions that can't trace back through the layers above

---

## The Mantra

**When in doubt, return to the *ding*.**

Then trace forward: *ding* → bullet points → reframings → does your decision fit?
