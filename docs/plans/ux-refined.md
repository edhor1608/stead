# Stead Control Room: Definitive UX Specification

**Created:** 2026-02-05
**Status:** Definitive spec for SwiftUI implementation
**Sources:** control-room-ux.md, product-vision.md, design-language.md, visionary-concepts.md, NORTH_STAR.md, architecture-review.md, agent-orchestration-2025.md, ffi-comparison.md

---

## Design North Star

Every design decision answers one question: **When the *ding* arrives, how fast does the human regain full context?**

The Control Room is not an app you use. It is a sense you develop. The primary experience is ambient awareness through the menu bar. The full window is a drill-down for deep work. 95% of supervision happens without opening a window.

---

## 1. Information Architecture

### Three Tiers of Engagement

```
Tier 0: Menu Bar Icon       (always visible, zero attention cost)
Tier 1: Popover              (quick scan, 5-second interaction)
Tier 2: Full Window           (deep review, extended interaction)
```

Each tier is a progressive disclosure of the same data. No tier contains information that contradicts another.

### Tier 0: Menu Bar Icon

A single glyph in the macOS menu bar. Communicates aggregate system state.

**Icon:** `circle.circle` (SF Symbols), rendered with `.symbolRenderingMode(.hierarchical)`.

| State | Visual | When |
|-------|--------|------|
| Idle | Monochrome, standard weight | No agents running, no items needing attention |
| Running | Monochrome with subtle inner dot | Agents active, nothing needs you |
| Attention | Badge count overlay | Completed items or anomalies awaiting review |
| Decision | Inner dot tinted `.red` + badge | Agent blocked on human input |

**Rules:**
- The icon never animates. Static states only.
- Badge count = (decisions waiting) + (anomalies) + (completed awaiting review).
- Red tint appears ONLY when a decision is needed. Red is reserved.
- When macOS "Reduce Motion" is on, badge count changes instantly (no fade).

```swift
Image(systemName: "circle.circle")
    .symbolRenderingMode(.hierarchical)
    .foregroundStyle(hasDecisions ? .red : .primary)
```

### Tier 1: Popover

Opens on click or global shortcut. A compact panel (~360px wide, max 500px tall) showing only items that need attention.

```
+--------------------------------------+
|  NEEDS DECISION  (1)                 |
|  +----------------------------------+|
|  | ! qwer-q  API design: pick one   ||
|  |           3 options  .  12m ago   ||
|  +----------------------------------+|
|                                      |
|  COMPLETED  (2)                      |
|  +----------------------------------+|
|  | * picalyze  Auth refactor done    ||
|  |             3 files  .  tests pass||
|  +----------------------------------+|
|  | * stead  Workspace split done     ||
|  |           8 files  .  tests pass  ||
|  +----------------------------------+|
|                                      |
|  RUNNING  (3)        QUEUED  (5)     |
|                                      |
|  ----------------------------------- |
|  Open Control Room         Cmd+Shift+S|
+--------------------------------------+
```

**What's shown:**
- Items needing attention only (Decisions, Anomalies, Completed)
- One line per item: project name, summary, key metadata
- Running/Queued as counts only (they don't need you)
- Footer action to open full window

**What's NOT shown:**
- Session details, timelines, logs
- Running agent output
- Historical data
- Project filtering

**Popover behavior:**
- Uses `MenuBarExtra` with `.window` style
- Vibrancy material background (`.ultraThinMaterial`)
- Dismissed on click outside or Esc
- Clicking an item opens the Full Window focused on that item

### Tier 2: Full Window

A standard macOS window for deep work: reviewing diffs, making decisions, browsing sessions.

**Layout (MVP):**

```
+---------------------------------------------------+
|  Toolbar: [Focus Mode] [Filter] [New Contract] [?]|
+---------------------------------------------------+
|                                                    |
|  [Since you were last here banner, if applicable]  |
|                                                    |
|  NEEDS DECISION (1)                                |
|  [contract row, expandable]                        |
|                                                    |
|  ANOMALY (0) -- hidden when empty                  |
|                                                    |
|  COMPLETED (2)                                     |
|  [contract row]                                    |
|  [contract row]                                    |
|                                                    |
|  RUNNING (3) -- collapsed by default               |
|  QUEUED (5)  -- collapsed by default               |
|                                                    |
+---------------------------------------------------+
```

Single-column layout. No sidebar in MVP. Items are grouped by attention priority and ordered within groups by age (oldest first). Empty groups are hidden.

**Why single column, no sidebar:**
- Multi-column implies equal attention across columns. Single column enforces priority hierarchy -- you scroll past decisions before seeing running items.
- A sidebar adds a navigation concept that competes with the attention-priority model.
- The design-language doc specifies `NavigationSplitView` three-column, but for MVP, a flat `List` with `DisclosureGroup` is simpler and matches the "earn screen presence" principle. Sidebar can be added post-MVP for session browsing and project filtering.

---

## 2. Attention Priority System

Everything in stead is organized by supervisory need, not by project or recency.

### Priority Tiers

| Priority | Tier | Color | SF Symbol | When |
|----------|------|-------|-----------|------|
| 1 | **Needs Decision** | `.red` | `exclamationmark.circle.fill` | Agent blocked on human input |
| 2 | **Anomaly** | `.yellow` | `exclamationmark.triangle.fill` | Errors, timeouts, resource conflicts |
| 3 | **Completed** | `.green` | `checkmark.circle.fill` | Finished, awaiting human review |
| 4 | **Running** | `.secondary` | `circle.dotted.and.circle` | Executing normally |
| 5 | **Queued** | `.tertiary` | `circle.dashed` | Waiting for dependencies/scheduling |

**Within each tier, sort by:**
1. Age within tier (oldest first -- longest-waiting is most urgent)
2. Project grouping (same-project items adjacent within tier)

**State color rules:**
- Color appears on the status indicator only (6-14pt dot/icon). Never flood the row.
- "Needs Decision" is the only tier that may use a very subtle background tint (5-8% opacity red) to pull the eye.
- State is NEVER communicated by color alone. Each state has: text label, distinct SF Symbol shape, and positional grouping.
- Support "Differentiate Without Color" -- symbols change shape per state.

### Contract State Mapping

The contract lifecycle has 10 states (from M6 plan). Each maps to an attention tier:

| Contract State | Attention Tier | UI Treatment |
|---------------|----------------|--------------|
| Pending | Queued | Gray, shows dependencies |
| Ready | Queued | Subtle highlight |
| Claimed | Queued | Neutral |
| Executing | Running | Running indicator (static label) |
| Verifying | Running | "Verifying" label |
| Completed | Completed | Green, expandable for review |
| Failed | Anomaly | Red, show error summary |
| RollingBack | Anomaly | Amber + "rolling back" label |
| RolledBack | Anomaly | Amber, collapsed |
| Cancelled | Hidden | In history only, not shown in active view |

---

## 3. Keyboard Shortcuts

Developer-first, vim-inspired where it fits macOS conventions. Every action is reachable without a mouse.

### Global (works from any app)

| Key | Action |
|-----|--------|
| `Cmd+Shift+S` | Toggle Control Room (opens popover if closed, closes if open) |

Registered via `NSEvent.addGlobalMonitorForEvents`. Configurable in Preferences if it conflicts with another app.

### Popover

| Key | Action |
|-----|--------|
| `j` / `Down` | Navigate to next item |
| `k` / `Up` | Navigate to previous item |
| `Enter` / `Return` | Open selected item in Full Window |
| `Esc` | Close popover |
| `f` | Toggle Focus Mode |
| `Cmd+Shift+S` | Close popover (same as global toggle) |

### Full Window

| Key | Action |
|-----|--------|
| `j` / `Down` | Navigate to next item |
| `k` / `Up` | Navigate to previous item |
| `Enter` | Expand/collapse selected item detail |
| `Esc` | Collapse detail / close window |
| `a` | Approve (on completed or decision item) |
| `r` | Reject (on completed or decision item) |
| `d` | Open decision response field |
| `x` | Cancel contract (with confirmation) |
| `Cmd+N` | New contract |
| `1`-`5` | Jump to attention tier (1=Decision, 2=Anomaly, 3=Completed, 4=Running, 5=Queued) |
| `/` | Open filter/search |
| `?` | Show keyboard shortcut overlay |
| `f` | Toggle Focus Mode |
| `Cmd+C` | Copy CLI command for selected item |

**Keyboard hint display:** Action buttons show their shortcut key as a small badge (e.g., the Approve button shows `A`). These hints appear after 0.5s of no mouse movement in the Full Window, matching macOS menu behavior.

**Why vim-style `j`/`k`:** Target users are developers. `j`/`k` is muscle memory for anyone who uses vim, less, man pages, lazygit, or any developer TUI. Arrow keys also work for non-vim users.

---

## 4. Notification Tiering

Human attention is finite. Notifications cost 23 minutes of recovery time (Gloria Mark, UC Irvine). The notification system respects an **interrupt budget**.

### Tier 1: Interrupt (macOS notification + sound)

Only these justify breaking the human's focus:

1. **Blocked on human decision** -- agent cannot continue without input
2. **Critical failure** -- something broken that affects other work (port conflict, expired auth token affecting multiple agents)

**Notification format:**
```
stead - qwer-q
API design: pick one of 3 options
[Decide]  [Later]
```

- "Decide" opens the Control Room directly to the decision item.
- "Later" dismisses the notification. Menu bar icon retains red tint + badge.
- Delivered via `UNUserNotificationCenter` with `interruptionLevel: .timeSensitive`.

### Tier 2: Badge (menu bar dot, no sound, no banner)

- Contract completed (awaiting review)
- Anomaly detected (unexpected error, timeout)
- Verification failed

The badge count on the menu bar icon shows how many items need attention. No notification banner appears.

### Tier 3: Silent (no notification, no badge change)

- Agent started running
- Agent executing normally
- Contract queued
- Session activity (tool calls, file edits)

Visible in the Full Window if the human chooses to look, but never pushed.

### Sound Design

Two sounds only. Both short, distinctive, and unlike any macOS system sound or common app notification.

| Sound | Trigger | Description | Duration |
|-------|---------|-------------|----------|
| Decision | Agent needs human input | Soft double-tone, two rising notes | ~0.3s |
| Failure | Critical failure | Single low tone | ~0.2s |

No sound for completions. No sound for running status. No ambient audio in MVP.

Both sounds are configurable in Preferences (on/off). Default: both on.

### Focus Mode

A toggle accessible from the popover (key: `f`) and the Full Window toolbar.

**When active:**
- No notifications whatsoever (not even decisions)
- Menu bar icon shows a focus indicator (small slash overlay, like macOS Do Not Disturb)
- Agents that need decisions queue silently
- Exiting Focus Mode shows a "catch-up" summary

**macOS Focus integration (post-MVP, from visionary-concepts.md):**
- macOS Shortcuts action: "Set Stead Attention Level"
- Coding Focus mode -> auto-enable Focus Mode
- Meeting Focus mode -> auto-enable Focus Mode
- Sleep schedule -> auto-enable Focus Mode

### The "I Was Away" Summary

When the Control Room opens and items have accumulated since last interaction:

```
+------------------------------------------------------+
|  Since you were last here (2h ago):                   |
|  1 decision waiting . 3 completed . 2 failed          |
|                                           [Dismiss]   |
+------------------------------------------------------+
```

Shown as a banner at the top of both the popover and the Full Window. Dismissed on tap or after the user interacts with any item.

---

## 5. Interaction Patterns

### The Supervision Loop (Primary, 5-15 seconds)

```
1. Menu bar icon shows badge (or red tint)
2. Click icon (or Cmd+Shift+S)
3. Popover: scan items (2 seconds)
4. Click item needing action
5. Full Window: take action (approve/decide/dismiss)
6. Close (Esc or Cmd+Shift+S)
```

This is the hot path. It must be optimized for speed.

### Deep Review (Secondary)

```
1. Open Full Window (click "Open Control Room" from popover or Cmd+Shift+S when popover is open)
2. Review completed work (expand, view diff, check tests)
3. Approve or reject
4. Optionally check running agents
5. Close
```

### Decision Response

When an agent needs a decision, the expanded item in the Full Window shows:

**Multiple choice decisions:**
- Option cards, selectable by click or arrow keys + Enter
- Comment field below options
- Actions: [Choose Selected] [Need More Info] [Cancel]

**Yes/No decisions:**
- Two prominent buttons. Keyboard: `y` / `n`

**Approval decisions:**
- Inline diff view (native `NSTextView` with syntax highlighting, not a web view)
- Actions: [Approve `a`] [Reject `r`]

**Open-ended decisions:**
- Text input field. Submit on Enter.

### Completed Work Review

Expanded completed items show:

```
+-----------------------------------------------------+
|  COMPLETED                                           |
|  picalyze . Contract mx9a-3j7 . 4m ago              |
|                                                      |
|  Auth flow refactor                                  |
|                                                      |
|  Verification: cargo test            PASSED          |
|  Files changed: 3  (+47 -23)                         |
|  Duration: 2m 34s                                    |
|                                                      |
|  Changes:                                            |
|    M src/auth/middleware.rs     +12 -8                |
|    M src/auth/session.rs       +28 -15               |
|    M tests/auth_test.rs        +7  -0                |
|                                                      |
|  [View Diff]  [View Logs]                            |
|                                                      |
|           [Approve a]  [Reject r]  [Dismiss]         |
+-----------------------------------------------------+
```

"View Diff" opens an inline native diff viewer. "View Logs" shows the agent's session timeline.

### New Contract

`Cmd+N` opens a creation sheet:

```
+-----------------------------------------------------+
|  NEW CONTRACT                                        |
|                                                      |
|  Project:  [ picalyze          v ]                   |
|  CLI:      [ Claude Code       v ]                   |
|  Task:     [ Fix the auth token refresh bug     ]    |
|  Verify:   [ cargo test --lib auth              ]    |
|                                                      |
|                           [Cancel]  [Run Contract]   |
+-----------------------------------------------------+
```

---

## 6. Empty States and Onboarding

Empty states are the first impression. They must feel intentional, not broken.

### First Launch (No contracts ever)

```
+-----------------------------------------------------+
|                                                      |
|              (circle.circle icon, large)              |
|                                                      |
|            No projects need attention.                |
|                                                      |
|     Run `stead run` from the terminal to create      |
|     your first contract.                             |
|                                                      |
|     stead works with:                                |
|       Claude Code  .  Codex CLI  .  OpenCode         |
|                                                      |
+-----------------------------------------------------+
```

Tone: calm, welcoming, not empty-feeling. The "No projects need attention" phrasing implies this is the desired state. The CLI command hint provides a clear next step.

### No Items Needing Attention (Popover)

```
+--------------------------------------+
|                                      |
|         All clear.  (checkmark)      |
|                                      |
|  RUNNING  (3)        QUEUED  (2)     |
|                                      |
|  Open Control Room         Cmd+Shift+S|
+--------------------------------------+
```

"All clear" is the desired state. It should feel good, not empty. Running/Queued counts remain visible so you know work is happening.

### No Sessions Found

```
+-----------------------------------------------------+
|                                                      |
|  No AI CLI sessions found.                           |
|                                                      |
|  stead detects sessions from:                        |
|    Claude Code  (~/.claude/projects/)                 |
|    Codex CLI    (~/.codex/sessions/)                  |
|    OpenCode     (~/.local/share/opencode/)            |
|                                                      |
|  Start a session in any of these CLIs and it will    |
|  appear here automatically.                          |
|                                                      |
+-----------------------------------------------------+
```

### Error States

| Error | Display | Action |
|-------|---------|--------|
| SQLite unavailable | Banner: "Cannot access stead database. Is another process locking it?" | [Retry] button |
| Adapter failure | Per-CLI inline: "Could not read Claude Code sessions: [reason]" | Other adapters continue |
| Stale data | "Some data may be stale." | Auto-retry after 5s |

Errors are non-modal. They appear as banners at the top of the content area, not as alert dialogs. The app remains usable even when one adapter fails.

---

## 7. CLI <-> UI Synchronization

### Same Mental Model

The CLI and UI present identical data, identical hierarchy, identical vocabulary.

| CLI Command | UI Equivalent |
|-------------|---------------|
| `stead list` | Full Window contract list |
| `stead list --needs-decision` | Needs Decision tier |
| `stead list --anomaly` | Anomaly tier |
| `stead list --completed` | Completed tier |
| `stead list --running` | Running tier |
| `stead list --queued` | Queued tier |
| `stead show <id>` | Expanded contract detail |
| `stead verify <id>` | "Re-verify" action button |
| `stead session list` | Session browser |
| `stead session show <id>` | Session detail with timeline |

### Shared Vocabulary

Both CLI and UI use identical terms. This is non-negotiable.

| Term | NOT |
|------|-----|
| Contract | task, ticket, job |
| Passed / Failed | success/error, complete/incomplete |
| Running / Queued | active/waiting, in-progress/pending |
| Decision | blocker, escalation |
| Anomaly | warning, issue |

### CLI Output Mirrors UI Layout

`stead list` in the terminal follows the same attention-priority ordering:

```
NEEDS DECISION (1)
  qwer-q  lx4f-8k2  API design: pick one         12m ago

COMPLETED (2)
  picalyze  mx9a-3j7  Auth refactor      passed   4m ago
  stead     nw2b-5p1  Workspace split     passed   8m ago

RUNNING (3)
  meinungsmache  kj8c-2m4  Rebranding assets       2m
```

A developer who uses the CLI has the same situational awareness as one using the GUI.

### Cross-Reference

**UI -> CLI:** Every item in the UI has a "Copy CLI Command" action (context menu or `Cmd+C`). Contract detail copies `stead show lx4f-8k2`. Session copies `stead session show claude-abc123`.

**CLI -> UI:** The CLI outputs a hint: `Open in Control Room: stead open lx4f-8k2`. Running `stead open <id>` opens the GUI and navigates directly to that item.

### Data Flow (Shared stead-core)

```
stead-core (Rust library)
  SQLite database: ~/.stead/stead.db
    |
    +--> CLI reads/writes directly (link stead-core)
    |
    +--> SwiftUI reads/writes via FFI (UniFFI -> Swift)
         SteadBridge (Swift) wraps FFI calls
           -> SteadAppState (@Observable)
             -> SwiftUI Views (reactive)
```

Both CLI and UI use the same stead-core library, same SQLite database, same data. No synchronization protocol needed -- they share storage. stead-core watches the SQLite database for changes (via `kqueue`/`FSEvents` or polling). When the CLI writes a contract, the UI sees it within 1 second.

Session discovery (USF adapters) re-scans every 30 seconds or on pull-to-refresh.

---

## 8. Project Identity

### Project Color Coding

Each project gets a persistent color from a muted 8-color palette. Colors are deterministic from the project path hash -- consistent across sessions, no configuration needed.

**Color appears as:**
- Left border accent (3pt) on every contract/session row
- Project name text color in headers
- Subtle background tint when item is expanded (3-5% opacity)

```swift
extension Color {
    static let projectPalette: [Color] = [
        .blue, .purple, .pink, .orange,
        .teal, .indigo, .mint, .cyan
    ]

    static func forProject(_ path: String) -> Color {
        let hash = path.hashValue
        let index = abs(hash) % projectPalette.count
        return projectPalette[index]
    }
}
```

### Project Filtering (Full Window)

A filter bar below the toolbar:

```
[All Projects]  [picalyze]  [qwer-q]  [stead]  [+2 more]
```

Clicking a project shows only that project's items (still ordered by attention priority). The popover always shows all projects -- no filtering there, it's for quick scan.

---

## 9. Accessibility

Accessibility is not an afterthought. It is a design constraint applied from the start.

### VoiceOver

All items have proper accessibility labels that read the complete context:

```swift
.accessibilityLabel("Needs Decision: qwer-q, API design pick one of 3 options, 12 minutes ago")
.accessibilityHint("Press Enter to open details")
```

VoiceOver reads: attention tier, project name, summary, time, available actions.

### Dynamic Type

All text uses system font styles (`.body`, `.caption`, `.callout`). Sizes scale with the user's Dynamic Type preference. Row heights are flexible to accommodate larger text.

### Reduce Motion

When "Reduce Motion" is enabled:
- No animation on state transitions (instant change)
- No scale pulse on status dots
- Running indicator is a static "Running" label (no animation even for sighted users, per design-language.md)
- Section expand/collapse is instant

### Increase Contrast

System semantic colors automatically adapt. The color system uses `.red`, `.yellow`, `.green`, `.secondary`, `.tertiary` -- all respond to Increase Contrast.

### Differentiate Without Color

Every state has three distinguishing features:
1. **Text label** ("Needs Decision", "Completed", etc.)
2. **SF Symbol shape** (circle with exclamation vs checkmark vs dashed circle)
3. **Position** (grouped by tier -- you know the state by where it appears)

Color is reinforcement, never the sole signal.

### Keyboard Navigation

Full keyboard navigation is mandatory. Every interactive element is reachable via Tab and actionable via Enter/Space. The vim-style `j`/`k` shortcuts are in addition to, not instead of, standard macOS keyboard navigation.

### Screen Reader Announcements

When contract state changes while the app is open, VoiceOver announces:
- "qwer-q completed. Tests passed. Review available."
- "picalyze needs decision. API design: pick one."

Implemented via `AccessibilityNotification.Announcement`.

---

## 10. Visionary Concepts: What's MVP vs. Post-MVP

The visionary-concepts.md document proposes 12 concepts across 4 layers. Here is the reconciliation with MVP reality:

### In MVP (Layer 1: Subtlety)

| Concept | How It Maps to MVP |
|---------|--------------------|
| **Disappearing Interface** | The menu bar icon IS the primary interface. Full Window is the drill-down. Already in the plan. |
| **Attention Thermostat** | MVP version = Focus Mode (binary on/off). Post-MVP: gradient 0-100 tied to macOS Focus modes. |

### Post-MVP (Layer 2: Awareness)

| Concept | When | Dependency |
|---------|------|------------|
| **Peripheral Vision Strip** | After MVP launch, based on user feedback | `NSWindow` with `.canBecomeKey = false` |
| **Morning Briefing** | After MVP launch | Context generator in stead-core |
| **Agent Negotiation Protocol** | With contract engine maturity (M6+) | Resource registry in SQLite |
| **Spatial Audio** | Experimental, user opt-in | AVAudioEngine |
| **Cognitive Load Estimation** | After morning briefing | Focus event tracking in SQLite |

### Future (Layer 3-4: Prediction, Autonomy)

Generative Interface, Agent Economy, Temporal Spaces, Invisible Hand, Agent Presence. These require trust built through Layers 1-2. Not designed now.

### The MVP Boundary

The MVP Control Room is:
1. Menu bar icon with state-encoded color
2. Popover showing attention-priority items
3. Full Window for deep review, decisions, and session browsing
4. Focus Mode (binary toggle)
5. Two notification sounds (decision, failure)
6. Keyboard-first navigation (vim-style `j`/`k`)
7. Native macOS feel (SF Symbols, system colors, system font)

What it is NOT:
- No peripheral vision strip
- No spatial audio
- No morning briefing
- No attention thermostat (beyond binary Focus Mode)
- No generative interface
- No agent presence indicators
- No cost tracking
- No dependency graph visualization
- No chat with agents
- No multi-user/team support

---

## 11. SwiftUI Implementation Notes

### App Structure

```swift
@main
struct SteadApp: App {
    @State private var appState = SteadAppState()

    var body: some Scene {
        // Tier 0 + Tier 1: Menu bar icon + popover
        MenuBarExtra {
            PopoverView()
                .environment(appState)
        } label: {
            MenuBarIcon(state: appState.aggregateState)
        }
        .menuBarExtraStyle(.window)

        // Tier 2: Full window
        Window("Control Room", id: "control-room") {
            ControlRoomView()
                .environment(appState)
        }
        .keyboardShortcut("s", modifiers: [.command, .shift])
    }
}
```

### Data Flow

```
stead-core (Rust, compiled as static library)
  |
  +--> UniFFI generates Swift bindings
         |
         +--> SteadBridge.swift (wraps FFI, converts types)
                |
                +--> SteadAppState (@Observable)
                       |
                       +--> SwiftUI Views (reactive)
```

`SteadAppState` is the single source of truth for the UI. It holds:
- `contracts: [Contract]` (sorted by attention priority)
- `sessions: [SessionSummary]` (sorted by recency)
- `aggregateState: AggregateState` (drives menu bar icon)
- `focusModeEnabled: Bool`
- `lastInteractionTime: Date?` (drives "I was away" summary)

### Native Feel Checklist

- [ ] SF Symbols for all icons (no custom icons except app icon)
- [ ] System semantic colors only (`.red`, `.green`, `.secondary`, etc.)
- [ ] System font (SF Pro for text, SF Mono for code/IDs)
- [ ] Respect light/dark mode automatically
- [ ] Respect Increase Contrast, Reduce Motion, Differentiate Without Color
- [ ] `.contextMenu` for right-click on items
- [ ] Standard macOS keyboard shortcuts (Cmd for app actions, plain keys for in-context)
- [ ] Vibrancy material for popover background
- [ ] Standard window management (full screen, split view, Stage Manager)
- [ ] System accent color (don't override)
- [ ] Standard button styles (`.borderedProminent` for primary action, `.bordered` for secondary)

### Component Sizing

| Component | Size |
|-----------|------|
| Contract row height | 36pt (compact) |
| Status icon | 14pt in list, 18pt in detail |
| Inline status dot | 6pt diameter |
| Popover width | 360pt |
| Popover max height | 500pt |
| Spacing base unit | 4pt |
| Content margins | 16pt (lg) |
| Section spacing | 24pt (xl) |

### Motion

From design-language.md -- motion proves state changes, it is not decoration:

- Contract state change: status dot cross-fade + scale pulse (1.0 -> 1.15 -> 1.0), 300ms, `.easeOut`
- Contract moves between groups: row slides to new position, 350ms, `.spring(response: 0.35, dampingFraction: 0.85)`
- New contract appears: fade in + slide down, 250ms, `.easeOut`
- Contract dismissed: fade out + slide left, 200ms, `.easeIn`
- **Never animate running state.** No spinners, no pulsing dots.
- **No loading skeletons.** If data isn't ready, show nothing.
- System animations for system behaviors (window resize, sidebar, navigation).

---

## 12. Design Decisions Log

1. **Popover over panel.** Menu bar popover as primary surface instead of floating panel. Popover is dismissed when not needed. A floating panel would become "another thing to manage."

2. **Single column over multi-column.** Single attention-priority list, not Kanban. Multi-column implies equal attention across columns. Single column enforces priority.

3. **No real-time streaming of agent output.** Running agents show summary only, not live output. The CLI is for watching output; the GUI is for supervision.

4. **Two sounds only.** More sounds = faster alarm fatigue. Two sounds cover the two interrupt cases.

5. **Project colors are automatic.** Deterministic from path hash. Zero configuration. Consistent across reinstalls.

6. **No sidebar in MVP.** Flat list with disclosure groups. Sidebar adds complexity that can wait until there's enough content to justify navigation hierarchy.

7. **UniFFI over swift-bridge.** Mozilla-backed, better tooling (cargo-swift), richer type support. See ffi-comparison.md.

8. **Focus Mode is binary, not a thermostat (MVP).** The Attention Thermostat from visionary-concepts.md is powerful but complex. Binary Focus Mode ships faster and covers the core need. Thermostat is post-MVP.

9. **Vim-style shortcuts are additive.** `j`/`k` supplements standard macOS keyboard navigation (arrow keys, Tab, Enter/Space). Never replaces it.

10. **No chat with agents (MVP).** Direct conversation requires deeper CLI integration and a different UX paradigm. The Control Room is for supervision, not collaboration. Revisit post-MVP.
