# Enhanced Codex CLI Adapter Research

Date: 2026-02-05

## Current Adapter State

The existing adapter at `rust/stead-core/src/usf/adapters/codex.rs` parses Codex CLI's JSONL session logs from `~/.codex/sessions/`:

**What it does well:**
- Discovers sessions recursively under `sessions/YYYY/MM/DD/rollout-*.jsonl`
- Parses `session_meta` for session ID, working directory, git info, model provider
- Extracts model name from `turn_context` entries
- Maps `response_item` entries (messages, function calls, function call outputs) to USF timeline
- Handles `event_msg` for user messages
- Tracks timestamps for created/last_modified
- Builds lightweight summaries with early termination (stops after 50 lines when key data found)

**What it misses (gaps for Control Room):**
- No token usage extraction (Codex emits `token_count` events)
- No cost tracking (Codex has a cost tracker at `~/.codex/usage/`)
- No error detection on tool results (hardcodes `success: true`)
- No reasoning content extraction
- No session state detection (active vs. completed vs. failed)
- No `history.jsonl` parsing (separate from rollout files)
- No CODEX_THREAD_ID awareness
- Tool name mapping is incomplete for newer Codex tools
- No MCP tool call detection
- No model switching mid-session detection (only captures first model)
- No `CODEX_HOME` environment variable support (hardcoded to `~/.codex`)

---

## Codex CLI Session Format (as of v0.98)

### File Organization

```
~/.codex/
  sessions/
    YYYY/MM/DD/
      rollout-{timestamp}-{session_id}.jsonl   # Per-session log
  usage/                                        # Cost/usage records
  config.toml                                   # User configuration
  skills/                                       # Custom skills
```

The base directory can be changed via `CODEX_HOME` environment variable (defaults to `~/.codex`).

### JSONL Entry Types

Each line in a rollout file is a JSON object with this structure:

```json
{
  "type": "<entry_type>",
  "timestamp": "2026-01-04T18:54:29.163Z",
  "payload": { ... }
}
```

| Entry Type | Purpose | Currently Parsed |
|-----------|---------|:---:|
| `session_meta` | Session ID, cwd, git info, model provider | Yes |
| `turn_context` | Model name for current turn | Yes (first only) |
| `response_item` | Messages, function calls, function outputs | Yes |
| `event_msg` | User messages, token counts, system events | Partially |

### event_msg Subtypes

The `event_msg` type uses a `payload.type` discriminator:

| Subtype | Purpose | Currently Parsed |
|---------|---------|:---:|
| `user_message` | User input text | Yes |
| `token_count` | Cumulative token totals per turn | No |
| `agent_message_delta` | Streaming assistant output | No (not needed for final state) |
| Other event types | Various lifecycle events | No |

### Token Count Event

Each `event_msg` with `payload.type === "token_count"` reports cumulative totals. The CLI subtracts previous totals to get per-turn usage:

```json
{
  "type": "event_msg",
  "timestamp": "...",
  "payload": {
    "type": "token_count",
    "input": 12345,
    "cached_input": 8000,
    "output": 2345,
    "reasoning": 500,
    "total": 15190
  }
}
```

The adapter should track the last `token_count` event to populate `SessionMetadata.tokens`.

### response_item Subtypes

| Subtype (payload.type) | Purpose | Currently Parsed |
|------------------------|---------|:---:|
| `message` | User/assistant text messages | Yes |
| `function_call` | Tool invocation with name, call_id, arguments | Yes |
| `function_call_output` | Tool result with call_id, output | Yes (no error detection) |
| `reasoning` | Model reasoning/chain-of-thought | No |

---

## Codex CLI Integration Points

### 1. OpenTelemetry Export

Codex supports structured event export via OTEL, configured in `~/.codex/config.toml`:

```toml
[otel]
exporter = { otlp-http = {
  endpoint = "https://otel.example.com/v1/logs",
  protocol = "binary",
  headers = { "x-otlp-api-key" = "${OTLP_TOKEN}" }
}}
log_user_prompt = true
environment = "dev"
```

**Emitted events:** Conversation starts, API requests, SSE events, WebSocket activity, user prompts, tool decisions, tool results.

**Stead opportunity:** Stead could run a lightweight OTEL collector that receives Codex events in real-time, rather than polling JSONL files. This would enable live session monitoring in the Control Room.

### 2. Notify Hook

Codex can invoke an external script on `agent-turn-complete`:

```toml
notify = ["python3", "/path/to/notify.py"]
```

The script receives a JSON argument containing:
- `thread-id`
- `turn-id`
- Working directory
- Input messages
- Assistant responses

**Stead opportunity:** A stead notify script could push turn-complete events to the Control Room, enabling near-real-time "agent just finished a turn" updates without polling.

### 3. CODEX_THREAD_ID Environment Variable

Shell executions now receive `CODEX_THREAD_ID`, so child processes (including stead) can detect which Codex thread spawned them.

**Stead opportunity:** If stead is invoked from within a Codex session, it can read this env var to correlate its own activity with the parent Codex session.

### 4. MCP Server Integration

Codex connects to MCP servers defined in config.toml. Stead could register as an MCP server, giving Codex direct access to stead's session visibility tools.

```toml
[mcp_servers.stead]
command = ["stead", "mcp-serve"]
enabled_tools = ["list_sessions", "get_session", "get_project_status"]
```

**Stead opportunity:** Codex agents could query stead for cross-CLI session state (e.g., "what's Claude Code doing in this project?") via MCP tool calls.

### 5. --json Flag for Structured Output

`codex exec --json` outputs all events as JSONL to stdout:

Event types: `thread.started`, `turn.started`, `turn.completed`, `turn.failed`, `item.*`, `error`.

**Stead opportunity:** For programmatic Codex invocations, stead could parse the JSON stream for live monitoring.

### 6. Usage Records

Codex stores usage/cost data at `~/.codex/usage/`. The cost tracker intercepts API responses to record token counts and model pricing.

**Stead opportunity:** Parse usage records to populate `SessionMetadata.cost` without needing to compute pricing ourselves.

---

## Tool Name Mapping Gaps

The current `UniversalTool::from_codex()` handles basic tools. Codex's actual tool names (based on the Responses API function calling):

| Codex Tool | Current Mapping | Correct Mapping |
|-----------|----------------|-----------------|
| `read_file` / `read` | Read | Read |
| `write_file` / `write` | Write | Write |
| `edit_file` / `edit` / `apply_diff` | Edit | Edit |
| `shell` / `bash` / `run_command` | Bash | Bash |
| `grep` / `search` | Search | Search |
| `glob` / `list_files` | Glob | Glob |
| `ls` / `list_dir` | List | List |
| `ask` / `ask_user` | Ask | Ask |
| `call_agent` / `spawn_agent` | Task | Task |
| `web_search` | **unmapped** | WebSearch |
| `web_fetch` / `fetch_url` | **unmapped** | WebFetch |
| `mcp_*` (MCP tool calls) | **unmapped** | Unknown (or new MCP category) |
| `create_file` | **unmapped** | Write |
| `patch` / `apply_patch` | **unmapped** | Edit |

MCP tool calls appear as function calls with names prefixed by the MCP server identifier. These should map to `Unknown` unless we add a dedicated `Mcp` variant to `UniversalTool`.

---

## Missing Data for Control Room

### Critical Gaps

1. **Token usage** -- The `token_count` event_msg provides cumulative input/output/reasoning/cached token counts. The adapter ignores these entirely, leaving `SessionMetadata.tokens` as `None`.

2. **Session liveness** -- No way to distinguish active from completed sessions. Could be inferred from file modification time recency plus absence of a `turn.failed` or similar terminal event.

3. **Error detection** -- `function_call_output` entries don't have an explicit error flag, but output content can indicate errors. The adapter hardcodes `success: true`. Heuristic: if output contains common error patterns or is empty, mark as potentially failed.

4. **Cost data** -- Available in `~/.codex/usage/` but not parsed. Also could be computed from token counts + model pricing.

5. **Model switching** -- The adapter only captures the first `turn_context` model. Codex supports `/model` command to switch mid-session. Should track model per turn or at least capture the last model used.

### Important Gaps

6. **Reasoning content** -- `response_item` entries with `payload.type === "reasoning"` contain chain-of-thought text. Could map to `AssistantMessage.thinking` field.

7. **CODEX_HOME support** -- Adapter hardcodes `~/.codex`. Should check `CODEX_HOME` environment variable first.

8. **history.jsonl** -- Separate from rollout files, controlled by `history.persistence` config. Contains a different view of session data. May be redundant with rollout files but worth investigating.

9. **Parallel tool execution** -- Codex supports parallel shell tools. The adapter's timeline is strictly sequential. May need to represent concurrent tool calls.

### Nice-to-Have

10. **Steer mode detection** -- Codex's "steer mode" allows queuing follow-up input while agent is running. Could indicate user engagement pattern.

11. **Skills detection** -- Skills loaded from `~/.codex/skills/` or `~/.agents/skills/`. Could enrich session context.

12. **Approval mode** -- Whether session ran in auto/read-only/full-access mode. Indicates trust level.

---

## Recommendations for Enhanced Adapter

### 1. Parse token_count events

Extract the last `token_count` event_msg to populate `SessionMetadata.tokens`:

```rust
"event_msg" => {
    if payload.item_type.as_deref() == Some("token_count") {
        if let (Some(input), Some(output)) = (payload.input_tokens, payload.output_tokens) {
            token_usage = Some(TokenUsage { input, output });
        }
    }
}
```

### 2. Support CODEX_HOME

```rust
fn resolve_codex_home() -> Option<PathBuf> {
    std::env::var("CODEX_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| expand_home("~/.codex"))
}
```

### 3. Track model changes across turns

Instead of stopping at the first `turn_context`, keep updating the model field. Store the latest model seen.

### 4. Add web tool mappings

```rust
"web_search" => Self::WebSearch,
"web_fetch" | "fetch_url" => Self::WebFetch,
"create_file" => Self::Write,
"patch" | "apply_patch" => Self::Edit,
```

### 5. Parse reasoning content

Map `response_item` entries with `type: "reasoning"` to `AssistantMessage.thinking`, attaching to the preceding or following assistant message.

### 6. Heuristic error detection for tool results

Since Codex doesn't have an explicit error flag, apply heuristics:
- Empty output with a function that should produce output -> likely error
- Output containing "Error:", "error:", "ENOENT", "Permission denied" -> mark `success: false`

### 7. Notify hook integration (future)

Ship a `stead-notify` script that Codex can invoke:

```toml
# ~/.codex/config.toml
notify = ["stead", "notify", "--source", "codex"]
```

This would push turn-complete events to stead's event bus for real-time Control Room updates.

### 8. MCP server mode (future)

Implement `stead mcp-serve` so Codex can query stead directly:

```toml
[mcp_servers.stead]
command = ["stead", "mcp-serve"]
```

Exposed tools:
- `list_sessions` -- All active sessions across CLIs
- `get_session` -- Full session details
- `project_status` -- What's happening in a given project directory

---

## Comparison: Integration Approaches

| Approach | Latency | Complexity | Data Richness | Requires Config |
|----------|---------|-----------|---------------|:---:|
| **JSONL polling** (current) | High (file scan) | Low | Full session history | No |
| **Notify hook** | Medium (per-turn) | Low | Turn summaries only | Yes |
| **OTEL collector** | Low (real-time) | High | Rich event stream | Yes |
| **MCP server** | On-demand | Medium | Query-based | Yes |
| **--json pipe** | Low (streaming) | Medium | Full event stream | Programmatic only |

**Recommendation:** Start with improved JSONL polling (fix gaps above), then add notify hook support as a low-effort real-time upgrade. MCP server integration is the highest-value long-term investment since it enables bidirectional communication.

---

## USF Schema Additions Needed

| Field | Where | Purpose |
|-------|-------|---------|
| `reasoning_tokens` | `TokenUsage` | Codex reports reasoning tokens separately |
| `cached_tokens` | `TokenUsage` | Both Codex and Claude track cached input tokens |
| `parent_session_id` | `SessionSource` | Subagent hierarchy (shared need with OpenCode) |
| `tool_state` | `ToolCall` or new enum | Execution state for live sessions |
| `session_state` | `SessionMetadata` | Active/completed/failed/compacted |
| `approval_mode` | `SessionMetadata` or `ModelInfo.config` | Trust level context |

---

## Sources

- [Codex CLI Features](https://developers.openai.com/codex/cli/features/)
- [Codex Configuration Reference](https://developers.openai.com/codex/config-reference/)
- [Codex Advanced Configuration](https://developers.openai.com/codex/config-advanced/)
- [Codex CLI Reference](https://developers.openai.com/codex/cli/reference/)
- [Codex Changelog](https://developers.openai.com/codex/changelog/)
- [Codex GitHub Releases](https://github.com/openai/codex/releases)
- [Codex Non-Interactive Mode](https://developers.openai.com/codex/noninteractive/)
- [Cost Tracking Issue #5085](https://github.com/openai/codex/issues/5085)
- [Session Export Issue #10407](https://github.com/openai/codex/issues/10407)
- [JSON Output Issue #2288](https://github.com/openai/codex/issues/2288)
- [Codex Session Persistence (DeepWiki)](https://deepwiki.com/openai/codex/3.3-session-management-and-persistence)
- [AI Observer (third-party Codex monitoring)](https://github.com/tobilg/ai-observer)
- [Codex OpenTelemetry Monitoring (SigNoz)](https://signoz.io/docs/codex-monitoring/)
