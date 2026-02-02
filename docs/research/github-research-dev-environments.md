# GitHub Research: Dev Environments & Execution Daemons

Research into existing projects for development environment management, terminal alternatives, and agent execution frameworks. Focus: what can we steal/build upon for stead's "execution daemon" concept.

---

## 1. Dev Environment Managers

### devenv
**Repo:** https://github.com/cachix/devenv
**What:** Nix-powered declarative dev environments with sub-100ms activation via precise evaluation caching.

**Key ideas to steal:**
- Declarative secrets with SecretSpec - retrieves from Keychain, 1Password, LastPass, dotenv, or env vars
- Git-hooks integration (switching to Rust-based `prek` from `pre-commit`)
- Backend abstraction layer for multiple Nix implementations (preparing for Snix, their Rust rewrite)
- Changelog option for module authors to declare breaking changes

**Limitations:** Heavy Nix dependency. Leaky abstraction requiring Nix knowledge.

---

### devbox (Jetify)
**Repo:** https://github.com/jetify-com/devbox
**What:** Nix-powered but abstracts Nix completely away. Simple `devbox.json` config.

**Key ideas to steal:**
- GitHub Actions cache integration for near-instant setup
- "Developer mode" vs "production mode" distinction
- 400k+ packages from Nix registry without learning Nix
- Single command onboarding: `devbox shell`

**Limitations:** Less powerful than devenv due to abstraction. Still Nix under the hood.

---

### Flox
**Repo:** https://github.com/flox/flox
**What:** "Virtual environment and package manager" - containerless, uses Nix but hides it.

**Key ideas to steal:**
- Environments as "layers" that compose and replace dependencies
- Full software lifecycle portability (dev -> CI -> production)
- 80k+ packages in catalog with historical versions
- Works seamlessly with existing shells/dotfiles (no container isolation)
- Designed for agentic coding - explicitly targets AI agent workflows

**Limitations:** Still Nix-based. Limited to Nix package ecosystem.

---

### mise (formerly rtx)
**Repo:** https://github.com/jdx/mise
**Docs:** https://mise.jdx.dev/
**What:** Fast asdf-compatible runtime version manager. Replaces asdf, nvm, pyenv, rbenv.

**Key ideas to steal:**
- **No shims** - modifies PATH directly, eliminating ~120ms overhead per call (asdf) vs ~5ms at prompt (mise)
- Backends beyond asdf plugins (aqua, ubi) with native signature verification
- Supports Windows when using non-asdf backends
- Replaces direnv for env var switching
- Legacy version file support (.node-version, .python-version, etc.)

**Highly relevant:** Fast, no overhead approach is exactly what stead needs.

---

### Daytona
**Repo:** https://github.com/daytonaio/daytona
**What:** Secure runtime for AI-generated code execution. 200ms workspace startup.

**Key ideas to steal:**
- **AI code sandbox** - designed for running agent-generated code safely
- OCI container-based workspaces
- Remote targets - deploy to local or remote Docker
- devcontainer.json integration
- SSH access with public key or access token auth

**Highly relevant:** Their framing as "infrastructure for AI-generated code" aligns with stead's direction.

---

## 2. Terminal Multiplexers & Alternatives

### Zellij
**Repo:** https://github.com/zellij-org/zellij
**What:** Modern terminal workspace with WebAssembly plugin system.

**Key ideas to steal:**
- **WebAssembly/WASI plugins** - write plugins in any language that compiles to WASM
- Built-in web-client for browser-based terminal access
- Plugin Manager with runtime loading/reloading
- Floating and stacked panes
- Layouts for personal automation
- Session resurrection

**Very relevant:** Plugin architecture via WASM is compelling for extensibility.

**Limitations:** No native Windows support (WSL only).

---

### WezTerm
**Repo:** https://github.com/wez/wezterm
**What:** GPU-accelerated terminal emulator + multiplexer in Rust.

**Key ideas to steal:**
- **Lua configuration** with hot-reload
- Rich CLI (wezterm cli) for programmatic control
- Workspace/session management
- Multiplexer built into terminal (no tmux needed)
- SSH connectivity, serial ports

**Relevant:** Programmable terminal concept. But we're building something different.

---

## 3. Shell Alternatives (Structured Data)

### Nushell
**Repo:** https://github.com/nushell/nushell
**Docs:** https://www.nushell.sh/
**What:** Shell that treats everything as structured data (tables, not text streams).

**Key ideas to steal:**
- **Structured pipelines** - data flows as tables/records, not text
- Built-in parsing for JSON, YAML, TOML, SQLite, Excel
- Cross-platform with first-class Windows support
- Type-aware operations (select, filter, sort) work consistently

**Highly relevant:** If stead has a "shell layer", structured data is the way. This is the antithesis of parsing text.

---

### Oils (formerly Oil Shell)
**Repo:** https://github.com/oils-for-unix/oils
**What:** Bash upgrade path - POSIX-compatible (osh) with better language (ysh).

**Key ideas to steal:**
- **Gradual adoption** - works with existing bash scripts, then improve incrementally
- Minimal dependencies (GCC, libc, make, bash)
- Target: existing shell codebases with thousands of lines

**Less relevant:** We're not trying to be bash-compatible.

---

## 4. Process Supervisors & Task Runners

### Overmind / Hivemind
**Repos:** https://github.com/DarthSim/overmind, https://github.com/DarthSim/hivemind
**What:** Procfile-based process managers for development.

**Key ideas to steal:**
- **tmux integration** (Overmind) - each process gets a tmux pane, connect/detach
- Can restart individual processes without restarting the stack
- Hivemind uses PTY capture - fixes TTY color/output issues
- Simple Procfile format (widely understood from Heroku)

**Relevant:** Procfile-style config is dead simple. tmux integration for debug access is clever.

---

### mprocs
**Repo:** https://github.com/pvolok/mprocs
**What:** TUI for running multiple parallel processes.

**Key ideas to steal:**
- Split-pane TUI showing all processes
- Switch between outputs, interact with processes
- `mprocs.yaml` config at project root
- Can run vim/interactive apps inside mprocs

**Relevant:** TUI approach for multi-process development. But mprocs is finite-lifetime focused.

---

### pueue
**Repo:** https://github.com/Nukesor/pueue
**What:** Command queue manager - sequential/parallel execution with persistence.

**Key ideas to steal:**
- **Queue persists to disk** - survives crashes, system restarts
- Not bound to terminal - control from any terminal on machine
- Pause/resume individual tasks or groups
- Task dependencies (`--after` flag)
- Parallel execution within groups

**Highly relevant:** Queue + persistence + daemon model is exactly what stead needs.

---

### just
**Repo:** https://github.com/casey/just
**Docs:** https://just.systems/
**What:** Modern command runner (Makefile alternative).

**Key ideas to steal:**
- Simple, familiar syntax (Makefile-inspired but saner)
- Cross-platform (Mac, Linux, Windows)
- Shell completion for all major shells
- Can be bundled into Node.js projects (just-install)

**Relevant:** Simple task definition format. Not a daemon though.

---

### watchexec
**Repo:** https://github.com/watchexec/watchexec
**What:** Run commands on file changes.

**Key ideas to steal:**
- Respects .gitignore by default
- Debouncing (50ms default)
- Process group management (SIGTERM then SIGKILL)
- Can be used as Rust library (Tokio-based)

**Relevant:** File watching + command execution is a common pattern. Library approach useful.

---

## 5. Agent Execution Frameworks

### Claude Agent SDK
**Repo:** https://github.com/anthropics/claude-agent-sdk-python
**What:** Python SDK for Claude Code / agent execution.

**Key ideas to steal:**
- Tools: bash, file edit, file create, file search
- Custom tools via in-process MCP servers (no separate process)
- Bidirectional interactive conversations
- Hooks for extending behavior

**Highly relevant:** This is what we're building infrastructure for.

---

### Anthropic Computer Use Demo
**Repo:** https://github.com/anthropics/anthropic-quickstarts/tree/main/computer-use-demo
**What:** Reference implementation for Claude controlling a desktop.

**Key ideas to steal:**
- Agentic loop pattern (loop.py)
- Screen/keyboard/mouse abstraction
- Coordinate scaling for different resolutions
- Container-based isolation (Ubuntu VM)

**Relevant:** Shows how to structure tool execution for agents.

---

### LangGraph
**Repo:** https://github.com/langchain-ai/langgraph
**What:** State machine / graph-based agent orchestration.

**Key ideas to steal:**
- **Workflows as graphs** - nodes = agents/functions, edges = transitions
- State management - shared state snapshot across nodes
- Persistence, streaming, debugging built-in
- Supervisor pattern for multi-agent coordination

**Relevant:** Graph-based orchestration model. May be overkill for single-agent execution.

---

### smolagents (Hugging Face)
**Repo:** https://github.com/huggingface/smolagents
**What:** Minimal agent library (~1000 lines of code).

**Key ideas to steal:**
- **Code agents write code** instead of JSON tool calls - 30% fewer steps
- Sandboxed Python interpreter
- Docker/E2B isolation options
- Model-agnostic (OpenAI, Anthropic, local via LiteLLM)

**Highly relevant:** Minimalist philosophy. Code-as-action is interesting.

---

### Model Context Protocol (MCP)
**Repos:** https://github.com/modelcontextprotocol/servers, https://github.com/modelcontextprotocol/python-sdk
**What:** Open protocol for LLM-tool integration (Linux Foundation).

**Key ideas to steal:**
- **Standardized tool interface** - Resources, Tools, Prompts
- Structured output validation against schemas
- SDKs for Python, TypeScript, Go, Kotlin, Swift, C#
- FastMCP for quick server creation

**Highly relevant:** MCP is becoming the standard. stead should be MCP-native.

---

### PydanticAI
**What:** Agent framework emphasizing type-safe tool calls.

**Key ideas to steal:**
- Clear schemas for tools and responses
- Dependency injection
- Structured inputs/outputs with validation

**Relevant:** Type safety for agent tools.

---

## 6. Workflow & Orchestration

### Temporal
**Repo:** https://github.com/temporalio/temporal
**What:** Durable execution platform - workflows survive failures.

**Key ideas to steal:**
- **Durable execution** - state captured at every step, auto-recovery
- Workflows + Activities separation (orchestration vs execution)
- Workers that execute on any machine
- History replay for debugging
- MIT licensed, self-hostable

**Potentially relevant:** Overkill for dev workflows, but durable execution concept is powerful.

---

### Dagger
**Repo:** https://github.com/dagger/dagger
**What:** Programmable CI/CD in containers, by Docker's creator.

**Key ideas to steal:**
- **DAG execution** via modified BuildKit
- SDKs in 8 languages
- Interactive REPL for debugging pipelines
- Every operation is cacheable
- Full OpenTelemetry tracing

**Relevant:** Container-based execution with caching. REPL for debugging is nice.

---

### Earthly
**Repo:** https://github.com/earthly/earthly
**What:** Build automation - "Dockerfile + Makefile had a baby".

**Key ideas to steal:**
- Familiar syntax (combines Docker + Make)
- Self-contained builds via containers
- Parallel execution without race conditions
- Glue between language-specific tools and CI

**Less relevant:** Build-focused, not runtime/daemon.

---

## 7. Environment & Config Management

### direnv
**Repo:** https://github.com/direnv/direnv
**What:** Load/unload env vars based on current directory.

**Key ideas to steal:**
- **.envrc** file per project
- Security: new files are "blocked" until explicitly allowed
- Fast (compiled binary, unnoticeable on each prompt)
- Hooks for all major shells

**Relevant:** Project-scoped environment is essential. direnv does this well.

---

## Summary: Key Patterns for stead

### From Dev Environment Managers:
1. **Declarative config** (devbox.json, devenv.nix, mise config)
2. **Secrets management** from system keychains
3. **No-shim approach** (mise) - modify PATH, not shims
4. **AI agent awareness** (Flox, Daytona)

### From Terminal Alternatives:
1. **WebAssembly plugins** (Zellij) for extensibility
2. **Structured data** (Nushell) instead of text streams
3. **Programmable via Lua/code** (WezTerm)

### From Process Supervisors:
1. **Persistent queue** (pueue) survives restarts
2. **Daemon model** - control from any terminal
3. **Task dependencies**
4. **Individual process restart** (Overmind)
5. **Procfile simplicity** for config

### From Agent Frameworks:
1. **MCP as standard interface** for tools
2. **Code-as-action** (smolagents) may be more efficient than JSON tools
3. **Agentic loop pattern** from Anthropic demos
4. **Type-safe tool schemas** (PydanticAI)

### From Workflow Systems:
1. **Durable execution** concept (Temporal)
2. **DAG + caching** (Dagger)
3. **OpenTelemetry** for observability

---

## Top Repos to Study Deeper

| Priority | Repo | Why |
|----------|------|-----|
| 1 | pueue | Queue/daemon model, persistence, exactly what we need |
| 2 | mise | No-shim PATH modification, fast activation |
| 3 | Zellij | WASM plugin architecture |
| 4 | MCP servers | Standard tool protocol |
| 5 | Nushell | Structured data approach |
| 6 | Overmind | tmux integration pattern |
| 7 | smolagents | Minimal agent library, code-as-action |
| 8 | Daytona | AI code sandbox framing |
