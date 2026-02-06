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
      schema/               # Contract types and lifecycle
      storage/              # JSONL persistence (SQLite in M3)
      usf/                  # Universal Session Format adapters
      commands/             # Command implementations
  stead-cli/                # Binary — thin clap wrapper
    src/main.rs
    tests/integration.rs    # CLI integration tests
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

## Status

Rust implementation in progress. M1 (USF adapters) and M2 (workspace restructure) complete. M3 (SQLite storage) next.
