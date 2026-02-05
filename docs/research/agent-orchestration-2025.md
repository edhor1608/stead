# AI Agent Orchestration: State of the Art (2025-2026)

Research for stead's positioning in the agent orchestration landscape.

---

## 1. How Teams Currently Orchestrate Multiple AI Coding Agents

The dominant pattern in early 2026 is **manual terminal juggling**. Developers run multiple CLI agents (Claude Code, Codex CLI, OpenCode) in separate terminal tabs/windows and context-switch between them. This is exactly Theo's *ding* problem.

More sophisticated teams use:

- **Git worktrees** -- one agent per worktree, avoiding file conflicts but adding cognitive overhead tracking which worktree is which
- **tmux/screen sessions** -- named sessions per project, but no unified visibility
- **GitHub Issues as dispatch** -- assign issues to Copilot coding agent, which works in background via GitHub Actions
- **IDE agent mode** -- Cursor Composer or Windsurf Cascade for inline multi-file changes, but single-agent at a time

Anthropic's 2026 Agentic Coding Trends Report found that developers use AI in ~60% of their work but can "fully delegate" only 0-20% of tasks. The rest requires active supervision -- confirming stead's "human supervises, agent works" paradigm.

### The Supervision Gap

No tool currently provides a unified "control room" view across multiple agents from different providers. Each tool has its own UI:
- Claude Code: terminal output
- Codex CLI: terminal output
- Copilot coding agent: GitHub PR interface
- Devin: web-based Devin IDE
- Cursor/Windsurf: IDE panels

A developer running Claude Code on project A, Codex on project B, and Copilot on project C has **three completely separate interfaces** with no cross-visibility.

---

## 2. Existing Tools

### Terminal CLI Agents

| Tool | Provider | Model | Key Feature | Limitation |
|------|----------|-------|-------------|------------|
| **Claude Code** | Anthropic | Claude 4.x | Subagent teams, TeammateTool, task system | Terminal-only, no cross-CLI visibility |
| **Codex CLI** | OpenAI | GPT-5/o3 | MCP server mode, Agents SDK integration | Requires OpenAI ecosystem |
| **OpenCode** | Open source | Multi-provider | Go-based TUI, SQLite storage, LSP support | No multi-agent orchestration |
| **Aider** | Open source | Multi-provider | Git-native, pair programming model | Single agent, no background work |

### IDE-Based Agents

| Tool | Key Feature | Multi-Agent? |
|------|-------------|--------------|
| **Cursor** | Composer agent mode, background agents | Parallel background agents (new 2025) |
| **Windsurf** | Cascade flow, Memories persistence | 8 parallel tool calls per turn |
| **Google Antigravity** | Manager View for parallel agents | Yes -- dispatch 5+ agents in parallel |
| **GitHub Copilot** | Coding agent via GitHub Actions | Yes -- multiple issues in parallel |

### Autonomous Agents

| Tool | Key Feature | Limitation |
|------|-------------|------------|
| **Devin** | Full cloud IDE, parallel instances, Devin Wiki | $20+/mo, cloud-only, vendor lock-in |
| **SWE-Agent** | Agent-computer interface (ACI) design | Research-focused, GitHub issue resolver |
| **OpenHands** | Open platform, 66k+ stars, multiple agent types | Platform, not orchestration layer |

---

## 3. Multi-Agent Frameworks

### Claude Code's Built-in Teams

Claude Code has a complete multi-agent orchestration system built in:
- **TeammateTool** -- spawn teams, assign tasks, send messages
- **Task system** -- create/update/list tasks with blocking dependencies
- **SendMessage** -- DMs, broadcasts, shutdown requests
- **Subagent types** -- Explore, Plan, Bash, general-purpose, custom agents

Patterns: leader-worker, sequential handoff, parallel independent work. Workers self-assign from task queue. All agents run in background.

This is the most relevant system for stead because **stead is literally being built using it right now**. But it's confined to a single conversation/session. No persistence across sessions, no cross-CLI coordination.

### Codex CLI + Agents SDK

OpenAI's approach exposes Codex CLI as an MCP server, orchestrated by the Agents SDK:
- `codex()` and `codex-reply()` MCP tools
- Multi-agent workflows: Project Manager, Designer, Frontend Dev, Backend Dev, Tester
- Built-in Traces for observability
- Deterministic, auditable workflows

### General Multi-Agent Frameworks

| Framework | Approach | Maturity | Coding Focus? |
|-----------|----------|----------|---------------|
| **CrewAI** | Role-based crews + event-driven flows | Production | General, not coding-specific |
| **LangGraph** | Graph-based state machines | Production | General |
| **AutoGen** (Microsoft) | Modular multi-agent with layered abstractions | Production | General |
| **Microsoft Agent Framework** | Enterprise SDK combining Semantic Kernel + AutoGen | Early | General |
| **claude-flow** | Claude-specific swarm orchestration | Community | Yes |

Key observation: **None of these frameworks solve the cross-CLI visibility problem.** They all orchestrate agents within their own ecosystem. CrewAI coordinates CrewAI agents. LangGraph coordinates LangGraph agents. Claude Code teams coordinate Claude Code subagents.

There is no "air traffic control" that spans across all of them.

---

## 4. Unsolved Pain Points

### Cross-Tool Visibility
No unified view exists across Claude Code, Codex CLI, Copilot, Cursor, Devin. Each has its own session format, its own state representation, its own notification system. This is the foundational gap stead addresses.

### Session Memory
Agents forget everything between sessions. Developers repeatedly re-explain project context, conventions, and constraints. Claude Code's `CLAUDE.md` and Codex's `AGENTS.md` are static workarounds, not solutions. Anthropic's report confirms this as a top frustration.

### Context Fragmentation
Running 5 agents = 5 terminal windows + N browser tabs + IDE windows. No tool groups these by project. The *ding* problem remains fully unsolved.

### Resource Collisions
Ports, auth redirects, environment variables, database connections -- all unmanaged across concurrent agents. Docker solves isolation but adds massive overhead.

### Verification and Trust
Most frameworks fire-and-forget. Devin's "Artifacts" (screenshots, recordings) and Claude Code's tool approvals are steps toward verification, but there's no contract-based guarantee system. "Did it actually work?" requires manual checking.

### Cost Observability
Running 10+ Claude agents costs ~$2,000/month. No tool provides cross-agent cost tracking or budget controls at the project level.

### Agent-to-Agent Coordination
When Agent A modifies a file that Agent B depends on, there's no conflict detection or resolution. Git handles post-hoc merging, but not real-time coordination.

---

## 5. How Stead Differs

Stead occupies a unique position: **control plane for heterogeneous agent orchestration**.

| Aspect | Existing Tools | Stead |
|--------|---------------|-------|
| **Scope** | Single-vendor ecosystems | Cross-CLI, cross-provider |
| **Visibility** | Per-tool UI | Unified Control Room across all agents |
| **Session format** | Proprietary per tool | Universal Session Format (adapter layer) |
| **Execution** | Build our own runtime | Orchestrate existing CLIs (they're already runtimes) |
| **Task model** | Kanban/issues (human-centric) | Contracts (input/output/verify/rollback) |
| **Attention model** | Everything equal | Priority-based: Needs Decision > Anomalies > Completed > Running > Queued |
| **Architecture** | Cloud services or IDE plugins | Local-first monolith (Rust core + native UIs) |

### Key differentiators:

1. **Universal Session Format** -- adapter layer that reads Claude Code, Codex CLI, and OpenCode sessions into a common format. No other tool does this.

2. **Contract Engine** -- not tasks, but contracts with preconditions, postconditions, and programmatic verification. Closer to database transactions than project management.

3. **Attention-Prioritized Control Room** -- "air traffic control" metaphor. Default state is calm. Screen presence is earned by urgency. This directly solves the *ding* problem by answering "what needs me right now?"

4. **Local-first, no server** -- Rust library shared by CLI and native SwiftUI app. No cloud dependency, no daemon to manage, no "is the server running?" failure mode.

---

## 6. What We Can Learn

### From Claude Code Teams
- Task system with blocking dependencies is effective for coordination
- Subagent types (Explore, Plan, Bash) provide useful specialization
- Leader-worker pattern works well for parallelization
- **Lesson for stead**: The task/contract system should support dependency chains and blocking relationships

### From Codex CLI + MCP
- Exposing agent as MCP server is powerful for integration
- Traces provide essential observability
- **Lesson for stead**: Consider MCP integration for the contract engine -- agents could claim/complete contracts via MCP tools

### From GitHub Copilot
- "Assign an issue to Copilot" is the simplest dispatch UX
- Background execution via Actions eliminates "watching the terminal" problem
- Mission Control for real-time session monitoring
- **Lesson for stead**: The Control Room should be as simple as "assign to agent, get PR back"

### From Google Antigravity
- Manager View for parallel agent dispatch is closest to stead's Control Room vision
- Artifacts (task lists, screenshots, recordings) build trust
- **Lesson for stead**: Contract verification outputs should be rich, not just pass/fail

### From Devin
- Full cloud IDE means total isolation (solves resource collisions)
- Multiple Devin instances in parallel with sub-task dispatch
- Devin Wiki/Search for persistent project knowledge
- **Lesson for stead**: Session proxy / identity isolation is valuable even if we don't build a full IDE

### From CrewAI
- Role-based agents with memory (short-term, long-term, entity, contextual)
- Planning with reflection and structured plan injection
- **Lesson for stead**: The Context Generator concept aligns with CrewAI's memory approach but goes further -- synthesizing rather than storing

### From the Anthropic Report
- Engineers orchestrate, not code -- this IS stead's thesis
- 60% AI usage but only 0-20% fully delegated -- supervision is the bottleneck, confirming Control Room value
- Multi-agent coordination is a "strategic priority for 2026"
- **Lesson for stead**: We're building for a confirmed market need, not a speculative one

---

## Summary

The agent orchestration landscape in 2026 is fragmented by vendor. Every tool orchestrates within its own ecosystem. The unsolved problem is **cross-tool, cross-provider unified visibility and coordination** -- exactly what stead's Control Room + Universal Session Format + Contract Engine addresses.

The timing is right: Anthropic identifies multi-agent coordination as a 2026 strategic priority, developers are already running multiple agents in parallel (poorly), and no tool provides the "air traffic control" view that stead envisions.

---

*Research compiled February 2026. Sources: Anthropic 2026 Agentic Coding Trends Report, Claude Code documentation, OpenAI Codex documentation, GitHub Copilot documentation, Google Antigravity documentation, Cognition (Devin) blog, CrewAI documentation, RedMonk developer surveys, various industry analyses.*
