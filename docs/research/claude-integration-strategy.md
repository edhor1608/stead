# Claude Code Integration Strategy

**Created:** 2026-02-05
**Author:** Claude Worker (running inside Claude Code on this project)
**Status:** Research

---

## Context

This document is written by a Claude Code agent that is *currently running* inside the stead project. Everything here is grounded in direct observation of how Claude Code works from the inside, combined with inspection of the actual filesystem at `~/.claude/`.

The stead adapter at `rust/stead-core/src/usf/adapters/claude.rs` currently does read-only JSONL parsing. This document proposes how to go deeper.

---

## 1. Claude Code Internal Architecture (What Actually Exists)

### Filesystem Layout (observed on this machine)

```
~/.claude/
  projects/                     # Session JSONL files, keyed by mangled project path
    -Users-jonas-repos-stead/   # One dir per project
      {uuid}.jsonl              # One file per conversation session
      memory/                   # Per-project persistent memory (CLAUDE.md-like)
  teams/                        # Agent team configurations
    stead-marathon/
      config.json               # Team members, roles, models, prompts
  tasks/                        # Task lists for teams
    stead-marathon/             # Task files per team
      {uuid}                    # Individual task JSON files
  session-env/                  # Session environment snapshots
    {session-id}                # Per-session env data
  shell-snapshots/              # Shell state capture (zsh functions, aliases)
    snapshot-zsh-{timestamp}-{id}.sh
  todos/                        # TodoWrite tool output, per-session JSON
    {session-id}-agent-{id}.json
  plugins/
    installed_plugins.json      # Plugin registry with versions
    cache/                      # Downloaded plugin code
  skills/                       # Custom slash commands (.md files)
  commands/                     # Custom slash commands (.md files)
  file-history/                 # File backup snapshots per message
  plans/                        # Plan mode artifacts
  debug/                        # Debug logs
  telemetry/                    # Usage telemetry
  settings.json                 # Global settings (hooks, plugins, env vars)
  statsig/                      # Feature flags
  transcripts/                  # Session transcripts
  paste-cache/                  # Clipboard paste history
  cache/                        # General cache
  downloads/                    # Downloaded files
  ide/                          # IDE integration data
```

### JSONL Entry Format (actual observed structure)

Each line in a session JSONL file contains:

```json
{
  "parentUuid": "uuid-or-null",     // Message threading
  "isSidechain": false,              // Whether this is a branched conversation
  "userType": "external",            // "external" = human, "agent" = subagent
  "cwd": "/Users/jonas/repos/stead", // Working directory at time of message
  "sessionId": "uuid",              // Session identifier
  "version": "2.1.20",             // Claude Code version
  "gitBranch": "main",             // Git branch at message time
  "type": "user|assistant|progress|file-history-snapshot",
  "message": {                      // The actual API message
    "role": "user|assistant",
    "model": "claude-opus-4-6",     // (assistant only)
    "content": [...]                // Content blocks
  },
  "uuid": "message-uuid",
  "timestamp": "ISO-8601",
  "thinkingMetadata": {},           // Extended thinking config
  "toolUseID": "...",              // For tool use tracking
  "parentToolUseID": "..."         // For nested tool calls
}
```

Key fields the current adapter MISSES:
- `version` - Claude Code version (useful for compatibility)
- `parentUuid` / `isSidechain` - conversation branching/threading
- `userType` - distinguishes human from subagent messages
- `type: "progress"` entries with `hook_progress` data
- `type: "file-history-snapshot"` entries with file backup data
- `toolUseID` / `parentToolUseID` - tool call nesting hierarchy

### Team Config Format (actual observed structure)

```json
{
  "name": "stead-marathon",
  "description": "Full-throttle execution...",
  "createdAt": 1770318191003,
  "leadAgentId": "team-lead@stead-marathon",
  "leadSessionId": "session-uuid",
  "members": [
    {
      "agentId": "rust-expert@stead-marathon",
      "name": "rust-expert",
      "agentType": "general-purpose",
      "model": "claude-opus-4-6",
      "prompt": "You are the Rust Expert...",
      "color": "blue",
      "planModeRequired": false,
      "joinedAt": 1770318359889,
      "tmuxPaneId": "in-process",
      "cwd": "/path/to/repo",
      "subscriptions": [],
      "backendType": "in-process"
    }
  ]
}
```

---

## 2. stead as an MCP Server for Claude Code

### The Opportunity

Claude Code connects to MCP servers (Model Context Protocol). I can see MCP tools in my own tool list right now (e.g., `mcp__plugin_playwright_playwright__browser_*`, `mcp__plugin_context7_context7__*`, `mcp__plugin_greptile_greptile__*`). This means Claude Code already has a robust MCP client.

If stead exposes an MCP server, Claude Code agents could directly:
- Query active contracts and their state
- Report progress on contracts they're executing
- Check for resource conflicts before claiming a port
- Read project context from stead's context generator
- Coordinate with agents from other CLIs (Codex, OpenCode) working on the same project

### Proposed MCP Tools

```
stead://contracts/list          -> List active contracts
stead://contracts/get/{id}      -> Get contract details
stead://contracts/claim/{id}    -> Claim a contract for execution
stead://contracts/update/{id}   -> Update contract status/progress
stead://contracts/complete/{id} -> Mark contract complete with output

stead://projects/list           -> List known projects
stead://projects/{path}/context -> Get synthesized project context
stead://projects/{path}/ports   -> Get allocated ports (avoid collision)
stead://projects/{path}/agents  -> List agents working on this project

stead://sessions/register       -> Register this session with stead
stead://sessions/heartbeat      -> Signal "I'm still alive"
```

### Integration Method

Claude Code discovers MCP servers from `.claude/settings.json` or project-level `.claude/settings.json`. The config looks like:

```json
{
  "mcpServers": {
    "stead": {
      "command": "stead",
      "args": ["mcp-serve"],
      "env": {}
    }
  }
}
```

stead would implement the MCP server as a subcommand (`stead mcp-serve`) that speaks the MCP stdio protocol. This is the same pattern used by Context7, Greptile, and other MCP servers visible in my current tool list.

### Key Advantage

With MCP, the agent doesn't just passively get observed by stead -- it actively *participates*. An agent can:
1. Check `stead://projects/{path}/ports` before choosing a port
2. Call `stead://contracts/claim/{id}` to formally claim work
3. Push `stead://sessions/heartbeat` periodically so stead knows it's alive
4. Report structured completion via `stead://contracts/complete/{id}`

This turns stead from a passive observer into an active coordination layer.

---

## 3. Hooks for Automatic Contract Tracking

### How Claude Code Hooks Work

Claude Code has a hook system that triggers shell commands on events. The settings show hooks configured at `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/statusline.sh"
  }
}
```

Hook events observed in the JSONL data:
- `SessionStart` - fires when a new session begins
- Likely also: `PreToolUse`, `PostToolUse`, `Notification` (based on the hooks-handlers directory pattern)

### Proposed Hook Integration

stead could register hooks that automatically:

**On SessionStart:**
```bash
#!/bin/bash
# .claude/hooks/session-start.sh
stead session register --cli claude --session-id "$CLAUDE_SESSION_ID" --cwd "$PWD" --git-branch "$(git branch --show-current 2>/dev/null)"
```

**On PostToolUse (for Bash tool):**
```bash
#!/bin/bash
# Detect port usage in bash commands
if echo "$TOOL_INPUT" | grep -qE '(localhost|127\.0\.0\.1|0\.0\.0\.0):[0-9]+'; then
  PORT=$(echo "$TOOL_INPUT" | grep -oE ':[0-9]+' | head -1 | tr -d ':')
  stead port register --session "$CLAUDE_SESSION_ID" --port "$PORT"
fi
```

**On session completion (via statusLine or notification hook):**
```bash
# Signal to stead that this session is done
stead session complete --session-id "$CLAUDE_SESSION_ID" --status "$EXIT_STATUS"
```

### Limitation

Hook events are not yet fully documented, and the environment variables available inside hooks are not standardized. The safest approach is to start with the MCP server (which is fully under our control) and use hooks as an optional enhancement for users who want zero-config tracking.

---

## 4. Detecting Claude Code Session State

### Current Approach (filesystem polling)

The current adapter reads JSONL files, which gives us:
- Session exists (file present)
- Last activity time (most recent timestamp in JSONL)
- What was happening (last message type and content)

### Better Approach: Active State Detection

**Method 1: Process detection**
```bash
# Claude Code runs as a Node.js process
pgrep -f "claude" | xargs ps -p -o pid,command
```
This tells us if Claude Code is running, but not which session is active.

**Method 2: JSONL tail watching**
Watch for new lines being appended to JSONL files. If a file is being written to, that session is active. If the last write was >60s ago, it's likely idle or waiting for user input.

```rust
// In stead-core, use kqueue/FSEvents to watch
// ~/.claude/projects/*//*.jsonl for modifications
fn detect_session_state(path: &Path) -> SessionState {
    let metadata = fs::metadata(path)?;
    let last_modified = metadata.modified()?;
    let elapsed = SystemTime::now().duration_since(last_modified)?;

    if elapsed < Duration::from_secs(5) {
        SessionState::Active  // Currently executing
    } else if elapsed < Duration::from_secs(60) {
        SessionState::Idle    // Waiting for input or between turns
    } else {
        SessionState::Stale   // Probably finished or abandoned
    }
}
```

**Method 3: MCP heartbeat (best)**
With the MCP server approach, the agent actively pings stead. No heartbeat for 30s = session ended or stuck.

**Method 4: JSONL content analysis**
The last entry in the JSONL tells you state:
- Last entry is `type: "assistant"` with tool calls pending -> agent is executing
- Last entry is `type: "user"` -> waiting for human response (or agent turn started)
- Last entry is `type: "assistant"` with no tool calls -> agent finished, waiting for human
- Last entry has `AskUserQuestion` tool call -> blocked on human decision

### Recommended Approach

Combine Methods 2 + 4 for passive detection, with Method 3 as the premium path for MCP-enabled sessions.

---

## 5. Extractable Metadata from Claude Code Sessions

### Currently Extracted (by the adapter)

| Field | Source |
|-------|--------|
| Session ID | `sessionId` field |
| Working directory | `cwd` field |
| Git branch | `gitBranch` field |
| Model | `message.model` field |
| Timestamps | `timestamp` field |
| Messages | `message.content` blocks |
| Tool calls | `tool_use` content items |
| Tool results | `tool_result` content items |

### Not Yet Extracted (but available)

| Field | Source | Value for stead |
|-------|--------|-----------------|
| Claude Code version | `version` field | Compatibility checking |
| Conversation threading | `parentUuid`, `isSidechain` | Understanding conversation branches |
| Human vs agent messages | `userType` field | Distinguish human prompts from subagent messages |
| File changes | `file-history-snapshot` entries | Which files were modified and when |
| Hook execution | `progress` entries with `hook_progress` | What hooks ran and their status |
| Tool nesting | `parentToolUseID` | Understanding tool call hierarchies (e.g., Task spawning subagents) |
| Active team | `~/.claude/teams/{name}/config.json` | Team structure, member roles, coordination |
| Task list state | `~/.claude/tasks/{name}/` | What tasks exist, their status, who owns them |
| Installed plugins | `~/.claude/plugins/installed_plugins.json` | What MCP tools are available |
| Shell environment | `~/.claude/shell-snapshots/` | Full shell state at session start |
| Todo items | `~/.claude/todos/{session}.json` | Structured task lists the agent created |

### High-Value Extractions for the Control Room

**File change tracking** from `file-history-snapshot` entries:
```json
{
  "type": "file-history-snapshot",
  "snapshot": {
    "trackedFileBackups": {
      "/path/to/file.rs": "backup-content-or-hash"
    },
    "timestamp": "2026-01-27T18:03:29.259Z"
  }
}
```
This gives stead a real-time diff of what the agent changed, without running `git diff`.

**Team awareness** from `~/.claude/teams/`:
When Claude Code is running a team (like the current stead-marathon team with 15+ members), stead can see:
- All team members and their roles
- Which sessions belong to which team members
- The task list and dependencies
- Who is working on what

This maps directly to the Control Room's attention priority system.

**Tool usage patterns** for the Control Room:
By analyzing tool calls in the timeline, stead can compute:
- Read/Write/Edit ratio (is the agent mostly exploring or making changes?)
- Bash command patterns (running tests? building? deploying?)
- How many files touched
- Agent "confidence" signal: lots of Read then Write = deliberate; lots of Edit with errors = struggling

---

## 6. Control Room + Claude Code Tool Usage Patterns

### What the Control Room Could Show

**Per-session tool usage timeline:**
```
[Read] [Read] [Read] [Grep] [Read] [Edit] [Bash:cargo test] [Edit] [Bash:cargo test] PASS
```
This gives a visual "heartbeat" of what the agent is doing. The pattern tells the story:
- Read-heavy = exploring/understanding
- Edit-Bash-Edit-Bash = iterative development
- Long Bash gaps = running slow commands (tests, builds)
- Repeated same-file Edits = struggling with something

**Aggregate dashboard data:**
- Sessions per project per day
- Average tool calls per session
- Most-used tools (weighted by time)
- Error rate (ToolResult with `is_error: true`)
- "Cost" approximation from message count and model

**Anomaly detection:**
- Agent has been running for 20 minutes with no tool calls -> stuck or in long thinking
- Agent has called the same Bash command 5 times -> likely in a retry loop
- Agent is editing files in a project it shouldn't be in -> resource collision

### Implementation Approach

The USF `UniversalTool` enum already maps Claude Code tools. The `TimelineEntry::ToolCall` and `TimelineEntry::ToolResult` pairs give us everything we need. The missing piece is aggregation logic in stead-core and presentation in the Control Room.

---

## 7. Integration with Claude Code's Team Features

### How Teams Work (from the inside)

I am running as a team member right now (`claude-worker` on `stead-marathon`). Here's how it works:

1. **Team config** at `~/.claude/teams/stead-marathon/config.json` lists all members
2. **Each member** has their own session (JSONL file) under the project's session directory
3. **Task coordination** happens through `~/.claude/tasks/stead-marathon/` (JSON files per task)
4. **Messaging** is through the `SendMessage` tool (DMs and broadcasts)
5. **Shutdown** is coordinated through `shutdown_request`/`shutdown_response` protocol

### What stead Could Do With Teams

**Passive observation:**
- Read team config to understand the team structure
- Map each member's `sessionId` to their JSONL file
- Track task progress through the task directory
- Build a team activity timeline (who did what, when)

**Active coordination via MCP:**
If stead is an MCP server, team members could query it for:
- Cross-team resource allocation (ports, file locks)
- Contract status from stead's perspective (not just Claude Code's tasks)
- Other CLI agents working on the same project (Codex, OpenCode)

**Control Room team view:**
The Control Room could show a team as a single "meta-session" that expands to show individual members. The attention priority applies per-member:
- Team lead waiting for input -> NEEDS DECISION
- Member completed task -> COMPLETED (roll up to team status)
- All members idle -> team is done

### Cross-CLI Team Coordination

This is stead's killer feature. Claude Code's team system only coordinates Claude Code agents. But a project might have:
- Claude Code team (3 agents) doing feature work
- Codex CLI agent doing documentation
- OpenCode agent doing infrastructure

stead sees ALL of them through USF adapters. The Control Room shows the unified view.

---

## 8. What Would Make Claude Code Better at Working with stead

### A. CLAUDE.md Section for stead Conventions

Yes. If a project uses stead, its `.claude/CLAUDE.md` (or project `CLAUDE.md`) should include:

```markdown
## stead Integration

This project uses stead for agent coordination.

### Conventions
- Before starting work, check for active contracts: `stead list --running`
- Before using a port, check allocation: `stead port check <port>`
- When done with a task, mark the contract: `stead complete <id>`
- Use `stead run` to create contracts, not ad-hoc task descriptions

### MCP Server
stead exposes an MCP server. Available tools:
- `stead://contracts/*` - Contract CRUD
- `stead://projects/*/ports` - Port allocation
- `stead://sessions/*` - Session coordination

### Resource Rules
- Never hardcode ports. Use `stead port allocate` or check `.stead/ports.json`
- Never modify files under `.stead/` directly -- use the CLI or MCP tools
- If verification fails, do NOT manually mark contracts as passed
```

This turns stead conventions into constraints that shape the agent's behavior, which is exactly what NORTH_STAR.md describes as the goal of project memory: "Decisions aren't stored as facts. They become constraints."

### B. stead MCP Server for Claude to Query Contracts

Absolutely. Here's what the MCP server should expose:

**Resources (read-only context):**
```
stead://project/status          # Overall project health
stead://contracts/active        # Currently active contracts
stead://contracts/{id}          # Specific contract detail
stead://ports/allocated         # All allocated ports
stead://sessions/active         # All active agent sessions (all CLIs)
```

**Tools (actions):**
```
stead_claim_contract(id)        # Claim a contract for execution
stead_update_progress(id, msg)  # Push progress update
stead_complete_contract(id, output) # Mark contract done
stead_allocate_port(project)    # Get a non-conflicting port
stead_register_session()        # Register this session
stead_request_decision(question, options) # Ask human via Control Room
```

The `stead_request_decision` tool is particularly powerful. Instead of using `AskUserQuestion` (which interrupts in the terminal), the agent pushes the question to the Control Room where the human can answer it on their own schedule. This directly solves the *ding* problem.

### C. stead Plugin for Claude Code

Rather than requiring manual CLAUDE.md additions, stead could ship as a Claude Code plugin:

```json
// ~/.claude/plugins/installed_plugins.json
{
  "stead@stead-plugins": [{
    "scope": "user",
    "installPath": "~/.claude/plugins/cache/stead-plugins/stead/1.0.0",
    "version": "1.0.0"
  }]
}
```

A plugin can provide:
- MCP server auto-configuration
- Hooks for session start/end
- Custom slash commands (`/stead-status`, `/stead-claim`)
- System prompt additions (conventions injected automatically)

This is the zero-config path. Install the plugin, and every Claude Code session automatically coordinates through stead.

---

## 9. Implementation Priority

### Phase 1: Enhanced Passive Adapter (Low effort, immediate value)

Enhance `claude.rs` to extract:
- `version` field (Claude Code version tracking)
- `userType` field (human vs subagent distinction)
- `file-history-snapshot` entries (file change tracking)
- `parentUuid` threading (conversation branch understanding)
- Team config reading from `~/.claude/teams/`
- Task list reading from `~/.claude/tasks/`

This requires only changes to the existing adapter code. No new infrastructure.

### Phase 2: MCP Server (Medium effort, high value)

Implement `stead mcp-serve` subcommand:
- MCP stdio protocol implementation
- Contract query/update tools
- Port allocation tool
- Session registration and heartbeat

This makes stead an active participant in agent workflows, not just an observer.

### Phase 3: Claude Code Plugin (Medium effort, zero-config UX)

Package Phase 2 as a Claude Code plugin:
- Auto-registers MCP server
- Injects stead conventions into system prompt
- Adds hooks for session lifecycle
- Provides `/stead-*` slash commands

### Phase 4: Team-Aware Control Room (High effort, differentiating feature)

The Control Room understands Claude Code teams:
- Team members shown as grouped entries
- Task dependency visualization
- Cross-CLI coordination (Claude team + Codex agent + OpenCode agent)
- Unified attention priority across all agents, all CLIs

---

## 10. What the Current Adapter Gets Wrong (or Misses)

### Structural Issues

1. **Missing entry types:** The adapter only handles `user` and `assistant` message types. It skips `progress` entries (hook execution, status updates) and `file-history-snapshot` entries (file change tracking). These contain valuable state information.

2. **No conversation threading:** The `parentUuid` and `isSidechain` fields are ignored. This means stead can't distinguish a main conversation from a side-chain (branched conversation), which matters for understanding what the agent actually did.

3. **No team awareness:** The adapter has no concept of teams. It treats every session independently. But in a team scenario (like right now), the team config at `~/.claude/teams/` provides crucial context about how sessions relate.

4. **Session directory structure:** The adapter iterates `~/.claude/projects/*/` for JSONL files. But the project directory names are mangled paths (e.g., `-Users-jonas-repos-stead`). The adapter could reconstruct the actual project path from the directory name by reversing the mangling (replace leading `-` with `/`, replace `-` with `/`). This would allow matching sessions to projects even without opening each file.

5. **The `version` field is gold:** Knowing which Claude Code version produced a session lets stead handle format changes gracefully. The JSONL format has evolved (visible in the `version: "2.1.20"` field). Future-proofing the adapter means checking this.

### Data Freshness

The current adapter reads files synchronously on demand. For the Control Room, stead needs:
- File system watcher on `~/.claude/projects/` for new sessions
- Tail-follow on active JSONL files for real-time updates
- Periodic scan of `~/.claude/teams/` for team changes

On macOS, `FSEvents` (via the `notify` crate) is the right tool.

---

## Summary

The integration strategy has four layers:

1. **Passive adapter enhancement** -- Extract more metadata from what already exists in the JSONL files and surrounding directories. Low risk, immediate value.

2. **MCP server** -- Let Claude Code agents actively coordinate through stead. Solves port conflicts, enables cross-CLI awareness, enables structured decision requests. This is the highest-impact single feature.

3. **Claude Code plugin** -- Zero-config packaging of the MCP server + hooks + conventions. Makes stead adoption frictionless for Claude Code users.

4. **Team integration** -- Understand and visualize Claude Code's team/subagent system in the Control Room. This is where stead's cross-CLI unified view becomes a true differentiator.

Each layer builds on the previous. Start with 1, which is just code changes to the existing adapter. Then 2, which requires a new `mcp-serve` subcommand. Then 3 and 4, which require the Control Room to be functional.

The key insight: Claude Code already has all the infrastructure stead needs (MCP, hooks, plugins, structured session data). stead doesn't need to build new protocols -- it needs to plug into what's already there.
