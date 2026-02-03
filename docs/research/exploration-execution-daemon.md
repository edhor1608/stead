# Exploration: Execution Daemon

> **Historical Note:** This exploration document was written when TypeScript/Bun was being considered for implementation. The tech stack decision has since been made: **stead will be implemented in Rust**. See [decisions-log.md](../plans/decisions-log.md) for details. The architectural concepts and requirements discussed here remain valid; only the implementation language has changed.

Date: 2026-02-02

## The Core Insight

**Agents don't need terminals. They need a task execution daemon with optional terminal visualization.**

The terminal is a human interface to a shell. It provides:
- Visual rendering of text output
- Keyboard input handling
- Scrollback history
- Color and formatting interpretation

Agents need none of this. They need:
- Command execution with structured results
- Process lifecycle management
- Output capture (stdout, stderr, exit code)
- Environment and working directory control
- Timeout and resource management

The terminal emulator is overhead. The shell is the real interface—and even then, agents often bypass shell parsing entirely when calling tools directly.

---

## 1. Architecture Overview

### High-Level Design

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Execution Daemon                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Project    │  │   Project    │  │   Project    │         │
│  │   Session    │  │   Session    │  │   Session    │         │
│  │  (stead-1)   │  │  (picalyze)  │  │  (qwer-q)    │         │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
│         │                 │                 │                  │
│  ┌──────┴───────┐  ┌──────┴───────┐  ┌──────┴───────┐         │
│  │ Execution    │  │ Execution    │  │ Execution    │         │
│  │ Contexts     │  │ Contexts     │  │ Contexts     │         │
│  │ (isolated    │  │ (isolated    │  │ (isolated    │         │
│  │  env vars,   │  │  env vars,   │  │  env vars,   │         │
│  │  cwd, etc.)  │  │  cwd, etc.)  │  │  cwd, etc.)  │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                      Core Services                              │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐  │
│  │  Process   │ │  Output    │ │  Resource  │ │  Event     │  │
│  │  Manager   │ │  Router    │ │  Limiter   │ │  Bus       │  │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘  │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                        API Layer                                │
│  ┌────────────────────────────────────────────────────────────┐│
│  │  Unix Socket (primary) │ HTTP (optional) │ gRPC (optional) ││
│  └────────────────────────────────────────────────────────────┘│
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                      Client Adapters                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │
│  │  Claude  │ │  MCP     │ │ Terminal │ │  ACP     │         │
│  │  Code    │ │  Server  │ │  Bridge  │ │  Adapter │         │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Project-scoped by default**: Every execution belongs to a project session
2. **Structured output first**: Raw text is just one channel among many
3. **Observable**: Every execution emits telemetry by default
4. **Protocol-agnostic core**: Multiple client protocols can wrap the same execution engine
5. **Optional PTY**: Terminal emulation available when needed, not always

---

## 2. Protocols & Interfaces

### Primary Interface: Unix Domain Socket + JSON-RPC 2.0

**Why this combination:**
- Unix sockets are fast, local-only (secure by default), support file descriptor passing
- JSON-RPC is simple, well-understood, has good tooling
- Same approach used by LSP, MCP, and many daemon services

**Socket location:**
```text
~/.stead/daemon.sock                    # Global daemon socket
~/.stead/sessions/{project}/exec.sock   # Per-project sockets (optional)
```

### Core RPC Methods

```typescript
// Session Management
interface SessionMethods {
  "session.create": (params: {
    projectId: string;
    workingDirectory: string;
    environment?: Record<string, string>;
    shell?: string;  // default: user's shell
  }) => { sessionId: string };

  "session.destroy": (params: { sessionId: string }) => void;

  "session.list": () => Session[];
}

// Execution
interface ExecutionMethods {
  "exec.run": (params: {
    sessionId: string;
    command: string;           // Shell command string
    args?: string[];           // Or direct exec with args
    timeout?: number;          // ms, default 120000
    stdin?: string;            // Input to provide
    outputMode?: "buffered" | "streaming";
    channels?: OutputChannel[];  // What to capture
  }) => ExecutionResult | ExecutionHandle;

  "exec.runBackground": (params: {
    sessionId: string;
    command: string;
    notifyOn?: ("exit" | "output" | "timeout")[];
  }) => { executionId: string };

  "exec.status": (params: { executionId: string }) => ExecutionStatus;

  "exec.cancel": (params: { executionId: string }) => void;

  "exec.output": (params: {
    executionId: string;
    channel?: string;
    offset?: number;
    limit?: number;
  }) => OutputChunk[];
}

// Interactive (PTY) - Optional
interface InteractiveMethods {
  "pty.spawn": (params: {
    sessionId: string;
    command?: string;  // default: shell
    size?: { rows: number; cols: number };
  }) => { ptyId: string };

  "pty.write": (params: { ptyId: string; data: string }) => void;

  "pty.resize": (params: { ptyId: string; rows: number; cols: number }) => void;

  "pty.close": (params: { ptyId: string }) => void;
}
```

### Notifications (Server -> Client)

```typescript
interface Notifications {
  "exec.started": { executionId: string; command: string; timestamp: string };
  "exec.output": { executionId: string; channel: string; data: string };
  "exec.completed": { executionId: string; exitCode: number; duration: number };
  "exec.failed": { executionId: string; error: string };
  "pty.output": { ptyId: string; data: string };
}
```

### Alternative Protocols

**HTTP REST API** (for web-based UIs):
```
POST /sessions
GET  /sessions/{id}
POST /sessions/{id}/exec
GET  /sessions/{id}/executions/{execId}
WS   /sessions/{id}/stream  (for real-time output)
```

**MCP Tool Provider** (for AI agents):
The daemon can expose itself as an MCP server, providing `execute_command` and `spawn_shell` tools that agents can call through standard MCP.

**ACP Adapter** (for editor integration):
Wrap the daemon as an ACP agent, allowing Zed/JetBrains to use it as an execution backend.

---

## 3. Project-Scoped Sessions

### The Problem with Global Shells

Traditional terminal sessions are global or at best window-based. When you run `npm start`, there's nothing linking that process to "the picalyze project" other than the working directory.

This causes:
- Port conflicts (multiple projects want :3000)
- Environment bleeding (wrong API keys)
- Process orphaning (which `node` belongs to which project?)

### Session Model

```typescript
interface Session {
  id: string;
  projectId: string;

  // Execution context
  workingDirectory: string;
  environment: Record<string, string>;  // Merged with system env
  shell: string;

  // Resource tracking
  activeExecutions: ExecutionHandle[];
  backgroundProcesses: ProcessInfo[];

  // Port allocation
  allocatedPorts: PortRange;

  // Lifecycle
  createdAt: Date;
  lastActivity: Date;
  state: "active" | "idle" | "suspended";
}
```

### Port Namespacing

Each project session gets a dedicated port range:

```typescript
interface PortAllocation {
  // Automatic allocation
  project: "picalyze" → ports: 3100-3199
  project: "qwer-q"   → ports: 3200-3299
  project: "stead"    → ports: 3300-3399

  // Environment injection
  PORT=3100                    // Primary port
  DEV_SERVER_PORT=3101         // Secondary
  STORYBOOK_PORT=3102          // Tertiary
  // etc.
}
```

The daemon manages allocation and can:
- Inject port environment variables
- Rewrite localhost URLs in output
- Provide `.local` domain aliases (picalyze.local:3000 → localhost:3100)

### Environment Isolation

Sessions inherit from system environment but can override:

```typescript
const session = await daemon.call("session.create", {
  projectId: "picalyze",
  workingDirectory: "/Users/jonas/repos/picalyze",
  environment: {
    NODE_ENV: "development",
    DATABASE_URL: "postgres://localhost:5432/picalyze_dev",
    // API keys loaded from project-specific secrets
    OPENAI_API_KEY: await loadSecret("picalyze", "OPENAI_API_KEY"),
  }
});
```

---

## 4. Structured Output Channels

### Beyond stdout/stderr

Traditional processes have two output channels. This is insufficient for:
- Structured logs (JSON)
- Metrics and telemetry
- Progress indicators
- Agent-specific metadata

### Channel Architecture

```typescript
interface OutputChannel {
  name: string;           // "stdout", "stderr", "logs", "metrics", "agent"
  format: "raw" | "lines" | "json" | "jsonl";
  source: "fd" | "file" | "socket" | "structured";
}

interface ExecutionResult {
  exitCode: number;
  duration: number;

  channels: {
    stdout: string;                    // Raw output
    stderr: string;                    // Raw errors
    logs?: StructuredLog[];            // Parsed JSON logs
    metrics?: Metric[];                // Timing, counts, etc.
    agent?: AgentMetadata;             // Tool calls, decisions
  };

  // For commands that produce structured output
  structured?: {
    format: "json" | "yaml" | "toml";
    data: unknown;
  };
}
```

### Structured Output Detection

The daemon can automatically detect and parse structured output:

```typescript
// Command: bun test --reporter=json
// Daemon detects JSON output, parses it:
{
  channels: {
    stdout: '{"passed": 42, "failed": 0, ...}',
    structured: {
      format: "json",
      data: { passed: 42, failed: 0, tests: [...] }
    }
  }
}
```

### Integration with OpenTelemetry

Executions can emit OpenTelemetry-compatible telemetry:

```typescript
interface ExecutionSpan {
  traceId: string;
  spanId: string;
  parentSpanId?: string;

  name: string;           // "exec: npm test"
  startTime: number;
  endTime: number;

  attributes: {
    "exec.command": string;
    "exec.exit_code": number;
    "exec.session_id": string;
    "exec.project_id": string;
  };

  events: SpanEvent[];    // Output chunks, errors, etc.
}
```

This allows:
- Correlation between agent decisions and executions
- Tracing across multi-step workflows
- Integration with existing observability tools

---

## 5. Integration Strategies

### Claude Code Integration

**Current state:** Claude Code has a Bash tool that executes commands and returns results. It manages timeouts, sandboxing, and output truncation internally.

**Integration approach:** Claude Code could delegate execution to the daemon:

```typescript
// Claude Code Bash tool implementation
async function executeBash(command: string, options: BashOptions) {
  const daemon = await connectToDaemon();
  const session = await daemon.getOrCreateSession(getCurrentProject());

  return daemon.call("exec.run", {
    sessionId: session.id,
    command,
    timeout: options.timeout,
    outputMode: "buffered",
  });
}
```

**Benefits:**
- Unified execution across all tools
- Project context preserved
- Execution history centralized
- Resource limits enforced consistently

### MCP Server Mode

The daemon can expose an MCP server interface:

```typescript
// MCP tool definitions
const tools = [
  {
    name: "execute",
    description: "Execute a shell command in the project context",
    inputSchema: {
      type: "object",
      properties: {
        command: { type: "string" },
        timeout: { type: "number" },
        background: { type: "boolean" }
      }
    }
  },
  {
    name: "get_output",
    description: "Get output from a background execution",
    inputSchema: {
      type: "object",
      properties: {
        executionId: { type: "string" }
      }
    }
  }
];
```

This allows any MCP-compatible agent to use the daemon without custom integration.

### Terminal Bridge

For humans who still want terminal UIs:

```text
┌─────────────────────────────────────────────┐
│  Terminal Emulator (any: iTerm, Ghostty)    │
├─────────────────────────────────────────────┤
│  Terminal Bridge Process                    │
│  - Connects to daemon session               │
│  - Spawns PTY through daemon                │
│  - Renders output to terminal               │
│  - Forwards input to daemon                 │
├─────────────────────────────────────────────┤
│  Execution Daemon                           │
└─────────────────────────────────────────────┘
```

Usage:
```bash
# Instead of opening a new terminal
stead shell picalyze

# This connects to the picalyze session through the daemon
# All commands are tracked, output is captured, ports are managed
```

### Editor Integration (ACP)

The daemon can implement ACP to integrate with Zed, JetBrains, etc.:

```typescript
// ACP methods the daemon would implement
{
  "acp/session/create": async (params) => {
    const session = await daemon.createSession(params);
    return { sessionId: session.id };
  },

  "acp/tool/execute": async (params) => {
    return daemon.call("exec.run", {
      sessionId: params.sessionId,
      command: params.command,
    });
  }
}
```

---

## 6. Hard Problems

### 6.1 PTY Compatibility

**Problem:** Many CLI tools detect whether they're running in a TTY and change behavior:
- `git` shows colors and pagers
- `npm` shows progress bars
- Interactive prompts expect terminal input

**Challenge:** The daemon needs to support both:
- Pure execution (no PTY, structured output)
- PTY emulation (for interactive tools)

**Potential solutions:**
1. **Selective PTY**: Detect commands that need PTY, spawn with PTY only for those
2. **Force non-interactive**: Set environment variables (`CI=true`, `TERM=dumb`) to disable TTY features
3. **Hybrid mode**: Capture PTY output but also parse structured data from it

**Recommendation:** Default to non-PTY execution with environment hints. Provide explicit PTY mode for interactive use.

### 6.2 Shell State Persistence

**Problem:** Shell state (aliases, functions, environment changes) doesn't persist between commands in most execution models.

```bash
# Command 1
export FOO=bar
cd /some/dir

# Command 2 (different process)
echo $FOO  # Empty! cd didn't persist!
```

**Current workarounds:**
- Claude Code resets CWD between commands explicitly
- Each command runs in isolation

**Potential solutions:**
1. **Stateful shell process**: Keep a shell process alive, send commands to it
2. **State extraction**: After each command, extract CWD/env and replay on next
3. **Explicit state**: Require callers to specify full context each time (current approach)

**Recommendation:** Keep explicit state model. Agents work better with explicit context. Provide session-level defaults that can be overridden per-execution.

### 6.3 Process Supervision

**Problem:** Long-running processes (dev servers, watchers) need supervision:
- Restart on crash
- Log rotation
- Resource monitoring
- Graceful shutdown

**Challenge:** This overlaps with systemd/supervisord/pm2 territory.

**Potential solutions:**
1. **Delegate to existing tools**: Use systemd user services or pm2 under the hood
2. **Basic built-in supervision**: Simple restart policies, minimal features
3. **Hybrid**: Own supervision for dev workflows, delegate to system tools for production

**Recommendation:** Start with basic built-in supervision (restart on crash, timeout). Don't try to replace systemd. Focus on dev-time workflows.

### 6.4 Sandboxing & Security

**Problem:** Agents executing arbitrary commands is dangerous. Need boundaries.

**Concerns:**
- File system access (don't delete system files)
- Network access (don't exfiltrate data)
- Resource consumption (don't fork bomb)
- Privilege escalation (don't sudo)

**Potential solutions:**
1. **User-level sandboxing**: Run as unprivileged user, use filesystem permissions
2. **macOS Sandbox**: Use `sandbox-exec` with profiles
3. **Container-based**: Each session runs in a container (heavyweight)
4. **Allow-list approach**: Only permit certain commands/paths

**Recommendation:** Start with user-level + allow-list. Add macOS Sandbox profiles for sensitive operations. Don't require containers for basic use.

### 6.5 Performance

**Problem:** Adding a daemon layer adds latency.

**Concerns:**
- IPC overhead for every command
- Memory usage of daemon process
- Startup time for daemon

**Benchmarks to target:**
- Command execution overhead: <10ms
- Daemon memory baseline: <50MB
- Cold start time: <500ms

**Mitigations:**
- Unix sockets are fast (nanosecond IPC)
- Lazy loading of per-project contexts
- Optional: keep hot sessions in memory, evict idle ones

### 6.6 Cross-Platform Support

**Problem:** Unix sockets, PTY APIs, process management all differ across platforms.

**macOS specifics:**
- BSD-style PTY (handled by standard APIs)
- launchd for daemon management
- Sandbox profiles for security

**Linux specifics:**
- Unix 98 PTY
- systemd for daemon management
- Namespaces/cgroups for isolation

**Windows:**
- Named pipes instead of Unix sockets
- ConPTY for terminal emulation
- Windows Services for daemons
- Fundamentally different process model

**Recommendation:** Start macOS-only (Jonas's environment). Design abstractions that could support Linux. Ignore Windows initially—agent workflows on Windows are rare.

---

## 7. Implementation Approach

### Phase 1: Minimal Viable Daemon

**Goal:** Replace Claude Code's Bash tool execution with daemon-backed execution.

**Scope:**
- Single Unix socket
- Basic session management (create, destroy, list)
- Synchronous command execution
- Stdout/stderr capture
- Exit code and duration tracking
- Timeout support

**Non-scope:**
- PTY support
- Background processes
- Structured output parsing
- Port management

**Implementation:**
```text
Language: TypeScript (Bun runtime)
Why: Fast startup, good async primitives, same ecosystem as target users
```

### Phase 2: Project Sessions

**Goal:** Project-scoped execution contexts with environment isolation.

**Scope:**
- Per-project sessions with environment overrides
- Working directory management
- Basic port allocation
- Session persistence across daemon restarts

### Phase 3: Structured Output

**Goal:** Rich output beyond stdout/stderr.

**Scope:**
- JSON/JSONL output detection and parsing
- OpenTelemetry span emission
- Output channel multiplexing
- Execution history and replay

### Phase 4: Interactive Support

**Goal:** Support interactive tools and terminal visualization.

**Scope:**
- PTY spawning and management
- Terminal bridge for human use
- Input forwarding
- Screen size management

### Phase 5: Integration Layer

**Goal:** First-class integration with Claude Code, MCP, ACP.

**Scope:**
- MCP server implementation
- ACP adapter
- Claude Code hook integration
- Editor extensions

---

## 8. Open Questions

### Architectural

1. **Should sessions be tied to projects, or to agents?**
   - Project-centric: Multiple agents share a session
   - Agent-centric: Each agent gets isolated session
   - Hybrid: Project sessions with agent sub-contexts

2. **How should the daemon discover projects?**
   - Explicit registration
   - Git repo detection
   - Workspace file detection
   - All of the above

3. **Should the daemon manage secrets?**
   - Pro: Convenient, project-scoped secrets
   - Con: Security complexity, duplication with existing tools

### Technical

4. **What's the right process model for long-running sessions?**
   - Single daemon process with multiplexed sessions
   - Process-per-session
   - Hybrid with pooling

5. **How to handle command output that's very large?**
   - Truncation (current Claude Code approach)
   - Streaming with backpressure
   - File-based storage with refs

6. **Should we use Nushell's structured data model?**
   - Pro: Native structured output, powerful pipelines
   - Con: Another shell to learn, compatibility concerns

---

## 9. Related Work & References

### Process Supervision
- [systemd](https://systemd.io/) - Linux init and service manager
- [supervisord](https://supervisord.org/) - Process control system
- [s6](https://skarnet.org/software/s6/why.html) - Minimal supervision suite
- [pm2](https://pm2.keymetrics.io/) - Node.js process manager

### Shells & Structured Data
- [Nushell](https://www.nushell.sh/) - Shell with structured data pipelines
- [Elvish](https://elv.sh/) - Expressive shell with structured data
- [Murex](https://murex.rocks/) - Shell with typed pipes

### Protocols
- [MCP](https://modelcontextprotocol.io/) - Model Context Protocol (Anthropic)
- [ACP](https://agentclientprotocol.com/) - Agent Client Protocol (Zed)
- [LSP](https://microsoft.github.io/language-server-protocol/) - Language Server Protocol
- [JSON-RPC 2.0](https://www.jsonrpc.org/specification) - Transport protocol

### PTY Libraries
- [node-pty](https://github.com/microsoft/node-pty) - Node.js PTY bindings
- [pilotty](https://github.com/msmps/pilotty) - Daemon-managed PTY for AI agents

### Agent Frameworks
- [Claude Code](https://github.com/anthropics/claude-code) - Anthropic's coding agent
- [libtmux](https://github.com/tmux-python/libtmux) - Python tmux control
- [CrewAI](https://docs.crewai.com/) - Multi-agent orchestration

### Observability
- [OpenTelemetry](https://opentelemetry.io/) - Observability framework

---

## 10. Summary

The Execution Daemon inverts the traditional model:

| Traditional | Execution Daemon |
|-------------|------------------|
| Terminal UI with shell backend | Shell daemon with optional terminal UI |
| Global shell sessions | Project-scoped execution contexts |
| Raw text output (stdout/stderr) | Structured output channels |
| Manual process management | Supervised, observable executions |
| Ad-hoc integration | Protocol-first (MCP, ACP, JSON-RPC) |

**Key insight:** Agents are the primary consumers of command execution. Humans are secondary. Design for agents first, add human interfaces as views.

**Buildable today:** A Bun/TypeScript daemon with:
- Unix socket + JSON-RPC interface
- Project sessions with environment isolation
- Synchronous execution with structured results
- Hook into Claude Code's execution path

**Hard problems to solve later:**
- PTY compatibility without full terminal emulation
- Process supervision without reinventing systemd
- Sandboxing without containerization overhead
- Cross-platform support (especially Windows)

**Next step:** Build Phase 1 minimal daemon and integrate with a single Claude Code workflow to validate the model.
