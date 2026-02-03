# AI CLI Session Format Research

Research into how major AI coding CLIs store session data, and feasibility of cross-CLI interoperability.

## Session Storage Locations

| CLI | Location | Format |
|-----|----------|--------|
| Claude Code | `~/.claude/` | JSONL streams |
| Codex CLI | `~/.codex/` | JSONL streams + SQLite |
| OpenCode | `~/.local/share/opencode/` | Normalized JSON files |

## Claude Code

**Structure:**
```
~/.claude/
├── projects/           # Per-project session data (by directory path)
├── transcripts/        # Full conversation transcripts as .jsonl
├── history.jsonl       # Command history across sessions
├── tasks/              # Task list state
├── plans/              # Saved plans
└── file-history/       # File edit history
```

**Session Format:** JSONL stream with entries like:
```json
{
  "type": "user",
  "timestamp": "2026-01-04T20:01:19.120Z",
  "content": "message text"
}
```

**Tool calls:** Inline in stream as `tool_use` / `tool_result` pairs.

## Codex CLI

**Structure:**
```
~/.codex/
├── sessions/           # Conversation logs by year (.json files)
├── sqlite/codex-dev.db # SQLite database for persistent state
├── prompts/            # Custom prompts
├── skills/             # Custom skills
├── auth.json           # API credentials
├── config.toml         # Configuration
├── AGENTS.md           # Project instructions
└── models_cache.json   # Cached model metadata
```

**Session Format:** JSONL stream with wrapper:
```json
{
  "timestamp": "2026-01-04T18:54:29.163Z",
  "type": "session_meta",
  "payload": { ... }
}
```

**Tool calls:** OpenAI function calling format inline.

## OpenCode

**Structure:**
```
~/.local/share/opencode/storage/
├── session/    # Session metadata (by project hash)
├── message/    # Messages organized by session ID
├── part/       # Tool calls/results stored separately per message
└── project/    # Project configurations
```

**Session Format:** Normalized relational structure:
- Each message is a separate JSON file
- Tool calls stored in `part/` directory, linked by message ID

```json
{
  "id": "msg_c0afd53360012AbThPmdmFRr91",
  "sessionID": "ses_3f502acd1ffewLJzmXP1QvC5Vg",
  "role": "user",
  "summary": "message preview..."
}
```

## Format Comparison

| Aspect | Claude Code | Codex CLI | OpenCode |
|--------|-------------|-----------|----------|
| Storage | JSONL stream | JSONL stream | Individual JSON files |
| Structure | Flat file per session | Flat file per session | Relational (msg → parts) |
| Tool calls | Inline in stream | Inline in stream | Separate part/ files |
| Session IDs | `ses_*` | `rollout-*` UUID | `ses_*` (matches Claude!) |
| Message format | `{type, content}` | `{type, payload}` | `{id, role, summary}` |

## Conversion Feasibility

### Easy to Convert
- User messages (text → text)
- Timestamps
- Working directory / project context
- Basic conversation flow

### Harder But Doable
- Tool calls need schema mapping (similar concepts across CLIs)
- Tool results structure differs slightly

### Problematic
- Assistant responses - different output formats, thinking patterns
- Model-specific features - Claude's extended thinking vs others
- Tool names - different naming conventions

## Key Observations

1. **Convergent design** - All three converged on similar patterns (skills, project instructions, sessions) despite different companies

2. **OpenCode ↔ Claude** most feasible - matching `ses_*` ID prefix, similar tool concepts

3. **OpenCode's normalized structure** - Better for querying (find all tool calls) but worse for streaming/resuming

4. **Tool mapping required** - `Read` (Claude) vs `read_file` (Codex) vs `read` (OpenCode)

## Conclusion

A Universal Session Format abstraction is technically feasible. The main challenges are:
1. Tool name/schema normalization
2. Model-specific response handling
3. Live bidirectional sync during session execution
