# stead

An operating environment for agent-driven development.

stead combines a contract lifecycle engine, session normalization across coding CLIs, and a macOS Control Room so supervision stays project-centered instead of tool-centered.

## Quick Start

```bash
# Build from source
cd rust && cargo build --release

# Install the binary
cp target/release/stead ~/.local/bin/

# Status-first entry (default)
stead
stead --json

# Grouped command families
stead contract list
stead session list
stead resource endpoint list
stead attention status
stead context generate --task "Restore context"
stead module list
stead daemon health
```

## Core Concepts

### Contracts

Contracts are machine-valid work units with explicit lifecycle transitions.

10-state lifecycle:

`Pending -> Ready -> Claimed -> Executing -> Verifying -> Completed`

Failure/rollback branches:

`Failed`, `RollingBack`, `RolledBack`, `Cancelled`

### Sessions (USF)

stead parses Claude, Codex, and OpenCode session artifacts into a shared Universal Session Format so listing/filtering works through one contract.

### Control Room (macOS)

The macOS app is a daemon client over UniFFI-backed Rust APIs and projects engine state into attention-oriented supervision views.

## Architecture

The rewrite is modular and exportable by crate:

```text
rust/
├── stead-contracts/     # lifecycle + event-sourced contract engine
├── stead-resources/     # generic resource lease/conflict model
├── stead-endpoints/     # named localhost endpoint negotiation
├── stead-usf/           # session adapters + normalized listing contracts
├── stead-module-sdk/    # optional modules: session proxy + context generator
├── stead-daemon/        # versioned command/event API over core crates
├── stead-cli/           # grouped CLI families, daemon-backed dispatch
└── stead-ffi/           # UniFFI bridge for macOS
```

Storage is workspace-local under `.stead/`.

## CLI Surface (v1)

| Family | Commands |
| --- | --- |
| `stead` | status overview (default) |
| `stead contract` | `create`, `get`, `list`, `transition` |
| `stead session` | `list`, `show`, `parse`, `endpoint` |
| `stead resource` | `claim`, `endpoint claim`, `endpoint list`, `endpoint release` |
| `stead attention` | `status` |
| `stead context` | `generate` |
| `stead module` | `list`, `enable`, `disable` |
| `stead daemon` | `health` |

All commands support `--json` for machine consumption.

## Development

```bash
cd rust
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

macOS tests:

```bash
xcodebuild -project macos/Stead/Stead.xcodeproj -scheme Stead -destination 'platform=macOS' test
```

## Documentation Authority

1. `/Users/jonas/repos/stead/docs/plans/planning-baseline-2026-02-13.md`
2. `/Users/jonas/repos/stead/docs/plans/canonical-decisions-2026-02-11.md`
3. `/Users/jonas/repos/stead/docs/plans/docs-authority-map.md`

## License

MIT

<!-- status:start -->
## Status
- State: active
- Summary: Define current milestone.
- Next: Define next concrete step.
- Updated: 2026-02-21
- Branch: `rewrite/v1`
- Working Tree: dirty (2 files)
- Last Commit: 0c55ef9 (2026-02-16) rewrite+tdd: drop stead-core from active workspace surface
<!-- status:end -->
