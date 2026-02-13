# Universal Session Format

**Status:** Partially implemented (read adapters complete; write/round-trip/orchestration still proposal)
**Traces to:** Control Room (unified view), Context Generator (sessions as memory)

## Problem

AI coding CLIs (Claude Code, Codex CLI, OpenCode) each store sessions in incompatible formats. This causes:

1. **Vendor lock-in** - Can't switch CLIs mid-project without losing context
2. **Fragmented visibility** - No unified view of all agent work across tools
3. **No cross-CLI workflows** - Can't chain sessions between different CLIs
4. **Duplicate effort** - Each CLI reinvents session management

## Proposed Solution

An abstraction layer that:
1. Defines a canonical **Universal Session Format**
2. Provides **adapters** for each CLI's native format
3. Enables **bidirectional conversion** (import/export)
4. Supports **live orchestration** (spawn CLI, inject session, capture output)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        GUI Layer                            │
│  (Session browser, editor, orchestrator controls)           │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                 Universal Session Format                     │
│  (Canonical representation of any AI coding session)         │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                    Adapter Layer                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │  Claude  │  │  Codex   │  │ OpenCode │  │  Future  │    │
│  │ Adapter  │  │ Adapter  │  │ Adapter  │  │ Adapters │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
└───────┼─────────────┼─────────────┼─────────────┼──────────┘
        │             │             │             │
        ▼             ▼             ▼             ▼
   ~/.claude/    ~/.codex/    ~/.local/share/   ...
                              opencode/
```

## Implementation Note

The schemas below are shown in TypeScript-like pseudo-code for readability. This is documentation, not implementation.

- **Implementation:** Rust structs
- **Interoperability:** The format is language-agnostic; any language can implement adapters using this schema
- **Future:** JSON Schema may be published for formal validation

## Universal Format Schema

```typescript
interface UniversalSession {
  id: string;
  version: '1.0';
  source: {
    cli: 'claude' | 'codex' | 'opencode' | 'universal';
    originalId?: string;  // preserve for round-trip
  };
  project: {
    path: string;
    name?: string;
    git?: {
      branch: string;
      commit: string;
      remote?: string;
    };
  };
  model: {
    provider: string;   // anthropic, openai, deepseek, etc.
    model: string;      // claude-opus-4-5, gpt-4.1, etc.
    config?: Record<string, unknown>;
  };
  timeline: TimelineEntry[];
  metadata: {
    created: string;      // ISO timestamp
    lastModified: string;
    tokens?: { input: number; output: number };
    cost?: number;
  };
}

type TimelineEntry =
  | UserMessage
  | AssistantMessage
  | ToolCall
  | ToolResult
  | SystemMessage;

interface UserMessage {
  id: string;
  timestamp: string;
  type: 'user';
  content: string;
}

interface AssistantMessage {
  id: string;
  timestamp: string;
  type: 'assistant';
  content: string;
  thinking?: string;  // extended thinking if available
}

interface ToolCall {
  id: string;
  timestamp: string;
  type: 'tool_call';
  tool: UniversalTool;
  input: Record<string, unknown>;
  originalTool?: string;  // preserve for round-trip
}

interface ToolResult {
  id: string;
  timestamp: string;
  type: 'tool_result';
  callId: string;  // references ToolCall.id
  success: boolean;
  output: string;
  error?: string;
}

interface SystemMessage {
  id: string;
  timestamp: string;
  type: 'system';
  content: string;
}

// Normalized tool names across CLIs
type UniversalTool =
  | 'read'        // Read file content
  | 'write'       // Write/create file
  | 'edit'        // Edit existing file
  | 'bash'        // Execute shell command
  | 'search'      // Search files (grep/ripgrep)
  | 'glob'        // Find files by pattern
  | 'list'        // List directory
  | 'ask'         // Ask user question
  | 'task'        // Spawn subagent
  | 'web_fetch'   // Fetch URL
  | 'web_search'  // Web search
  | 'unknown';    // Unmapped tool
```

## Tool Mapping

| Universal | Claude Code | Codex CLI | OpenCode |
|-----------|-------------|-----------|----------|
| `read` | `Read` | `read_file` | `read` |
| `write` | `Write` | `write_file` | `write` |
| `edit` | `Edit` | `edit_file` | `edit` |
| `bash` | `Bash` | `shell` | `bash` |
| `search` | `Grep` | `grep` | `grep` |
| `glob` | `Glob` | `glob` | `glob` |
| `list` | `LS` | `ls` | `ls` |
| `ask` | `AskUserQuestion` | `ask` | `ask` |
| `task` | `Task` | `call_agent` | `task` |

## Capabilities

| Capability | Description |
|------------|-------------|
| **Session Browser** | View all sessions across CLIs in one UI |
| **Cross-CLI Resume** | Start in Claude, continue in Codex |
| **Session Forking** | Branch a session, try different approaches |
| **Replay/Debug** | Step through a session, see what happened |
| **Prompt Templates** | Reusable session starters that work everywhere |
| **A/B Testing** | Run same prompt through multiple CLIs, compare |
| **Session Linking** | Chain sessions: output of one → input of next |

## Implementation Phases

### Phase 1: Read-Only Converters
- [ ] Claude Code → Universal parser
- [ ] Codex CLI → Universal parser
- [ ] OpenCode → Universal parser
- [ ] `stead` CLI subcommand: `stead session convert <session> --from claude --to universal`

### Phase 2: Round-Trip Support
- [ ] Universal → Claude Code writer
- [ ] Universal → Codex CLI writer
- [ ] Universal → OpenCode writer
- [ ] Verify: convert A→U→A produces equivalent session

### Phase 3: Session Browser UI
- [ ] List all sessions across CLIs
- [ ] View session timeline
- [ ] Search across sessions
- [ ] Compare sessions side-by-side

### Phase 4: Live Orchestration
- [ ] Spawn CLI subprocess with injected session
- [ ] Capture output in real-time
- [ ] Bidirectional sync during execution
- [ ] Cross-CLI session continuation

## Open Questions

1. **How to handle model-specific responses?**
   - Assistant messages contain model-specific patterns
   - Extended thinking is Claude-only
   - Options: strip, preserve with metadata, or transform

2. **What's the minimal viable session for resume?**
   - Full history? Last N messages? Just user messages?
   - Different CLIs may need different amounts of context

3. **How to handle tool results with binary data?**
   - Screenshots, images, etc.
   - Reference by path? Embed base64? External storage?

4. **Should we persist the universal format?**
   - Could be ephemeral (convert on demand)
   - Or become the source of truth (CLIs read from it)

## Connection to NORTH_STAR

This proposal traces back through:

1. **Control Room** - Unified view of agent state across tools
2. **Context Generator** - Sessions as project memory that persists
3. **Theo's *ding*** - Know which CLI finished, restore context instantly

The Universal Session Format is a foundational layer for the Control Room vision - you can't have unified visibility without unified data.
