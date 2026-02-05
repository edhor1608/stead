# Enhanced OpenCode Adapter Research

Date: 2026-02-05

## Current Adapter State

The existing adapter at `rust/src/usf/adapters/opencode.rs` successfully parses OpenCode's normalized JSON file structure:

```
~/.local/share/opencode/storage/
  session/{projectID}/{sessionID}.json
  message/{sessionID}/{messageID}.json
  part/{messageID}/{partID}.json
  project/{projectID}.json
```

**What it does well:**
- Discovers sessions across project directories
- Loads messages and parts with correct ordering
- Maps tool invocations and results to USF timeline
- Builds session summaries with title extraction

**What it misses (gaps for Control Room):**
- Model info is hardcoded to `"unknown"` -- no provider/model extraction
- No token usage or cost data
- No git branch tracking
- No parent/child session relationships (subagent sessions)
- Tool name mapping is incomplete (misses `patch`, `lsp`, `todowrite`, `todoread`, `webfetch`, `websearch`, `codesearch`, `task`, `question`, `skill`)
- No session version tracking
- No file change summary (additions/deletions/diffs)
- No reasoning/thinking part support
- No `session_diff` data

---

## OpenCode Session Format (Complete)

### Session.Info Fields

| Field | Type | Purpose | Currently Parsed |
|-------|------|---------|:---:|
| `id` | string | Descending ULID for chronological sort | Yes |
| `slug` | string | URL-friendly identifier for sharing | No |
| `projectID` | string | Links to project config | Yes |
| `directory` | string | Working directory path | No (loaded from project) |
| `parentID` | string? | Parent session for subagent sessions | No |
| `title` | string | Session display title | Yes |
| `version` | string | OpenCode version that created session | No |
| `time.created` | i64 | Creation timestamp (ms) | Yes |
| `time.updated` | i64 | Last update timestamp (ms) | Yes |
| `time.compacting` | i64? | When context compaction occurred | No |
| `time.archived` | i64? | When session was archived | No |
| `summary` | object? | File change stats (additions, deletions, diffs) | No |
| `share` | object? | Share URL if enabled | No |
| `permission` | object? | Permission ruleset | No |
| `revert` | object? | Undo state | No |

### MessageV2 Fields

| Field | Type | Purpose | Currently Parsed |
|-------|------|---------|:---:|
| `id` | string | Message identifier | Yes |
| `role` | `"user" \| "assistant"` | Sender role | Yes |
| `sessionID` | string | Parent session | Yes |
| `timestamp` | i64 | Unix timestamp | Indirectly (via time.created) |
| `model` | string? | Model ID (`provider/model`) on assistant msgs | No |
| `tokens` | object? | Token usage (assistant msgs only) | No |
| `cost` | f64? | Cost in USD (assistant msgs only) | No |
| `parentID` | string? | Links assistant reply to user message | No |
| `summary` | string? | Message preview text | No |

### Token Tracking (assistant messages)

```json
{
  "inputTokens": 1234,
  "outputTokens": 567,
  "cachedInputTokens": 890,
  "cacheReadTokens": 200,
  "cacheWriteTokens": 100,
  "reasoningTokens": 300
}
```

### Part Types

| Type | Fields | Currently Parsed |
|------|--------|:---:|
| `text` | `text`, `time` | Yes |
| `tool-invocation` | `toolName`, `toolInvocationInput`, `state` | Partially (no state) |
| `tool-result` | `toolInvocationId`, `text`, `time` | Yes |
| `reasoning` | `text`, `time` | No |
| `file` | `url`, `mime`, `source` | No |
| `image` | `url`, `mime` | No |
| `snapshot` | `snapshot` | No |
| `agent` / `subtask` | agent delegation info | No |

### Tool State (ToolPart)

Tool invocations have an execution state not currently captured:
- `pending` -- queued
- `running` -- executing
- `completed` -- finished successfully
- `failed` -- error occurred

---

## OpenCode's Tool Names (Actual)

The current adapter maps a guess at tool names. Here are the **actual** built-in tool identifiers:

| OpenCode Tool | USF Mapping | Notes |
|---------------|-------------|-------|
| `read` | `Read` | File read |
| `edit` | `Edit` | Exact string replacement |
| `write` | `Write` | Create/overwrite files |
| `patch` | `Edit` | Apply diffs (same USF category) |
| `bash` | `Bash` | Shell execution |
| `grep` | `Search` | Regex search via ripgrep |
| `glob` | `Glob` | File pattern matching |
| `list` | `List` | Directory listing |
| `lsp` | `Unknown` | LSP code intelligence (no USF equivalent) |
| `question` | `Ask` | Ask user for input |
| `task` | `Task` | Delegate to subagent |
| `webfetch` | `WebFetch` | Fetch web content |
| `websearch` | `WebSearch` | Web search |
| `todowrite` | `Unknown` | Task list management (no USF equivalent) |
| `todoread` | `Unknown` | Task list reading (no USF equivalent) |
| `skill` | `Unknown` | Load skill files (no USF equivalent) |
| `codesearch` | `Search` | Semantic code search |
| `multiedit` | `Edit` | Multi-file edit |

**Current adapter misses:** `patch`, `lsp`, `question`, `task`, `webfetch`, `websearch`, `todowrite`, `todoread`, `skill`, `codesearch`, `multiedit`

---

## Supported Providers and Models

### Cloud Providers

OpenCode supports 75+ providers via AI SDK + Models.dev. The major ones:

| Provider | Popular Models | Context Window |
|----------|---------------|----------------|
| Anthropic | claude-opus-4.5, claude-sonnet-4.5 | 200K |
| OpenAI | gpt-5, gpt-5.1-codex | 128K-1M |
| Google | gemini-3-pro | 1M-2M |
| DeepSeek | deepseek-v3.2, deepseek-r1 | 128K |
| Groq | llama-3.3-70b, qwen-qwq-32b | 32K-128K |
| Together AI | various open models | varies |
| OpenRouter | aggregator for many providers | varies |
| Fireworks AI | various open models | varies |
| xAI | grok models | 128K |

### Open-Weight Models (via Ollama/LM Studio/llama.cpp)

These are the models people commonly run locally with OpenCode:

| Model | Parameters | Context Window | Strengths |
|-------|-----------|----------------|-----------|
| **DeepSeek-Coder-V2** | 236B (16B active MoE) | 128K | 300+ languages, top coding benchmarks |
| **DeepSeek V3.2** | 685B (37B active MoE) | 128K | Near-linear attention, fits in memory |
| **DeepSeek R1** | 671B (37B active) | 128K | Reasoning chains, math, code |
| **Qwen 2.5 Coder 32B** | 32B | 32K (128K with YaRN) | Competitive with GPT-4o on code repair |
| **Qwen 3** | 0.6B-235B | 32K (expandable to 1M) | Multilingual, reasoning, code |
| **Llama 3.3 70B** | 70B | 128K | General reasoning |
| **Llama 4 Maverick** | 400B (17B active MoE) | 1M | Multimodal, large context |
| **Llama 4 Scout** | 109B (17B active MoE) | 10M | Extreme context length |
| **Mixtral 8x22B** | 176B (39B active MoE) | 64K | Cost-effective MoE |
| **Mistral Large** | -- | 128K | Strong instruction following |
| **Gemma 3n** | 2B-8B | 8K-32K | Small, fast, edge deployment |
| **StarCoder2** | 3B-15B | 16K | Purpose-built for code |

### Model ID Format

OpenCode stores model identifiers as `provider_id/model_id`:
- `anthropic/claude-sonnet-4-5-20250929`
- `openai/gpt-5`
- `ollama/deepseek-coder-v2`
- `lmstudio/qwen2.5-coder:32b`
- `groq/llama-3.3-70b-versatile`

This format directly encodes the provider, which the current adapter ignores.

---

## Missing Data for Control Room

The Control Room needs to answer: "What's running? What finished? What needs me?" For OpenCode sessions, these gaps matter:

### Critical Gaps

1. **Model identification** -- Control Room needs to show which model is being used. The `model` field on assistant messages contains `provider/model_id`. The adapter currently outputs `"unknown"` for both provider and model.

2. **Token usage and cost** -- Control Room needs resource consumption visibility. Assistant messages contain `tokens` (inputTokens, outputTokens, reasoningTokens, etc.) and `cost` (USD). The adapter discards all of this.

3. **Subagent relationships** -- Control Room needs to show agent delegation trees. The `parentID` field on sessions links child (subagent) sessions to parents. The adapter has no concept of session hierarchy.

4. **Tool execution state** -- Control Room shows running/completed/failed status. Tool parts have a `state` field (`pending`/`running`/`completed`/`failed`). The adapter hardcodes `success: true`.

### Important Gaps

5. **File change summary** -- Session-level `summary` field tracks additions, deletions, diffs. Useful for Control Room "what changed" view.

6. **Git context** -- OpenCode doesn't store git branch/commit in session metadata directly, but the working directory can be used to infer it at read time.

7. **Reasoning/thinking content** -- `reasoning` parts are the equivalent of Claude's extended thinking. The adapter drops them entirely.

8. **Session version** -- Which OpenCode version created the session. Useful for compatibility and debugging.

9. **Context compaction timestamp** -- `time.compacting` indicates when the session's context was pruned. Relevant for understanding session quality/completeness.

### Nice-to-Have

10. **Share URL** -- If session was shared, the URL is available.

11. **Session slug** -- Human-readable identifier for display.

12. **`session_diff` data** -- Full file diffs stored separately at `storage/session_diff/{sessionID}.json`.

---

## Recommendations for Enhanced Adapter

### 1. Parse model info from assistant messages

Extract `provider` and `model` from the `model` field on assistant messages (format: `provider/model_id`). First assistant message with a model field sets the session's model info.

### 2. Aggregate token usage and cost

Sum `tokens` and `cost` from all assistant messages to populate `SessionMetadata.tokens` and `SessionMetadata.cost`.

### 3. Parse `parentID` for subagent sessions

Requires USF schema change: add optional `parent_session_id` to `UniversalSession` or `SessionSource`.

### 4. Complete tool name mapping

Update `UniversalTool::from_opencode()` to handle all actual tool names:

```rust
fn from_opencode(name: &str) -> Self {
    match name {
        "read" => Self::Read,
        "write" => Self::Write,
        "edit" | "patch" | "multiedit" => Self::Edit,
        "bash" => Self::Bash,
        "grep" | "codesearch" => Self::Search,
        "glob" => Self::Glob,
        "list" => Self::List,
        "question" => Self::Ask,
        "task" => Self::Task,
        "webfetch" => Self::WebFetch,
        "websearch" => Self::WebSearch,
        // No USF equivalent yet:
        "todowrite" | "todoread" | "skill" | "lsp" => Self::Unknown,
        _ => Self::Unknown,
    }
}
```

### 5. Parse tool execution state

Map `state` field to `ToolResult.success`:
- `completed` -> `success: true`
- `failed` -> `success: false`
- `pending`/`running` -> no result yet (skip ToolResult or mark as in-progress)

### 6. Parse reasoning parts

Map `reasoning` parts to `AssistantMessage.thinking` field, same as Claude's extended thinking.

### 7. Parse file change summary

Add optional `file_changes` to `SessionMetadata` or `SessionSummary` for Control Room display.

### 8. USF Schema Additions Needed

| Field | Where | Purpose |
|-------|-------|---------|
| `parent_session_id` | `SessionSource` or `UniversalSession` | Subagent hierarchy |
| `file_changes` | `SessionMetadata` | File change stats |
| `session_version` | `SessionSource` | CLI version that created session |
| `tool_state` | `ToolCall` or new enum | Execution state (pending/running/completed/failed) |

---

## Environment Variable

OpenCode's data directory can be customized via `OPENCODE_DATA_DIR` (defaults to `~/.local/share/opencode`). The adapter should respect this:

```rust
const OPENCODE_DIR: &str = "~/.local/share/opencode";

fn resolve_data_dir() -> Option<PathBuf> {
    std::env::var("OPENCODE_DATA_DIR")
        .ok()
        .map(PathBuf::from)
        .or_else(|| expand_home(OPENCODE_DIR))
}
```

---

## Sources

- [OpenCode GitHub](https://github.com/opencode-ai/opencode)
- [OpenCode Models Documentation](https://opencode.ai/docs/models/)
- [OpenCode Providers Documentation](https://opencode.ai/docs/providers/)
- [OpenCode Config Documentation](https://opencode.ai/docs/config/)
- [OpenCode Tools Documentation](https://opencode.ai/docs/tools/)
- [OpenCode Built-in Tools Reference (DeepWiki)](https://deepwiki.com/sst/opencode/5.3-built-in-tools-reference)
- [OpenCode Session Management (DeepWiki)](https://deepwiki.com/sst/opencode/2.1-session-management)
- [OpenCode Session Management (anomalyco fork)](https://deepwiki.com/anomalyco/opencode/3.1-session-management)
- [OpenCode Session Sharing (DeepWiki)](https://deepwiki.com/sst/opencode/6.6-session-sharing)
- [OpenCode CLI Overview (ccusage)](https://ccusage.com/guide/opencode/)
- [Setting Up OpenCode with Local Models](https://theaiops.substack.com/p/setting-up-opencode-with-local-models)
- [Private AI Coding: OpenCode + Docker Model Runner](https://www.docker.com/blog/opencode-docker-model-runner-private-ai-coding/)
- [Best Open-Source LLMs 2025 (Hugging Face)](https://huggingface.co/blog/daya-shankar/open-source-llms)
- [Best Ollama Models for Coding 2025](https://www.codegpt.co/blog/best-ollama-model-for-coding)
