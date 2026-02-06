# stead (Rust)

Rust implementation of the stead CLI and core library.

**Status:** M1 (USF adapters) and M2 (workspace restructure) complete. M3 (SQLite storage) next.

## Workspace Structure

Cargo workspace with two crates:

```
rust/
├── Cargo.toml              # Workspace manifest
├── stead-core/             # Library — all logic lives here
│   └── src/
│       ├── lib.rs          # Public API
│       ├── cli/            # CLI argument definitions (clap)
│       ├── schema/         # Contract types and lifecycle
│       ├── storage/        # JSONL persistence (SQLite in M3)
│       ├── usf/            # Universal Session Format
│       │   ├── schema.rs   # Canonical session types
│       │   └── adapters/   # Claude Code, Codex CLI, OpenCode
│       └── commands/       # Command implementations
└── stead-cli/              # Binary — thin clap wrapper
    ├── src/main.rs
    └── tests/
        └── integration.rs  # CLI integration tests
```

**stead-core** contains all business logic: contracts, storage, USF adapters, and command handlers. It exposes a public API that any consumer (CLI, FFI, tests) can use.

**stead-cli** is a thin clap-based binary that parses arguments and delegates to `stead_core`.

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

Unit tests live alongside source in stead-core. Integration tests (CLI invocations via `assert_cmd`) are in `stead-cli/tests/`.

## Linting

```bash
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
```

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
- `cargo fmt --all --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Release build for macOS aarch64 with artifact upload
