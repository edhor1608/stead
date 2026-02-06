# Open Model Ecosystem & Adapter Strategy

Date: 2026-02-05

## 1. Open-Weight Models People Use for Coding Agents

The open-weight coding model landscape has matured significantly. These are the models developers actively use with coding agents (not just chat):

### Tier 1: Frontier Open-Weight (Compete with Proprietary)

| Model | Params (Active) | Context | SWE-Bench Verified | Key Strength |
|-------|-----------------|---------|---------------------|--------------|
| **Kimi K2.5** | 1T (32B MoE) | 128K | 76.8% | Best open-weight SWE-Bench score; strong tool use |
| **DeepSeek-V3.2** | 685B (37B MoE) | 128K | 70.2% | First open model with integrated thinking + tool use |
| **GLM-4.7** | 358B | 128K | 74.2% | Strong reasoning, competitive with proprietary |
| **Qwen3-Coder-Next** | 80B (3B MoE) | 128K | 70.6% | Sonnet 4.5-level coding with only 3B active params |

### Tier 2: Practical Local Models (Run on Consumer Hardware)

| Model | Params | Context | Notes |
|-------|--------|---------|-------|
| **Qwen2.5-Coder-32B** | 32B | 32K (128K w/ YaRN) | Best sub-70B coding model; competitive with GPT-4o |
| **DeepSeek-Coder-V2** | 236B (16B MoE) | 128K | 300+ languages, strong benchmarks |
| **DeepSeek R1 (distilled)** | 1.5B-70B | 128K | Reasoning chains for complex debugging |
| **Llama 3.3 70B** | 70B | 128K | General reasoning, large ecosystem |
| **Llama 4 Maverick** | 400B (17B MoE) | 1M | Multimodal, massive context |
| **Llama 4 Scout** | 109B (17B MoE) | 10M | Extreme context for whole-codebase analysis |

### Tier 3: Edge/Fast (Sub-10B, for autocomplete and quick tasks)

| Model | Params | Context | Notes |
|-------|--------|---------|-------|
| **Qwen3-Coder-Next** | 80B (3B active) | 128K | MoE efficiency makes it edge-viable |
| **Gemma 3n** | 2B-8B | 8K-32K | Google's small efficient model |
| **StarCoder2** | 3B-15B | 16K | Purpose-built for code completion |

### Key Trends

1. **MoE dominance** -- Nearly every frontier open model uses Mixture-of-Experts. Active parameter counts (3B-37B) are what matter for inference cost, not total params.
2. **Agentic by design** -- DeepSeek-V3.2 is the first model to integrate thinking into tool use natively. This is the direction all models are moving.
3. **SWE-Bench convergence** -- Top open models (70-77%) are approaching proprietary models (Claude Opus 4 ~80%). The gap is closing fast.
4. **Chinese labs leading** -- DeepSeek, Qwen (Alibaba), Kimi (Moonshot), GLM (Zhipu) dominate the open-weight frontier.

---

## 2. Coding Agent Tools Beyond OpenCode

### CLI-Native Agents (Terminal-first, stead's primary targets)

| Tool | Storage Format | Model Support | Architecture | Stars |
|------|---------------|---------------|--------------|-------|
| **Claude Code** | JSONL per session (`~/.claude/projects/`) | Anthropic only | Proprietary CLI | -- |
| **OpenCode** | JSON files in `~/.local/share/opencode/storage/` | 75+ providers via AI SDK | Go, Bubble Tea TUI | ~95K |
| **Aider** | Git-first (commits as records); chat logs in `.aider.chat.history.md` | Any LLM (best w/ Claude, DeepSeek, GPT) | Python, Architect/Editor pattern | ~30K |
| **Codex CLI** | JSON in `~/.codex/` | OpenAI models | TypeScript | ~20K |
| **Goose** | MCP-based; local storage varies | Any LLM via MCP | Rust, Apache 2.0, by Block | ~15K |
| **Crush** | **SQLite** per project (`.crush/crush.db`) | Multiple providers | Go, Bubble Tea TUI, by Charm | ~10K |
| **OpenHands** | Sandboxed workspace; Docker-based | Any LLM, model-agnostic | Python, web + CLI | ~50K |

### IDE-Integrated Agents (Secondary targets, less adapter need)

| Tool | Integration | Notes |
|------|-------------|-------|
| **Continue** | VS Code / JetBrains | Open-source, autocomplete + chat + edit |
| **Cline** | VS Code extension | Autonomous agent, uses Claude/etc. |
| **Cursor** | Fork of VS Code | Proprietary but popular; has CLI component |
| **Windsurf** | IDE (fork of VS Code) | Codeium's agent IDE |

### Key Observations for stead

1. **Session storage is fragmented** -- Every tool stores sessions differently (JSONL, JSON files, SQLite, git commits, markdown logs). This is exactly the problem USF adapters solve.
2. **Crush's SQLite move is notable** -- The Charm team (ex-OpenCode creator) moved to SQLite. OpenCode may follow. Adapters need to handle storage format changes.
3. **Aider's git-first approach is unique** -- Sessions are effectively git history. An Aider adapter would read `.aider.chat.history.md` and correlate with git commits.
4. **Goose uses MCP** -- Session data flows through Model Context Protocol. An adapter could tap MCP directly rather than parsing files.
5. **OpenHands is cloud/Docker-first** -- Sandboxed execution means session data may not be on the local filesystem. Adapter would need API access or Docker volume mounting.

---

## 3. How stead's Adapter Strategy Should Evolve

### Current State

stead has 3 adapters: Claude Code, Codex CLI, OpenCode. All implement the `SessionAdapter` trait:

```rust
pub trait SessionAdapter {
    fn cli_type(&self) -> CliType;
    fn is_available(&self) -> bool;
    fn base_dir(&self) -> Option<PathBuf>;
    fn list_sessions(&self) -> Result<Vec<SessionSummary>, AdapterError>;
    fn load_session(&self, id: &str) -> Result<UniversalSession, AdapterError>;
}
```

### Recommended Evolution

#### Phase 1: Harden Existing Adapters (Now)

- Fix the gaps documented in `opencode-adapter-enhanced.md` (model info, tokens, tool mapping)
- Handle format changes (OpenCode may move to SQLite)
- Add `OPENCODE_DATA_DIR` env var support

#### Phase 2: Add High-Value Adapters (Next)

Priority order based on user overlap with stead's target audience:

1. **Aider** -- Huge user base, git-native workflows. Adapter reads `.aider.chat.history.md` + correlates git log.
2. **Crush** -- SQLite storage is clean to parse. Same Charm ecosystem as OpenCode. Many OpenCode users will migrate.
3. **Goose** -- Block's backing means staying power. MCP-based architecture is interesting for stead.

#### Phase 3: Abstract the Adapter Interface (Later)

The current `SessionAdapter` trait is solid but tightly coupled to file-system discovery. Future adapters may need:

- **API-based discovery** (OpenHands, cloud agents)
- **Live/streaming sessions** (watching active agent work, not just finished sessions)
- **Bidirectional communication** (sending commands to agents, not just reading state)

### Proposed Trait Evolution

```rust
pub trait SessionAdapter: Send + Sync {
    fn cli_type(&self) -> CliType;
    fn is_available(&self) -> bool;

    // Discovery
    fn list_sessions(&self) -> Result<Vec<SessionSummary>, AdapterError>;
    fn load_session(&self, id: &str) -> Result<UniversalSession, AdapterError>;

    // Optional: live monitoring (Phase 3)
    fn watch_sessions(&self) -> Option<Box<dyn SessionWatcher>> { None }
}

pub trait SessionWatcher {
    fn poll(&mut self) -> Option<SessionEvent>;
}

pub enum SessionEvent {
    SessionStarted(SessionSummary),
    SessionUpdated(String), // session ID
    SessionCompleted(String),
}
```

---

## 4. Should stead Have a Plugin/Adapter System?

### The Case For Plugins

- The CLI agent landscape is fragmenting fast (7+ major tools, more emerging monthly)
- Each tool's storage format changes without notice
- Community contributors know their tools best
- stead can't keep up with every agent tool alone

### The Case Against (Right Now)

- Plugin APIs are a maintenance burden -- they become a compatibility promise
- The adapter trait is already simple enough for contributors to submit PRs
- Three adapters cover ~80% of the target audience (Claude Code + OpenCode + Codex)
- Premature abstraction risk: we don't know what Phase 3 adapters will need

### Recommendation: Staged Approach

**Now: In-tree adapters with a clean trait.** Keep adapters in `stead-core/src/usf/adapters/`. The `SessionAdapter` trait IS the plugin interface -- it just lives in-tree. Anyone can submit a PR to add an adapter.

**Later: Dynamic loading when the need is proven.** If/when:
- More than 8-10 adapters exist
- Storage formats change frequently (breaking adapters)
- Third-party tools want to ship their own stead adapter

Then consider:
- A `stead-adapter-sdk` crate that external adapters depend on
- Dynamic loading via `libloading` or a WASI-based plugin system
- An adapter registry (like MCP's server registry)

**The principle:** Don't build the plugin system until the pain of not having it is clear. The current trait-based approach scales to at least 10 adapters without friction.

---

## 5. Adapter Coverage Matrix

How well does stead's adapter strategy cover the market?

| Tool | Users (est.) | Adapter Status | Priority | Effort |
|------|-------------|----------------|----------|--------|
| Claude Code | High | **Exists** | -- | -- |
| OpenCode | High | **Exists** (needs enhancement) | -- | Medium |
| Codex CLI | Medium | **Exists** | -- | -- |
| Aider | High | **Not started** | P1 | Medium (markdown + git log parsing) |
| Crush | Growing | **Not started** | P2 | Low (clean SQLite schema) |
| Goose | Growing | **Not started** | P2 | Medium (MCP-based, different pattern) |
| OpenHands | Medium | **Not started** | P3 | High (Docker/API, not local files) |
| Continue | High | N/A (IDE, no sessions) | -- | -- |
| Cline | High | **Not started** | P3 | Medium (VS Code extension storage) |
| Cursor | Very High | N/A (proprietary, no public format) | -- | -- |

**Coverage of CLI-native agents with existing adapters: ~60%**
**Coverage with Aider + Crush + Goose added: ~90%**

---

## Sources

- [Best Open-Source LLMs 2026 (Hugging Face)](https://huggingface.co/blog/daya-shankar/open-source-llms)
- [Top Open Source LLMs 2026 (Contabo)](https://contabo.com/blog/open-source-llms/)
- [DeepSeek-V3.2 Outperforms GPT-5 on Reasoning (InfoQ)](https://www.infoq.com/news/2026/01/deepseek-v32/)
- [DeepSeek-V3.2 Benchmarks and Agent Case Studies](https://mgx.dev/blog/deepseek-v3-2-agents-benchmarks)
- [Kimi K2 Analysis (IntuitionLabs)](https://intuitionlabs.ai/articles/kimi-k2-open-weight-llm-analysis)
- [Kimi K2.5 Guide (Codecademy)](https://www.codecademy.com/article/kimi-k-2-5-complete-guide-to-moonshots-ai-model)
- [Qwen3-Coder-Next Release (MarkTechPost)](https://www.marktechpost.com/2026/02/03/qwen-team-releases-qwen3-coder-next-an-open-weight-language-model-designed-specifically-for-coding-agents-and-local-development/)
- [Qwen3-Coder Evaluation Results](https://eval.16x.engineer/blog/qwen3-coder-evaluation-results)
- [Top 7 Open Source AI Coding Agents 2026 (AIMultiple)](https://research.aimultiple.com/open-source-ai-coding/)
- [Aider vs OpenCode Comparison (OpenAlternative)](https://openalternative.co/compare/aider/vs/opencode)
- [Top 5 CLI Coding Agents 2026 (DEV)](https://dev.to/lightningdev123/top-5-cli-coding-agents-in-2026-3pia)
- [Goose: Open Source AI Agent (Block)](https://block.xyz/inside/block-open-source-introduces-codename-goose)
- [Crush CLI (Charm)](https://github.com/charmbracelet/crush)
- [OpenHands Platform](https://openhands.dev/)
- [Aider Documentation](https://aider.chat/docs/)
- [Building a TUI to index coding agent sessions (Stan's blog)](https://stanislas.blog/2026/01/tui-index-search-coding-agent-sessions/)
- [OpenCode Providers Documentation](https://opencode.ai/docs/providers/)
