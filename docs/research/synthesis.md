# stead Research Synthesis

**Date:** 2026-02-02
**Status:** Research complete, ready for implementation planning

---

## Executive Summary

stead is an operating environment for agent-driven development. Agents are the primary operators; humans are supervisors. This document synthesizes findings from 9 parallel research threads.

**Core insight confirmed:** The problem isn't "AI-assisted development" — it's that our entire toolchain assumes humans are the workers. Flipping this paradigm requires new infrastructure, but not new inventions. The pieces exist; nobody has assembled them for agent-first workflows.

---

## The Five Pillars

### 1. Contract Engine (Foundation)

**What:** Replace task tracking with contract-based execution.

**Key Design Decisions:**
- Contracts specify input/output/verification/rollback — not descriptions
- 10-state machine: `pending → ready → claimed → executing → verifying → completed`
- Dependencies: `blocks`, `requires_output`, `resource_conflict`
- Three-stage verification: schema → automated checks → human review (if needed)

**Build On:**
- XState v5 for state machines (zero deps, battle-tested)
- SQLite for local-first persistence
- Playwright's `storageState` JSON as inspiration for contract state format

**MVP Scope (6 weeks):**
- Core CRUD + lifecycle API
- Single-agent claiming
- Automated verification only
- No rollback, no human review queue

**Gap Filled:** No existing tool combines local-first durable execution + agent task contracts + desktop integration.

---

### 2. Execution Daemon

**What:** Task execution layer that replaces terminal-as-interface.

**Key Design Decisions:**
- Unix socket + JSON-RPC 2.0 (same pattern as LSP/MCP)
- Project-scoped sessions with isolated env + auto port allocation
- Structured output channels beyond stdout/stderr (JSON, metrics, telemetry)
- Terminal is a *view* into execution, not the foundation

**Build On:**
- pueue patterns (persistent daemon, task dependencies, crash survival)
- mise approach (no shims, direct PATH modification)
- Nushell concepts (structured data over text streams)
- MCP as the agent interface standard

**Hard Problems:**
- PTY compatibility (tools behave differently without TTY)
- <10ms overhead target
- Shell state persistence across commands

**Integration Points:**
- Claude Code: delegate Bash tool execution to daemon
- MCP Server: expose as tools any agent can call
- Terminal Bridge: human-facing shell routing through daemon

---

### 3. Session Proxy

**What:** Identity isolation without forking a browser.

**Key Design Decisions:**
- Hybrid architecture: daemon + Firefox extension + CDP controller + CLI
- Firefox-first (native `contextualIdentities` API exists)
- Playwright's `storageState` format as canonical state representation
- OAuth via subdomain/port namespacing per project

**Build On:**
- Firefox Multi-Account Containers (reference implementation)
- Playwright BrowserContext (agent automation)
- configurable-http-proxy (REST API for dynamic routing)

**Hard Problems:**
- Chrome lacks equivalent API — requires heavy workarounds
- Sync conflicts between human and agent browser use
- Cross-origin leakage in iframes

**MVP Scope:**
- Week 1-2: Agent-only library using storageState
- Week 3-4: Firefox extension + CLI switching
- Week 5-8: Full daemon integration with locking

---

### 4. Control Room

**What:** Human supervision interface — air traffic control, not Jira.

**Key Design Decisions:**
- Organize by attention priority, not by project
- `Needs Decision > Anomalies > Completed > Running > Queued`
- Keyboard-first interaction
- Default state is calm/empty — earn screen presence

**Build On:**
- Tauri for desktop app (small footprint, system tray, global hotkey)
- SSE over WebSockets (simpler, sufficient for server→client)
- Research: breaking attention at task boundaries reduces cognitive load 46%

**Core Views:**
- Decision queue (structured escalation items)
- Resource utilization (tokens, compute, time, cost)
- Previews (screenshots, deployment links, diffs)
- Cross-project by default, focus mode for deep work

**Hard Problems:**
- Distinguishing normal activity from required attention
- "I was gone for a day" catch-up problem
- Scale (works for 5 projects, but 50?)

---

### 5. Transformation Layer

**What:** Semantic changes that compile to git.

**Key Design Decisions:**
- Transformations capture intent: `rename(function, old, new)` not line diffs
- Dual representation: semantic for replay, implementation for git
- Conflict resolution via transformation replay, not patch merge
- Git remains the backend — full GitHub/CI compatibility

**Build On:**
- Jujutsu patterns (git as storage, different UX on top)
- Pijul concepts (commutative patches, order-independent)
- Comby for structural code transformation language
- ts-morph/LibCST for AST transformations that preserve formatting

**Hard Problems:**
- Language diversity (start TypeScript, tree-sitter for others)
- Platform boundary crossings (import external changes as raw patches)
- Transform composition complexity

**Key Insight:** "Start from merge, extract branches" (GitButler pattern) may match agent workflow better than traditional branching.

---

## Technology Stack Recommendations

| Layer | Technology | Rationale |
|-------|------------|-----------|
| State machines | XState v5 | Zero deps, TypeScript, battle-tested |
| Persistence | SQLite | Local-first, embedded, fast |
| Daemon IPC | Unix socket + JSON-RPC | Same as LSP/MCP, proven pattern |
| Desktop UI | Tauri | Small footprint, Rust backend, web frontend |
| Real-time | SSE | Simpler than WebSockets for server→client |
| Browser automation | Playwright | Best context isolation, agent-ready |
| Browser extension | Firefox WebExtensions | Native container API |
| Code transforms | tree-sitter + ts-morph | Language-agnostic parsing + TS-specific transforms |
| Git abstraction | Jujutsu patterns | Proven git-compatible approach |

---

## Build Order

```
Phase 1: Contract Engine (Weeks 1-6)
├── Core schema and state machine
├── SQLite persistence
├── REST API
├── Single-agent integration
└── Automated verification

Phase 2: Control Room MVP (Weeks 7-10)
├── Tauri app shell
├── Contract visualization
├── Decision queue
└── Basic notifications

Phase 3: Execution Daemon (Weeks 11-14)
├── Daemon architecture
├── Project-scoped sessions
├── Port allocation
└── Claude Code integration

Phase 4: Session Proxy (Weeks 15-18)
├── Agent library (storageState)
├── Firefox extension
├── CLI context switching
└── Control room integration

Phase 5: Transformation Layer (Weeks 19-24)
├── TypeScript PoC
├── Basic transforms
├── Git compilation
└── Conflict replay
```

**Total estimated timeline:** 6 months to functional system

---

## Critical Gaps Stead Fills

| Gap | Current State | Stead Solution |
|-----|---------------|----------------|
| Local-first durable execution | Everything assumes servers | SQLite-backed contract engine |
| Agent task contracts | No pre/postcondition definitions | Contract schema with verification |
| Desktop integration | All tools server/cloud-focused | Tauri control room |
| Unified workspace model | Agents, terminals, browsers separate | Single project context across all |
| Project-scoped identity | Manual profile switching | Session proxy with auto-switching |
| Semantic version control | Line-based diffs lose intent | Transformation layer |

---

## Open Decisions

### Immediate (Before Phase 1)

1. **Contract schema format** — JSON Schema? TypeScript types? Both?
2. **Agent SDK language** — TypeScript first? Multi-language from start?
3. **Naming** — "stead" final? What does it mean?

### Deferred (Decide During Build)

1. **Firefox-first or browser-agnostic** — for session proxy
2. **Human review UX** — how decisions appear in control room
3. **Multi-agent coordination** — how contracts are distributed

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Scope creep | High | High | Strict MVP boundaries, defer features |
| Integration complexity | Medium | High | Build integration points early, test continuously |
| Agent capability assumptions | Medium | Medium | Design for current Claude Code, adapt as agents improve |
| Adoption barrier | High | Medium | Transparent mode (works with existing git/terminal) |
| Maintenance burden | Medium | High | Use existing libraries (XState, Playwright, tree-sitter) |

---

## Next Steps

1. **Decide open questions** — schema format, SDK language, naming
2. **Create detailed Phase 1 spec** — contract engine MVP
3. **Set up project structure** — monorepo, packages, CI
4. **Build contract engine core** — schema, state machine, persistence
5. **Integrate with Claude Code** — prove the model works

---

## Appendix: Research Documents

| Document | Focus |
|----------|-------|
| `problem-analysis.md` | Original problem breakdown |
| `component-analysis-2026-02-02.md` | Full positive/negative/radical analysis |
| `architecture-principles.md` | Core architectural decisions |
| `exploration-contract-engine.md` | Contract schema, state machine, MVP |
| `exploration-execution-daemon.md` | Daemon architecture, protocols |
| `exploration-session-proxy.md` | Identity isolation, Firefox-first |
| `exploration-control-room.md` | UI/UX, attention design |
| `exploration-transformation-layer.md` | Semantic diffs, git interop |
| `github-research-workflow-engines.md` | XState, Restate, patterns |
| `github-research-browser-isolation.md` | Playwright, Firefox containers |
| `github-research-dev-environments.md` | pueue, mise, Nushell |
| `github-research-git-abstractions.md` | Jujutsu, Pijul, Comby |
