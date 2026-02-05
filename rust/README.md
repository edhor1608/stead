# stead (Rust)

Rust implementation of the stead CLI and core library.

## Workspace Structure

After M2 restructuring, this is a Cargo workspace:

```
rust/
├── Cargo.toml          # Workspace manifest
├── stead-core/         # Library — all logic lives here
│   └── src/
│       ├── lib.rs      # Public API
│       ├── schema/     # Contract types and lifecycle
│       ├── storage/    # JSONL persistence (SQLite in M3)
│       ├── usf/        # Universal Session Format
│       └── commands/   # Command implementations
├── stead-cli/          # Binary — thin wrapper calling stead-core
│   └── src/main.rs
└── tests/
    └── integration.rs
```

**stead-core** contains all business logic: contracts, storage, USF adapters, and command handlers. It exposes a public API that any consumer (CLI, FFI, tests) can use.

**stead-cli** is a thin clap-based binary that parses arguments and delegates to `stead_core`.

## Building

```bash
cd rust
cargo build --workspace
```

## Testing

```bash
cargo test --workspace
```

88 tests: 72 unit tests in stead-core, 16 integration tests.

## Modules

### Contracts (`schema/`)

Unit of work with verification. States: Pending, Running, Passed, Failed.

### Storage (`storage/`)

JSONL-based append-only persistence in `.stead/contracts.jsonl`.

### USF — Universal Session Format (`usf/`)

Canonical representation for AI coding CLI sessions. Adapters for:
- **Claude Code** — parses `~/.claude/projects/` JSONL files
- **Codex CLI** — parses `~/.codex/sessions/` JSONL files
- **OpenCode** — parses `~/.local/share/opencode/storage/` JSON files

### Commands (`commands/`)

- `run` — Create and execute a contract with verification
- `list` — List contracts with optional status filter
- `show` — Display contract details
- `verify` — Re-run verification for a contract
- `session list` — List sessions from all installed AI CLIs
- `session show` — Show session details with timeline

## CI

GitHub Actions workflow at `.github/workflows/ci.yml`:
- `cargo fmt --check`
- `cargo clippy --workspace`
- `cargo test --workspace`
- Release build for macOS aarch64
