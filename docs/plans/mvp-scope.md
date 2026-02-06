# MVP Scope

**Date:** 2026-02-05
**Status:** Active

---

## The One Thing the MVP Must Prove

A developer hears the *ding*. Instead of hunting through terminal tabs, they glance at a menu bar icon, click it, see what finished, and restore full context in under 10 seconds. That's it. Everything in the MVP exists to make that moment work.

If we nail this one loop -- notification, comprehension, action -- people will tell other people. If we don't, nothing else matters.

---

## What's IN the MVP (5 things)

### 1. stead-core library (Rust)

The brain. All logic lives here. CLI and Mac app are both views into it.

- Contract CRUD: create, list, show, verify (current 4-state lifecycle -- Pending, Running, Passed, Failed)
- USF read adapters: Claude Code, Codex CLI, OpenCode (already built)
- SQLite storage (replaces JSONL -- needed for concurrent CLI + GUI access)
- Clean public API: functions return data, callers decide how to display it

This is M2 + M3 from the phase 2 plan. Foundation. Non-negotiable.

### 2. stead CLI (Rust binary)

Thin wrapper around stead-core. The agent's interface.

- `stead run "task" --verify "cmd"` -- create and execute contract
- `stead list` -- attention-priority ordered contract list
- `stead session list` -- unified sessions across all CLIs
- `stead session show <id>` -- session detail
- JSON output mode for machine consumption (`--json`)

The CLI is the wedge. First thing people install. First impression. Must feel instant (<50ms for any command).

### 3. Menu bar presence (SwiftUI)

The ambient awareness layer. This IS the Control Room MVP -- not a window.

- Menu bar icon with state-encoded color (nothing = calm, dot = something finished, red = decision needed)
- Popover on click: attention-priority list of items that need you (Needs Decision, Completed, Anomalies). Running/Queued as counts only.
- One line per item: project name, summary, key metadata
- Global hotkey: Cmd+Shift+S to toggle

The popover is the primary UI surface. Most interactions are 5 seconds: glance, scan, dismiss or act. The visionary's "disappearing interface" concept -- this IS that.

### 4. Full window for review (SwiftUI)

When the popover isn't enough -- you need to see diffs, make decisions, browse sessions.

- Single-column attention-priority list (not multi-column, not kanban)
- Expandable contract detail: verification result, files changed, duration
- Session browser grouped by project, then by CLI
- Keyboard-first: j/k navigation, Enter to expand, a/r to approve/reject, Esc to close
- Project color coding (auto-assigned from path hash)

This is the deep-work surface. You open it 5% of the time. The other 95%, the popover is enough.

### 5. Notification strategy

Two sounds. Two interrupt cases. Everything else is silent.

- **Decision needed**: macOS notification + soft double-tone. Agent blocked on human input.
- **Critical failure**: macOS notification + low tone. Something broke that affects other work.
- **Completed**: Menu bar badge (no sound, no banner). You notice on your own schedule.
- **Running/Queued**: Silent. Visible in full window only if you choose to look.
- **Focus Mode toggle**: Suppress everything. Agents queue silently. Catch-up summary when you return.

---

## What's OUT (explicitly deferred)

| Feature | Why it's out | When it returns |
|---------|-------------|-----------------|
| **10-state contract lifecycle** (M6) | 4 states cover the MVP supervision loop. Ready/Claimed/Verifying/RollingBack add complexity without proving the core value. | After MVP ships and we validate the model |
| **Contract dependencies** (blocks/blockedBy) | Multi-agent coordination is layer 2. MVP proves single-agent supervision. | M6 |
| **Execution daemon** | We orchestrate existing CLIs. Don't build what already works. | Maybe never -- CLIs ARE the runtimes |
| **Session proxy** (browser isolation) | Solves port/auth collisions, but not the *ding* problem. Separate wedge. | Post-MVP, if port collisions prove to be acute pain |
| **Transformation layer** (semantic git) | Ambitious. Requires tree-sitter, AST transforms, conflict replay. Research project. | Way later |
| **Context generator** | The "project mind." Requires enough contract history to synthesize from. | After we have real usage data |
| **Agent negotiation protocol** | Resource coordination between agents. Needs the resource registry, port allocator. | Layer 2 (visionary concepts) |
| **Spatial audio / peripheral vision strip** | Visionary Layer 1 concepts. Cool. Not needed to prove the core loop. | After menu bar + popover proves the ambient model |
| **Attention thermostat** | Auto-adjusting interrupt level from macOS Focus modes. Elegant but premature. | After Focus Mode toggle proves people want batching |
| **Morning briefing** | Synthesis on session start. Needs context generator + enough history. | Post-MVP |
| **Cost tracking / token budgets** | Useful but not supervisory. Doesn't help you find context when the ding arrives. | Post-MVP |
| **Historical analytics** | Dashboard thinking. Stead is a control room. | Probably never |
| **Multi-user / team features** | Stead is single-user. Jonas's daily driver. | Only if the product grows beyond solo devs |
| **Windows / Linux native apps** | Mac first. Optimize for the daily driver. | If demand materializes |

---

## The First User Journey

**Persona:** Jonas. 5 active repos. Runs Claude Code and OpenCode daily. Feels Theo's *ding* pain.

### Install (30 seconds)

```bash
brew install stead
```

Single binary. No daemon. No config. No account.

### First contact: CLI (2 minutes)

```bash
stead session list
```

Output: every AI CLI session across Claude Code, Codex, and OpenCode. Grouped by project. Sorted by recency. Jonas sees sessions he forgot existed. "Oh, someone finally built this."

```bash
stead run "fix the auth token refresh bug" --verify "cargo test --lib auth"
```

A contract is created. Claude Code runs the task. Verification runs automatically. Result stored in SQLite.

```bash
stead list
```

Output: contracts ordered by attention priority. The auth fix shows "Passed" at the top (most recent completion). The format mirrors what the GUI will show.

### Second contact: Menu bar app (5 minutes)

Jonas opens the Stead macOS app. A menu bar icon appears. Nothing else. He goes back to work.

Three minutes later, an agent finishes. The menu bar icon shows a subtle badge: "1".

He clicks it. Popover: "qwer-q: auth fix complete. 3 files, tests pass."

He clicks the item. Full window opens to the contract detail. Diff is visible. He scans it, hits `a` to approve. Closes the window. 10 seconds total.

Meanwhile, another agent is running. It's in "Running (1)" at the bottom of the popover. He doesn't think about it. It hasn't earned his attention.

### The "wow" moment

The *ding* arrives and for the first time, Jonas doesn't feel a spike of anxiety. He doesn't scramble. He doesn't lose his train of thought. The information came to him, organized and complete.

That's the product.

---

## Success Criteria

### Must-hit (ship-blocking)

1. **`stead session list` returns results in <200ms** with 50+ sessions across 3 CLIs. The CLI must feel instant.

2. **Menu bar icon reflects actual agent state** within 5 seconds of a state change. Badge count is accurate.

3. **Popover-to-full-context restoration takes <10 seconds.** From seeing the badge to having the contract detail open with diff visible.

4. **Zero configuration required.** Install the binary, install the app. Sessions are discovered automatically. No setup wizard, no "connect your CLI" step.

5. **The app doesn't crash.** SQLite access from CLI and GUI simultaneously works without data corruption. Tested.

### Should-hit (quality bar)

6. **Keyboard-only workflow works end-to-end.** Cmd+Shift+S to open, j/k to navigate, Enter to expand, a to approve, Esc to close.

7. **Dark mode and light mode both look right.** System colors, system fonts, nothing custom that breaks.

8. **Focus Mode suppresses all notifications** and provides a catch-up summary on return.

9. **CLI and GUI show identical data.** Same contracts, same order, same vocabulary. No mental model mismatch.

### Aspirational (delight)

10. **Someone screenshots the Control Room and shares it.** The visual product people talk about. This requires the design language to land -- clean, native, confident.

11. **A developer tries it and immediately sends it to a friend who also runs multiple agents.** Word of mouth. The problem is so acute that seeing a solution creates urgency to share.

---

## Technical Milestones (Build Order)

```
M2: Library split (stead-core + stead-cli)
 |   Fix: remove tokio, fix .leak(), fix truncate UTF-8 bug
 |   Decouple commands from I/O (return data, don't println)
 |
M3: SQLite storage
 |   Replace JSONL, concurrent access safe
 |   Migration path for existing data
 |
M4: Swift FFI (UniFFI + cargo-swift)
 |   Expose: list_contracts, get_contract, list_sessions, get_session
 |   DateTime as ISO 8601 strings
 |
M5: SwiftUI Control Room
     Menu bar icon + popover + full window
     Attention-priority ordering
     Keyboard shortcuts
     Notification strategy (2 sounds)
     Focus Mode
```

Each milestone is independently useful. M2-M3 make the CLI better. M4 is the bridge. M5 is the product.

---

## What This Document Does NOT Cover

- Detailed technical specs for each milestone (see `phase2-revised.md`)
- Design language specifics (see `design-language.md`)
- UX wireframes and interaction patterns (see `control-room-ux.md`)
- Architecture decisions and rationale (see `decisions-log.md`)

Those documents remain authoritative for their domains. This document defines the boundary: what's in, what's out, and how we know it works.

---

## The Razor

When debating whether something belongs in the MVP, apply this test:

> Does this feature help a developer go from *ding* to full context in under 10 seconds?

If yes, it's in. If no, it waits. No exceptions. No "while we're at it." No "it would only take a day." The MVP is the supervision loop and nothing else.
