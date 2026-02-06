# stead

Project exploring solutions to parallel project cognitive overhead.

## ⚠️ Before Any Decision

**Read [NORTH_STAR.md](NORTH_STAR.md) first.**

Every architectural choice, tech selection, or feature must trace back to solving Theo's *ding* problem. If it doesn't reduce context fragmentation, interrupt pain, or resource collisions — question whether it belongs.

## Project Structure

```text
rust/                       # Cargo workspace
  stead-core/               # Library — all business logic
    src/
      cli/                  # CLI argument definitions (clap)
      schema/               # Contract types, 10-state lifecycle
      storage/              # SQLite persistence (auto-migrates from JSONL)
      usf/                  # Universal Session Format adapters
      commands/             # Command implementations (run, create, list, show, verify, claim, cancel, session)
  stead-cli/                # Binary — thin clap wrapper
    src/main.rs
    tests/integration.rs    # CLI integration tests
  stead-ffi/                # UniFFI Swift bindings
macos/
  Stead/                    # SwiftUI Control Room app
docs/
  research/                 # Problem analysis, user research, prior art
  plans/                    # Specs, architecture decisions, roadmaps
.github/workflows/ci.yml   # CI: fmt, clippy, test, build
```

## Key Files

- `docs/research/problem-analysis.md` - Core problem breakdown
- `docs/plans/decisions-log.md` - ADR-style decision tracking
- `rust/README.md` - Rust workspace details, build/test instructions

## Context

The problem: Modern dev workflows involve multiple projects running simultaneously (especially with AI agents). Our tools organize by app, not by project, causing:
- Context fragmentation across terminal/browser/IDE
- Interrupt-driven attention loss from agent notifications
- Resource collisions (ports, auth redirects)
- No way to "surface everything for Project X"

## Working Conventions

- **Document decisions immediately** - Add to `docs/plans/decisions-log.md` as they happen
- **Research goes in docs/research/** - Problem analysis, user interviews, prior art
- **Plans go in docs/plans/** - Specs, architecture, roadmaps

## Build & Test

```bash
cd rust
cargo test --workspace          # 114 tests (98 unit + 16 integration)
cargo clippy --workspace        # zero warnings
cargo fmt --all --check         # formatting
```

## Status

All milestones complete: M1 (USF adapters), M2 (workspace restructure), M3 (SQLite storage), M4 (UniFFI Swift bindings), M5 (SwiftUI Control Room), M6 (10-state contract lifecycle).
