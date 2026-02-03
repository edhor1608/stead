# Decision: First Slice

> **Historical Note:** The first slice was initially implemented in TypeScript/Bun (v1, merged February 2026). This document has been updated to reflect the v2 rewrite in Rust. The scope and success criteria remain valid; only the implementation technology changed.

## Context

We're scoping the first buildable slice of stead — an operating environment for agent-driven development.

Constraints:
- Must be buildable in ~2 weeks
- Must prove the core model (contracts > tasks, verification-first)
- Must demonstrate value immediately
- Must work with Claude Code as first agent
- Must not require all 6 pillars to work

The 6 pillars (Contract Engine, Execution Daemon, Session Proxy, Control Room, Transformation Layer, Context Generator) were designed to be buildable independently. The decision here is: what's the minimal contract engine that proves the model?

## The Slice

**A CLI that wraps Claude Code tasks in contracts with automated verification.**

```bash
stead run "fix the login bug" --verify "bun test auth"
```

What happens:
1. CLI creates a contract (YAML file in `.stead/contracts/`)
2. CLI invokes Claude Code with the task
3. When Claude Code completes, CLI runs the verification command
4. Contract is marked passed/failed based on verification result
5. Contract file persists with full history (input, output, verification result)

That's it. No daemon. No UI. No browser. Just: **contract creation + Claude Code invocation + verification + persistence.**

## What's IN Scope

1. **Contract schema (minimal)**
   - `id`: unique identifier
   - `task`: human-readable description (passed to Claude Code)
   - `verification`: command that exits 0 on success
   - `status`: `pending` | `running` | `passed` | `failed`
   - `created_at`, `completed_at`, timestamps
   - `output`: captured stdout/stderr from verification

2. **CLI commands**
   - `stead run "<task>" --verify "<cmd>"` — create contract + run + verify
   - `stead list` — show contracts and their status
   - `stead show <id>` — show contract details
   - `stead verify <id>` — re-run verification on existing contract

3. **Contract storage**
   - JSONL file at `.stead/contracts.jsonl`
   - Append-only, git-trackable, machine-readable
   - No database

4. **Claude Code integration**
   - Shell out to `claude` CLI
   - Pass the task as input
   - Wait for completion
   - Capture exit status

## What's OUT of Scope

- **No UI** — CLI only for first slice
- **No daemon** — synchronous execution, one contract at a time
- **No parallelism** — sequential contracts only
- **No dependencies** — contracts can't depend on other contracts yet
- **No rollback** — verification failure just marks contract as failed
- **No transformation output** — agents can't propose project changes yet
- **No context generation** — no automatic briefing synthesis
- **No session proxy** — browser identity isolation is a later pillar
- **No port management** — resource allocation is a later concern
- **No input specification** — task is just a string for now (not typed schema)
- **No subagent support** — just main Claude Code invocation

## Success Criteria

1. **Can wrap any Claude Code task in a contract in <10 seconds**
   - `stead run` should feel instant to start

2. **Verification runs automatically without human intervention**
   - No "did you check if it worked?" — the CLI does it

3. **Contract history shows what was attempted and whether it worked**
   - `stead list` gives immediate visibility into agent work

4. **Contracts are git-trackable**
   - Push `.stead/` to repo, team sees contract history

5. **Dogfood test**: Jonas uses this for real work within 1 week
   - Not a demo — actual tasks on active projects

## Why This Slice

**It proves the core insight with minimal machinery.**

The insight: agents need contracts (input/output/verify), not tasks (descriptions). Current Claude Code workflow:

```text
Run agent → *ding* → manually check if it worked → manually track what was done
```

After this slice:

```text
stead run → agent runs → verification runs → contract persists with result
```

This is the "hello world" of verification-first development:
- No human verification needed for programmatic checks
- History of what agents did and whether it succeeded
- Foundation for everything else (UI shows contracts, daemon runs contracts, etc.)

**Why not start with Control Room UI?**

UI shows contracts. If we don't have contracts, what does the UI show? Building UI first is putting the cart before the horse. The contract engine (even minimal CLI form) is the foundation.

**Why not start with Execution Daemon?**

Daemon runs multiple things in parallel. We don't need parallelism to prove the model works. Sequential execution + verification is enough. Daemon comes when we need concurrency.

**Why contracts in YAML files, not a database?**

- Human-readable (debug by reading)
- Git-trackable (audit trail for free)
- No server to run
- Can always migrate to SQLite later if needed

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Claude CLI doesn't exist or is unstable | Test early. If blocked, mock it for v1 and treat as external dependency. |
| Verification is too hard to express | Start with command-based (`exit 0 = pass`). Can add assertion DSL later. |
| YAML contracts get messy | Keep schema minimal. Add complexity only when proven needed. |
| Scope creep to "just add X" | This doc is the scope. Anything not IN SCOPE requires a new decision. |
| Takes longer than 2 weeks | Cut to even smaller: just `run` and `list`. No `show`, no `verify`. |
| Rust learning curve | Start with minimal features. Leverage compiler to catch errors. |

## Implementation Notes

**Tech stack (v2):**
- Rust (performance, single binary, type safety)
- JSONL storage (append-only, as per contract-schema-format decision)
- clap for CLI argument parsing

**v1 implementation (historical):**
- TypeScript/Bun
- YAML storage (simpler for debugging, migrating to JSONL in v2)

**Open questions to resolve during build:**
- Exact contract YAML schema
- How to capture Claude Code output (pipe vs file)
- Whether to store diffs/changes made by agent

These are implementation details, not scope decisions.

---

## The Test

If this slice works, we can answer:

> "What did agents do on this project, and did it work?"

That's the foundation. Everything else (UI, daemon, parallelism, context, transformations) builds on being able to answer that question.
