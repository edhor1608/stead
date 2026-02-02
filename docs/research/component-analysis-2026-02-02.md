# Component Analysis

Date: 2026-02-02

Full analysis of each proposed component with positive, negative, and radical takes.

---

## 1. Own Terminal (Ghostty Fork)

### Positive
- Ghostty is already fast, GPU-accelerated, and has clean internals
- A terminal that understands "this session belongs to project X" is huge
- Could expose structured output channels (not just stdout/stderr but agent metadata streams)
- Session multiplexing built for agents, not humans pretending to be multitaskers

### Negative
- Terminal emulation is deceptively complex — years of edge cases
- Ghostty is Zig, which limits contributor pool
- You'd be maintaining two things: terminal compat + agent features
- Most agent work doesn't need a visible terminal at all — it's the wrong abstraction

### Radical Take (Adopted)
What if agents don't need terminals? What if they need a **task execution daemon** with optional terminal visualization? The terminal is a human interface to a shell. Agents don't need the interface — they need the shell. Fork the *shell*, not the terminal.

---

## 2. Own Browser (Helium Fork)

### Positive
- Browser context per project solves tab chaos
- Could have agent-aware automation built in (not Playwright bolted on, but native)
- Auth session isolation per project would be massive

### Negative
- Browsers are the most complex software on earth after operating systems
- Helium is lightweight but that means missing features you'll eventually need
- Web compat issues will eat you alive
- The browser isn't the problem — the *identity* layer is (cookies, sessions, OAuth)

### Radical Take (Adopted)
Don't fork a browser. Build a **session proxy layer** that wraps any browser. Each project gets an isolated identity context (cookies, localStorage, auth tokens) that can be injected into Chrome/Arc/whatever. You get the isolation without the maintenance nightmare.

---

## 3. Task Tracking for Agents

### Positive
- Jira/Linear/Notion are designed for human coordination, not agent execution
- Agents need: dependency graphs, state machines, structured handoff data
- Human-readable task descriptions are actually *harmful* for agents — they need precise specs
- No "story points" or "sprints" — just: preconditions, postconditions, resources, verification

### Negative
- You're building a workflow engine, not a task tracker
- Workflow engines are notoriously hard (see: Temporal, Airflow, every failed startup)
- Risk of over-engineering for hypothetical agent capabilities

### Radical Take (Adopted)
Tasks aren't the right abstraction. **Contracts** are. An agent doesn't need a task — it needs:
- Input specification (what it receives)
- Output specification (what it must produce)
- Verification criteria (how to know it's done)
- Rollback procedure (what to do if it fails)

This is closer to database transactions than project management.

---

## 4. Non-Human-Readable Structure + Human UI

### Positive
- Separating agent data model from human presentation is architecturally clean
- Agents can use optimized formats (binary, graph structures, embeddings)
- Human UI becomes a *view* into agent state, not the source of truth
- Multiple UIs possible: dashboard for oversight, mobile for notifications, etc.

### Negative
- Two systems to maintain (agent backend + human frontend)
- Translation layer can drift or lose fidelity
- Debugging becomes harder when you can't read the source data

### Radical Take (Adopted)
The human UI isn't a dashboard — it's a **control room**. Think air traffic control, not Jira board. You're not managing tasks, you're supervising autonomous systems. The UI should show:
- What's running right now
- What's blocked and why
- What needs human decision
- What finished and needs review
- Resource utilization (tokens, compute, time)

---

## 5. Own Code Organization (Git for Agents)

### Positive
- Git assumes human-paced, human-sized commits
- Agents make thousands of micro-changes — current git can't handle this well
- Branching model was designed for team coordination, not agent parallelism
- Merge conflicts are a human problem — agents could negotiate programmatically

### Negative
- Git is the most entrenched tool in existence
- Any alternative has to be git-compatible or it's dead on arrival
- You'd be fighting every CI/CD system, every hosting platform, every IDE

### Radical Take (Adopted)
Don't replace git. Build a **layer above it**.
- Agents work in a structured workspace (not raw files)
- Changes are expressed as transformations, not diffs
- The layer compiles down to git commits for interop
- Merge conflicts are resolved by re-running the transformation, not manual resolution

Think: git is the "assembly language" of version control. Agents need a higher-level language that compiles to git.

---

## Meta-Analysis

### What This System Actually Is

An **operating environment for agent-driven development**.

Not an OS in the kernel sense, but in the "environment that coordinates resources for workers" sense:
- Workers: agents (Claude, subagents, background processes)
- Resources: compute, code, data, identity, external services
- Coordination: contracts, dependencies, state machines
- Supervision: human control room

### Scope Reality Check

This is:
- Probably 2-3 years of full-time work to get to usable
- Requires mass adoption to be valuable (network effects)
- Competing with every dev tool company adding "AI features"
- Unclear how it makes money

BUT:
- If agents are the future, agent-first infrastructure wins
- No one else is building this from first principles
- Daily pain makes Jonas the ideal user/designer
- It could start small (just the contract engine) and grow

### Why Contract Engine First

1. It doesn't require forking complex software
2. It's immediately useful with existing Claude Code workflows
3. It forces you to define the agent data model
4. The human UI can be built incrementally on top
5. Everything else (terminal, browser, git layer) can integrate later

Minimum viable contract engine:
- Structured contract format agents can consume
- State machine for contract lifecycle
- API for agents to claim/update/complete contracts
- Simple web UI showing current state
- Works with existing terminal/browser/git
