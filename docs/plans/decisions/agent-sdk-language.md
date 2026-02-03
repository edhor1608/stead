# Decision: Agent SDK Language

## Context

stead needs an interface for AI agents to interact with the Contract Engine:
- Claim contracts
- Report status/progress
- Propose transformations
- Query context
- Signal completion/failure

Constraints:
- Claude Code (dominant agent) runs as Node.js but interacts via bash tools + file operations
- Agents are not programs that import libraries — they're LLMs that use tools
- Future agents might run in different runtimes (Python, Rust, whatever)
- Contract Engine is TypeScript/Effect-TS

The question: what form should this SDK take?

## Decision

**Protocol-first, not language-first.**

The "SDK" is a CLI tool (`stead`) with a clean HTTP API underneath. No language-specific library as the primary interface.

```text
Primary:     stead CLI (what agents actually invoke)
Secondary:   HTTP/JSON API (what the CLI calls)
Optional:    TypeScript types package (for programmatic access if needed)
```

## Rationale

### Agents don't import libraries

Claude Code doesn't do:
```typescript
import { claimContract } from '@stead/sdk';
await claimContract('fix-bug-123');
```

It does:
```bash
stead contract claim fix-bug-123
```

The bash tool IS the SDK from the agent's perspective. Optimizing for `import` statements solves the wrong problem.

### Language-agnostic by default

A CLI works for:
- Claude Code (Node.js host, uses bash)
- Aider (Python, uses subprocess)
- Cursor/Copilot (VS Code extensions, can shell out)
- Future agents we haven't imagined

A TypeScript SDK would require Python/Rust/Go ports. A CLI works everywhere.

### Structured output for machine consumption

The CLI outputs JSON by default (agents parse it). Human-readable output is opt-in (`--human`).

```bash
$ stead contract list
{"contracts":[{"id":"fix-123","status":"available",...}]}

$ stead contract list --human
ID        STATUS     PRIORITY
fix-123   available  high
```

### HTTP API enables everything else

The CLI is a thin wrapper around HTTP calls. This means:
- Browser-based control room uses same API
- Language bindings can be added later (just HTTP calls)
- Debugging is `curl`-able

## API Surface

### CLI Commands (what agents use)

```bash
# Contract lifecycle
stead contract list [--status=available|claimed|completed]
stead contract claim <id>
stead contract status <id> --progress=0.5 --message="Running tests"
stead contract complete <id> --result=<json>
stead contract fail <id> --reason="Tests failed"
stead contract rollback <id>

# Transformations (for git layer)
stead transform propose <contract-id> --type=rename --args='{"from":"oldFn","to":"newFn"}'
stead transform list <contract-id>
stead transform apply <transform-id>

# Context (from context generator)
stead context get <contract-id>          # Get synthesized context for this contract
stead context query "what auth lib?"      # Ask the project mind

# Project state
stead project status                       # What's running, what needs attention
stead project ports                        # Allocated ports
```

### HTTP API (what CLI calls)

```text
POST   /contracts/:id/claim
PATCH  /contracts/:id/status
POST   /contracts/:id/complete
POST   /contracts/:id/fail
POST   /contracts/:id/rollback

POST   /transforms
GET    /transforms?contract=:id
POST   /transforms/:id/apply

GET    /context/:contractId
POST   /context/query

GET    /project/status
GET    /project/ports
```

### Optional: TypeScript Types Package

For humans building integrations or testing:

```typescript
// @stead/types - just types, no runtime
interface Contract {
  id: string;
  input: unknown;
  output: { schema: JSONSchema };
  verify: VerificationSpec;
  rollback: RollbackProcedure;
}

interface TransformProposal {
  type: 'rename' | 'move' | 'extract' | ...;
  args: Record<string, unknown>;
}
```

This is NOT the agent SDK. It's for humans writing tooling.

## Trade-offs

### Gains

- **Universal compatibility** — Works with any agent that can shell out
- **Debuggable** — Humans can run the same commands agents run
- **Simple** — No dependency management, version conflicts, or import issues
- **Evolvable** — API can change without agents needing to update imports
- **Matches reality** — This is how Claude Code actually works today

### Gives up

- **Type safety at call site** — Agents don't get autocomplete (they don't use autocomplete anyway)
- **In-process performance** — Shell out has overhead (negligible for our use case)
- **Atomic operations** — Can't do `claimAndStart()` in one call (but HTTP API could batch)

### Why not multi-language SDKs?

Maintenance burden for marginal benefit. Agents shell out. If someone really needs Python bindings, they can:
1. Use HTTP API directly
2. We add thin wrapper later

Don't solve hypothetical problems.

## Implementation Notes

The `stead` CLI should be:
- Single binary (Bun compile or similar)
- Fast startup (<50ms)
- JSON-first output
- Installed globally or per-project

The HTTP API runs as part of the stead daemon (which manages execution contexts, ports, etc.).

## Connection to North Star

**Does this reduce the *ding* problem?**
- Indirectly yes — clean contract interface means agents report status properly, control room knows what's happening

**Is this agent-first?**
- Yes — CLI is optimized for machine consumption (JSON output, predictable structure)
- Human mode is explicitly secondary (`--human` flag)

**Is this the simplest solution?**
- Yes — no language-specific libraries to maintain, no version conflicts, universal compatibility

**Does this trace back?**
- Contract Engine → needs agent interface → CLI is how agents interact with systems → this is that CLI
