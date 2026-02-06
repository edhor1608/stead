# stead (Rust)

Rust implementation of the stead CLI and core library.

## Workspace Structure

Cargo workspace with three crates:

```
rust/
├── Cargo.toml              # Workspace manifest
├── stead-core/             # Library — all logic lives here
│   └── src/
│       ├── lib.rs          # Public API
│       ├── cli/            # CLI argument definitions (clap)
│       ├── schema/         # Contract types and 10-state lifecycle
│       ├── storage/        # SQLite persistence (with JSONL migration)
│       ├── usf/            # Universal Session Format
│       │   ├── schema.rs   # Canonical session types
│       │   └── adapters/   # Claude Code, Codex CLI, OpenCode
│       └── commands/       # Command implementations
├── stead-cli/              # Binary — thin clap wrapper
│   ├── src/main.rs
│   └── tests/
│       └── integration.rs  # CLI integration tests
└── stead-ffi/              # UniFFI bindings for Swift/macOS
    └── src/lib.rs
```

**stead-core** contains all business logic: contracts, storage, USF adapters, and command handlers. It exposes a public API that any consumer (CLI, FFI, tests) can use.

**stead-cli** is a thin clap-based binary that parses arguments and delegates to `stead_core`.

**stead-ffi** provides UniFFI-based Swift bindings for the macOS Control Room app.

## Building

```bash
cd rust
cargo build --workspace
```

Release build for macOS aarch64:
```bash
cargo build --release --target aarch64-apple-darwin
```

## Testing

```bash
cargo test --workspace
```

114 tests: 98 unit tests (alongside source in stead-core) + 16 integration tests (CLI invocations via `assert_cmd` in stead-cli).

## Linting

```bash
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
```

## Modules

### Contracts (`schema/`)

Unit of work with verification. 10-state lifecycle with transition guards:

```
Pending → Ready → Claimed → Executing → Verifying → Completed
                                      ↘ Failed → (retry)
                               Cancelled ← (any non-terminal)
                               RollingBack → RolledBack
```

Fields: task, verification command, status, owner, blocked_by, blocks, output, timestamps.

### Storage (`storage/`)

SQLite database at `.stead/stead.db`. Automatic migration from legacy JSONL format on first access.

### USF — Universal Session Format (`usf/`)

Canonical representation for AI coding CLI sessions. Adapters for:
- **Claude Code** — parses `~/.claude/projects/` JSONL files
- **Codex CLI** — parses `~/.codex/sessions/` JSONL files
- **OpenCode** — parses `~/.local/share/opencode/storage/` JSON files

### Commands (`commands/`)

- `run` — Create and execute a contract with verification
- `create` — Create a contract without executing it (stays Pending)
- `list` — List contracts with optional status filter
- `show` — Display contract details (including owner, dependencies)
- `verify` — Re-run verification for a contract
- `claim` — Claim a contract for execution (auto-transitions Pending→Ready→Claimed)
- `cancel` — Cancel a non-terminal contract
- `session list` — List sessions from all installed AI CLIs
- `session show` — Show session details with timeline

## CI

GitHub Actions workflow at `.github/workflows/ci.yml`:
- `cargo fmt --all --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Release build for macOS aarch64 with artifact upload
