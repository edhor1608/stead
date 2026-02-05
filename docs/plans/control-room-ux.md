# Control Room UX Design

**Created:** 2026-02-05
**Status:** Draft
**Task:** #7

---

## Design Philosophy

The Control Room is not a dashboard. It is an **attention-allocating supervision interface** for autonomous agent systems. The design follows one rule:

> **Earn the human's attention. Never waste it.**

The default state is calm. Screen presence must be justified by supervisory need. Everything traces back to the *ding* problem: when an agent finishes, the human needs instant context restoration -- not a hunt through tabs.

---

## 1. Information Architecture

### Three Levels of Detail

**Level 0: Menu Bar Icon** -- always visible, zero attention cost

The menu bar icon communicates system state with a single glyph. It answers: "Does anything need me?"

| Icon State | Meaning | Visual |
|------------|---------|--------|
| Idle | No agents running | Monochrome stead logo |
| Running | Agents active, nothing needs you | Subtle pulse/dot |
| Attention | Something completed or has anomaly | Filled dot (badge count) |
| Decision | Blocked on human input | Red dot, persistent |

Clicking the menu bar icon opens the **Popover** (Level 1).

**Level 1: Popover** -- quick scan, 5-second interaction

A compact popover (roughly 360px wide, max 500px tall) showing only items that need attention. This is the primary interaction surface for the supervision loop.

```
┌──────────────────────────────────────┐
│  NEEDS DECISION  (1)                 │
│  ┌──────────────────────────────────┐│
│  │ qwer-q  API design: pick one    ││
│  │         3 options  ·  12m ago    ││
│  └──────────────────────────────────┘│
│                                      │
│  COMPLETED  (2)                      │
│  ┌──────────────────────────────────┐│
│  │ picalyze  Auth refactor done    ││
│  │           3 files  ·  tests pass ││
│  ├──────────────────────────────────┤│
│  │ stead  Workspace split done      ││
│  │         8 files  ·  tests pass   ││
│  └──────────────────────────────────┘│
│                                      │
│  RUNNING  (3)        QUEUED  (5)     │
│                                      │
│  ─────────────────────────────────── │
│  Open Control Room         ⌘⇧S      │
└──────────────────────────────────────┘
```

What's shown:
- Attention items only (Needs Decision, Anomalies, Completed)
- Running/Queued as counts only (no detail -- they don't need you)
- One line per item: project name, summary, key metadata
- Footer link to open full window

What's NOT shown:
- Session details, timelines, logs
- Running agent output
- Historical data

**Level 2: Full Window** -- deep work, extended interaction

The full window opens as a standard macOS window (not popover). This is for:
- Reviewing completed work (diffs, test output)
- Making decisions with full context
- Browsing sessions across projects
- Viewing running agent progress (when you choose to look)

Layout: single-column attention-priority list with expandable detail panels.

### Attention Priority Ordering

Everything is sorted by supervisory need, not recency or project:

1. **Needs Decision** -- agent blocked on human input (red accent)
2. **Anomaly** -- unexpected state: errors, timeouts, resource conflicts (amber accent)
3. **Completed** -- finished, awaiting human review (blue accent)
4. **Running** -- executing normally (neutral, collapsed by default)
5. **Queued** -- waiting for dependencies or scheduling (gray, collapsed)

Within each tier, items are sorted by:
- Age within tier (oldest first -- longest-waiting decision is most urgent)
- Then by project (group same-project items together within tier)

### Data at Each Level

| Data | Menu Bar | Popover | Full Window |
|------|----------|---------|-------------|
| Aggregate status | Yes (icon) | Yes (counts) | Yes (counts) |
| Item summaries | No | Yes (1-line) | Yes (multi-line) |
| Project name | No | Yes | Yes |
| Contract details | No | No | Yes |
| Session timeline | No | No | Yes |
| Diff viewer | No | No | Yes |
| Agent output/logs | No | No | Yes |
| Historical data | No | No | Yes |
| Decision options | No | Preview | Full |

---

## 2. Interaction Patterns

### The Supervision Loop

The primary workflow takes 5-15 seconds:

```
Menu bar icon shows badge
  -> Click (or ⌘⇧S)
  -> Popover: scan items (2 seconds)
  -> Click item needing action
  -> Full window: take action (approve/decide/dismiss)
  -> Close or ⌘⇧S to hide
```

The secondary (deep) workflow:
```
Open full window
  -> Review completed work (diffs, tests)
  -> Approve or reject
  -> Check running agents (optional)
  -> Close
```

### Responding to Decisions

When an agent needs a decision, the decision card in the full window shows:

```
┌─────────────────────────────────────────────────────┐
│  NEEDS DECISION                                      │
│  ─────────────────────────────────────────────────── │
│  qwer-q · Contract lx4f-8k2                         │
│                                                      │
│  API Rate Limiting Strategy                          │
│                                                      │
│  Context: Implementing enterprise API. Hit rate      │
│  limits during load testing. Need to decide approach │
│  before continuing.                                  │
│                                                      │
│  ┌─ Option A ─────────────────────────────────────┐  │
│  │ Exponential backoff                            │  │
│  │ Simple, but slow under sustained load          │  │
│  └────────────────────────────────────────────────┘  │
│  ┌─ Option B ─────────────────────────────────────┐  │
│  │ Request queue with batching                    │  │
│  │ Better throughput, more implementation work     │  │
│  └────────────────────────────────────────────────┘  │
│  ┌─ Option C ─────────────────────────────────────┐  │
│  │ Upgrade API tier ($200/mo)                     │  │
│  │ No code change, ongoing cost                   │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  ┌──────────────────────────────────────────────┐    │
│  │ Add comment...                                │    │
│  └──────────────────────────────────────────────┘    │
│                                                      │
│     [Choose Selected]  [Need More Info]  [Cancel]    │
└─────────────────────────────────────────────────────┘
```

Decision types and their UI:
- **Multiple choice**: Selectable option cards (click or arrow keys + Enter)
- **Yes/No**: Two prominent buttons, keyboard `y`/`n`
- **Approval**: Diff view + Approve/Reject buttons, keyboard `a`/`r`
- **Open-ended**: Text input field, Submit on Enter
- **Need more info**: Text field opens, response goes back to agent

### Reviewing Completed Work

Completed items expand to show:

```
┌─────────────────────────────────────────────────────┐
│  COMPLETED                                           │
│  ─────────────────────────────────────────────────── │
│  picalyze · Contract mx9a-3j7 · 4m ago             │
│                                                      │
│  Auth flow refactor                                  │
│                                                      │
│  Verification: cargo test            PASSED          │
│  Files changed: 3  (+47 -23)                        │
│  Duration: 2m 34s                                   │
│                                                      │
│  ┌─ Changes ──────────────────────────────────────┐  │
│  │  M src/auth/middleware.rs     +12 -8           │  │
│  │  M src/auth/session.rs       +28 -15          │  │
│  │  M tests/auth_test.rs        +7  -0           │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  [View Diff]  [View Logs]                           │
│                                                      │
│          [Approve]  [Reject]  [Dismiss]              │
└─────────────────────────────────────────────────────┘
```

"View Diff" opens an inline diff viewer (native NSTextView with syntax highlighting, not a web view). "View Logs" shows the agent's session timeline.

### Starting/Stopping Agents

The full window includes a "New Contract" action (⌘N) that opens a creation form:

```
┌─────────────────────────────────────────────────────┐
│  NEW CONTRACT                                        │
│                                                      │
│  Project:  [ picalyze          ▾ ]                  │
│  CLI:      [ Claude Code       ▾ ]                  │
│  Task:     [ Fix the auth token refresh bug     ]   │
│  Verify:   [ cargo test --lib auth              ]   │
│                                                      │
│                          [Cancel]  [Run Contract]    │
└─────────────────────────────────────────────────────┘
```

Running items show a "Cancel" action. No pause/resume in MVP -- that requires deeper agent integration.

### Keyboard-First Design

Every action is reachable without a mouse.

**Global shortcuts:**
| Key | Action |
|-----|--------|
| `⌘⇧S` | Toggle Control Room (global, works from any app) |

**Within popover:**
| Key | Action |
|-----|--------|
| `j` / `k` | Navigate items |
| `Enter` | Open item in full window |
| `Esc` | Close popover |

**Within full window:**
| Key | Action |
|-----|--------|
| `j` / `k` | Navigate items |
| `Enter` | Expand/collapse item detail |
| `Esc` | Collapse detail / close window |
| `a` | Approve (on completed/decision item) |
| `r` | Reject |
| `d` | Open decision response |
| `x` | Cancel contract |
| `⌘N` | New contract |
| `1-5` | Jump to attention tier (1=Decision, 2=Anomaly, etc.) |
| `/` | Filter/search |
| `?` | Show shortcut overlay |

Keyboard shortcuts are shown inline as hints (e.g., the Approve button shows `A` badge).

---

## 3. Notification Strategy

### The Interrupt Budget

Human attention is finite. Notifications are expensive -- 23 minutes to recover from an interruption (Gloria Mark, UC Irvine research). The notification system respects an **interrupt budget**.

### When to Interrupt (macOS native notification + sound)

Only two cases justify breaking the human's focus:

1. **Blocked on human decision** -- agent cannot continue without input
2. **Critical failure** -- something is broken in a way that affects other work (e.g., port conflict blocking another project, auth token expired affecting multiple agents)

Notification format:
```
stead - qwer-q
API design: pick one of 3 options
[Decide]  [Later]
```

"Decide" opens the Control Room directly to the decision item. "Later" dismisses the notification but the menu bar icon retains the red badge.

### When to Badge (menu bar dot, no sound, no banner)

- Contract completed (awaiting review)
- Anomaly detected (unexpected error, timeout)
- Agent completed but verification failed

The badge count on the menu bar icon shows how many items need attention. No notification banner appears -- the human notices on their own schedule.

### When to Stay Silent (no notification, no badge change)

- Agent started running
- Agent is executing normally
- Contract queued
- Session activity (tool calls, file edits)

These are visible in the full window if the human chooses to look, but never pushed.

### Sound Design

Two sounds only, both short and distinctive:

1. **Decision needed**: A soft double-tone (two rising notes, ~0.3s total). Plays once, never repeats. Distinct from every macOS system sound and common app notification sounds.

2. **Critical failure**: A single low tone (~0.2s). Slightly more urgent than the decision sound but not alarming.

No sound for completions, no sound for running status, no sound for badges.

Both sounds are optional (configurable in preferences). Default: decision sound on, failure sound on.

### Focus Mode

A toggle in the menu bar popover: "Focus Mode" (or keybind `F` in popover).

When active:
- No notifications whatsoever (not even decisions)
- Menu bar icon shows a focus indicator (small slash through it)
- Agents that need decisions queue silently
- Exiting focus mode shows a "catch-up" summary of what happened

This respects the human's deep work time. Agents can wait.

### The "I Was Away" Summary

When the Control Room opens and items have accumulated since last interaction, show a brief summary banner at the top:

```
┌──────────────────────────────────────────────────────┐
│  Since you were last here (2h ago):                  │
│  1 decision waiting · 3 completed · 2 failed         │
│                                          [Dismiss]   │
└──────────────────────────────────────────────────────┘
```

---

## 4. State Transitions

### Contract States (Maps to M6 Lifecycle)

The UI needs to handle the 10 contract states from the M6 plan:

| State | UI Treatment | Attention Tier |
|-------|-------------|----------------|
| Pending | Gray, shows dependencies | Queued |
| Ready | Subtle highlight, "ready to claim" | Queued |
| Claimed | Neutral | Queued |
| Executing | Running indicator (animated) | Running |
| Verifying | Running indicator + "verifying" label | Running |
| Completed | Blue accent, expandable for review | Completed |
| Failed | Red accent, show error summary | Anomaly |
| RollingBack | Amber indicator + "rolling back" | Anomaly |
| RolledBack | Amber, collapsed | Anomaly |
| Cancelled | Gray, collapsed, in history only | Hidden |

### Loading States

- **App launch**: Skeleton UI with gray placeholder blocks for ~200ms while SQLite loads. No spinner -- the data is local and fast.
- **Session discovery**: "Scanning sessions..." text in session list while USF adapters read filesystem. Progressive: show results as each adapter completes.
- **Contract creation**: Optimistic -- item appears immediately in "Running" tier. If creation fails, item transitions to error state inline.

### Empty States

- **No contracts ever**: "No contracts yet. Run `stead run` from the CLI to create your first contract." with a small illustration of the CLI command.
- **No items needing attention**: The popover shows "All clear." with a subtle checkmark. This is the desired state -- make it feel good, not empty.
- **No sessions found**: "No AI CLI sessions found. stead works with Claude Code, Codex CLI, and OpenCode." with links to each.

### Error States

- **SQLite unavailable**: Banner at top of window: "Cannot access stead database. Is another process locking it?" with Retry button.
- **Adapter failure**: Individual adapter errors shown per-CLI: "Could not read Claude Code sessions: [reason]". Other adapters continue working.
- **Stale data**: If file system watcher detects changes but can't process: "Some data may be stale. Pull to refresh." (or automatic retry after 5s).

### Optimistic Updates

When the human takes action:
1. UI updates immediately (button disables, state changes visually)
2. stead-core processes the action
3. On success: no visible change (already updated)
4. On failure: revert UI state, show inline error message

Since this is a monolith (no network), failures should be rare and nearly instant.

### Real-Time Updates (File System Watching)

stead-core watches:
- `.stead/` directory for contract changes (SQLite via polling or `kqueue`/`FSEvents`)
- Claude Code JSONL files for session updates
- Codex CLI and OpenCode session directories

Update strategy:
- Contract state changes: immediate UI update (push from stead-core via callback/delegate)
- Session discovery: re-scan every 30 seconds, or on-demand via pull-to-refresh
- New contract creation (from CLI): detected via SQLite change, appears in UI within 1 second

---

## 5. CLI <-> UI Consistency

### Same Mental Model

The CLI and UI present the same data, same hierarchy, same language:

| CLI Command | UI Equivalent |
|-------------|---------------|
| `stead list` | Full window contract list |
| `stead list --status=failed` | Anomaly tier in full window |
| `stead show <id>` | Expanded contract detail |
| `stead verify <id>` | "Re-verify" action button |
| `stead session list` | Session browser in full window |
| `stead session show <id>` | Session detail with timeline |

### Shared Vocabulary

Both CLI and UI use identical terms:

- "Contract" (not task, ticket, job)
- "Passed" / "Failed" (not success/error, complete/incomplete)
- "Running" / "Queued" (not active/waiting, in-progress/pending)
- "Decision" (not blocker, escalation)
- "Anomaly" (not warning, issue)

The attention tiers in the UI map to CLI filters:
```bash
stead list --needs-decision    # Tier 1
stead list --anomaly           # Tier 2
stead list --completed         # Tier 3
stead list --running           # Tier 4
stead list --queued            # Tier 5
```

### CLI Output Mirrors UI Layout

When running `stead list` in the terminal, the output follows the same attention-priority ordering:

```
NEEDS DECISION (1)
  qwer-q  lx4f-8k2  API design: pick one         12m ago

COMPLETED (2)
  picalyze  mx9a-3j7  Auth refactor      passed   4m ago
  stead     nw2b-5p1  Workspace split     passed   8m ago

RUNNING (3)
  meinungsmache  kj8c-2m4  Rebranding assets       2m

QUEUED (5)
  ...
```

This means a developer who uses the CLI has the exact same situational awareness as one using the GUI. No learning two mental models.

### Cross-Reference

The UI shows a "Copy CLI command" action on every item:
- Contract detail: copies `stead show lx4f-8k2`
- Session: copies `stead session show claude-abc123`

The CLI shows a hint: "Open in Control Room: stead open lx4f-8k2" (which triggers the GUI to open and navigate to that item).

---

## 6. Project Identity

### Project Color Coding

Each project gets a persistent, auto-assigned color from a muted palette (8 colors, cycling). The color appears as:
- Left border accent on every item row
- Project name text color in headers
- Subtle background tint when item is expanded

Colors are derived deterministically from project path (hash) so they're consistent across sessions without configuration.

### Project Filtering

The full window has a project filter bar:

```
[All Projects]  [picalyze]  [qwer-q]  [stead]  [+2 more]
```

Clicking a project shows only that project's items (still ordered by attention priority). The popover always shows all projects (no filtering -- it's for quick scan).

### Session Grouping

In the session browser, sessions are grouped by project, then by CLI:

```
picalyze
  Claude Code (3 sessions)
    "Fix auth token refresh..."        2h ago
    "Add enterprise rate limiting..."  1d ago
    "Refactor middleware stack..."      3d ago
  Codex CLI (1 session)
    "Generate API documentation..."    5h ago

qwer-q
  Claude Code (2 sessions)
    ...
```

---

## 7. Implementation Notes for SwiftUI

### Menu Bar App

Use `MenuBarExtra` with `.window` style for the popover. The menu bar icon updates via `@Published` property on the main `SteadAppState` observable.

### Full Window

Standard `Window` scene. Single `NavigationSplitView` is unnecessary for MVP -- use a flat `List` with `DisclosureGroup` for expandable items.

### Data Flow

```
stead-core (Rust, via FFI)
  -> SteadBridge (Swift, wraps FFI calls)
    -> SteadAppState (ObservableObject)
      -> Views (SwiftUI, reactive)
```

stead-core pushes updates via a callback registered at init. SteadBridge translates Rust types to Swift types and updates SteadAppState. SwiftUI handles the rest reactively.

### Native Feel

- Use SF Symbols for all icons (not custom icons)
- Use system colors for accents (`.red`, `.orange`, `.blue`, `.secondary`)
- Respect system appearance (dark/light mode automatic with semantic colors)
- Use `.contextMenu` for right-click actions on items
- Standard macOS keyboard shortcut conventions (⌘ for app actions, plain keys for in-context)
- Vibrancy and material effects for the popover background

### Accessibility

- All items have proper accessibility labels
- VoiceOver reads attention tier, project name, summary, and available actions
- Dynamic Type support (text sizes scale)
- Reduce Motion: disable running indicators, use static badges instead

---

## 8. What This Design Does NOT Include (Deferred)

- **Dependency graph visualization** -- too complex for MVP, revisit after M6
- **Multi-agent coordination view** -- needs more data about agent relationships
- **Cost tracking / token budgets** -- useful but not supervisory
- **Historical analytics** -- "how many contracts per week" is dashboard thinking
- **Chat with agents** -- direct conversation requires deeper CLI integration
- **Custom notification rules** -- focus mode is enough for MVP
- **Team / multi-user** -- stead is single-user for now

---

## Decision Log (for this document)

1. **Popover over panel**: Chose menu bar popover as primary surface instead of a persistent floating panel. Popover is dismissed when not needed, reducing screen clutter. A floating panel would become "another thing to manage."

2. **Single column over multi-column**: The full window uses a single attention-priority list, not a multi-column layout (like Kanban). Multi-column implies equal attention across columns. Single column enforces the priority hierarchy -- you literally have to scroll past decisions before you see running items.

3. **No real-time streaming of agent output**: Running agents show summary only, not live output. Streaming output in the GUI would turn it into a terminal replacement. The CLI is for watching output; the GUI is for supervision.

4. **Two sounds only**: Resisted adding per-project or per-event sounds. More sounds = faster alarm fatigue. Two sounds cover the two interrupt cases. Everything else is silent.

5. **Project colors are automatic**: No manual project color configuration. Deterministic from path hash means zero setup and consistency across reinstalls. The user never thinks about it.
