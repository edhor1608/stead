# The Parallel Project Problem

## Origin

Analysis sparked by [Theo's article](https://x.com/theo/article/2018091358251372601) on 2026-02-02.

## The Problem Statement

When working on multiple projects simultaneously (especially with AI agent workflows), our tools break down because:

1. **Apps organize by app, not by project** - Terminal, browser, IDE are separate. Finding "everything for Project A" requires hunting across all of them.

2. **Mental model collapse** - Single project work: apps are grouped mentally as "within Project A". Multi-project: projects are split BETWEEN apps with no natural grouping.

3. **Interrupt-driven context loss** - Agent completion notifications (*ding*) grab attention just long enough to lose track of current work.

4. **Resource collisions** - Port conflicts (everything wants :3000), auth redirect breakage, etc.

## Jonas's Reality (2026-02-02)

Active repos in ~/repos with commits in last 24 hours:
- `meinungsmache-app` - rebranding
- `qwer-q` - memory/protocol fixes (3 commits)
- `thinking-loop` - v11→v12 iterations

Active in last week:
- `picalyze` - enterprise features
- `create-edhor-stack` - release workflow

**5+ projects with meaningful work in a single week.**

## Core Insight

The cognitive cost isn't switching - it's **maintaining multiple contexts simultaneously** while being interrupted by the very tools helping you parallelize.

Traditional flow:
```
Focus → Context → Work → Switch (full reset)
```

New reality:
```
Focus → Context → Work → *ding* → Partial switch → Where was I? → *ding*
```

## What's Actually Broken

### 1. No project-as-entity abstraction in OSes
macOS/Windows/Linux organize by app, not by project. Mission Control can't show "everything for qwer-q."

### 2. Agents are context-free
When Claude Code finishes, it has no way to pull your attention WITH context. You get a ding, not a summary + state restoration.

### 3. Port collisions are a symptom
Dev servers assume they're the only thing running. `localhost:3000` isn't namespaced to project.

### 4. Browser tabs are project-hostile
47 tabs open. Which 8 are for picalyze? Nobody knows.

## Non-Solutions (per Theo, confirmed)

| Approach | Why it fails |
|----------|--------------|
| tmux | Helps terminal, adds another thing to context-switch into |
| Agent orchestration GUIs | Same as above |
| IDE with built-in browser | Doesn't solve github, auth, external services |
| Docker | Solves ports, adds massive overhead, doesn't solve browser/IDE |
| Background agents | Makes it worse - invisible work completing |

## Potential Solution Directions

### 1. Project-scoped workspaces at OS level
- Terminal sessions tagged to project
- Browser profiles per project (not just work/personal)
- One keybind to surface everything for "picalyze"

### 2. Agent handoffs with context
When agent finishes: "I finished X, here's where you left off, here's what changed" - not just a ding.

### 3. Explicit attention management
Track which project you're "in", queue notifications appropriately.

### 4. Port namespacing
Every project gets its own localhost subdomain automatically (picalyze.local:3000).

## The Hard Truth

The real solution requires coordination nobody will do:
- Apple won't rebuild macOS around projects
- Chrome won't add project-aware tab groups
- VS Code workspaces are close but don't extend to browser/terminal

## What Might Actually Be Buildable

A **project orchestrator** that:
- Owns port allocation
- Manages terminal sessions
- Groups browser tabs (via extension)
- Routes agent notifications with context
- Provides one place to see "what's running for each project"

Serious engineering effort. Unclear monetization.

## Tactical Bandaids (for now)

1. **Dedicated browser profile per active project** - preserves context
2. **Explicit port ranges** - picalyze: 3100-3199, qwer-q: 3200-3299
3. **tmux sessions named by project** - `tmux attach -t picalyze`
4. **Agent output to file** - check on YOUR schedule, not interrupt-driven

---

## Open Questions

- What's the minimum viable version of "project orchestrator"?
- Can this be solved with conventions alone, or does it require tooling?
- Is the problem severe enough that people would pay for a solution?
- What existing tools come closest?
