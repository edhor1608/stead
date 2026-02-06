# stead

An operating environment for agent-driven development.

Modern dev workflows involve multiple AI agents running across projects simultaneously. Your tools organize by app (terminal tabs, browser windows, IDE panels) — not by project. When an agent finishes, you hear a *ding* but can't find which terminal, which browser tab, which port. Context is fragmented. Attention is interrupted without restoration.

stead solves this by providing **contracts** (verified units of work), **session browsing** (unified view across AI CLIs), and a **control room** (macOS app that surfaces what needs your attention).

## Quick Start

```bash
# Build from source
cd rust && cargo build --release

# Install the binary
cp target/release/stead ~/.local/bin/

# Run a contract with verification
stead run "fix the login bug" --verify "cargo test --lib auth"

# List contracts
stead list

# Browse AI sessions across Claude Code, Codex CLI, OpenCode
stead session list
```

## Concepts

### Contracts

A contract is a unit of work with a verification command. Unlike tasks (human-readable descriptions), contracts are executable: they have inputs, verification criteria, and state transitions.

```bash
stead create "add rate limiting" --verify "cargo test rate_limit"
stead claim abc1 --owner agent-1
stead verify abc1
stead cancel abc1
```

**10-state lifecycle:**

```
Pending → Ready → Claimed → Executing → Verifying → Completed
                                      ↘ Failed → (retry) → Executing
                               Cancelled ← (any non-terminal)
                               RollingBack → RolledBack
```

### Sessions

stead reads session data from AI coding CLIs installed on your machine:

| CLI | Session Location |
|-----|-----------------|
| Claude Code | `~/.claude/projects/` |
| Codex CLI | `~/.codex/sessions/` |
| OpenCode | `~/.local/share/opencode/storage/` |

Sessions are normalized into a Universal Session Format (USF) for unified browsing.

```bash
stead session list                          # all sessions
stead session list --cli claude             # filter by CLI
stead session list --project stead          # filter by project
stead session show <session-id>             # full timeline
```

### Control Room (macOS)

Native SwiftUI app with menu bar presence. Surfaces contracts by attention priority:

**Failed > Executing > Verifying > Claimed > Ready > Pending > Completed**

Built with UniFFI bindings — the Swift app calls Rust directly, no IPC.

## Architecture

```
rust/
├── stead-core/        # Library: contracts, storage, USF, commands
├── stead-cli/         # Binary: thin clap wrapper
└── stead-ffi/         # UniFFI bindings for Swift

macos/
└── Stead/             # SwiftUI macOS app (xcodegen)
```

All business logic lives in `stead-core`. The CLI and FFI are thin wrappers.

**Storage:** SQLite database at `.stead/stead.db` per project, with automatic migration from legacy JSONL format.

## CLI Reference

| Command | Description |
|---------|-------------|
| `stead run <task> --verify <cmd>` | Create, execute, and verify a contract |
| `stead create <task> --verify <cmd>` | Create a contract without executing |
| `stead list [--status <s>]` | List contracts, optionally filtered |
| `stead show <id>` | Show contract details |
| `stead verify <id>` | Re-run verification |
| `stead claim <id> --owner <name>` | Claim a contract for execution |
| `stead cancel <id>` | Cancel a non-terminal contract |
| `stead session list` | Browse sessions across AI CLIs |
| `stead session show <id>` | Show session timeline |

All commands accept `--json` for machine-readable output.

## Development

```bash
cd rust

# Build
cargo build --workspace

# Test (114 tests: 98 unit + 16 integration)
cargo test --workspace

# Lint
cargo fmt --all --check
cargo clippy --workspace -- -D warnings

# Build macOS app
cd ../macos && ./build.sh
```

CI runs on every push to main: format check, clippy, tests, and release build.

## License

MIT
