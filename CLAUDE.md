# stead

Project exploring solutions to parallel project cognitive overhead.

## ⚠️ Before Any Decision

**Read [NORTH_STAR.md](NORTH_STAR.md) first.**

Every architectural choice, tech selection, or feature must trace back to solving Theo's *ding* problem. If it doesn't reduce context fragmentation, interrupt pain, or resource collisions — question whether it belongs.

## Project Structure

```
docs/
  research/       # Problem analysis, user research, prior art
  plans/          # Specs, architecture decisions, roadmaps
```

## Key Files

- `docs/research/problem-analysis.md` - Core problem breakdown
- `docs/plans/decisions-log.md` - ADR-style decision tracking

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

Research/exploration phase. Scope TBD.
