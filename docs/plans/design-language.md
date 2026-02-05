# Stead Control Room: Design Language

**Created:** 2026-02-05
**Status:** Active

This document defines the visual language for the Stead Control Room -- a native macOS application for supervising autonomous AI coding agents. Every decision here traces back to the North Star: reduce cognitive overhead when the *ding* arrives.

---

## Design Principles

### 1. Calm Until It Matters

The default state is silence. A blank canvas. Screen presence must be earned. When everything is running smoothly, the interface recedes. When something demands human attention, it surfaces clearly and immediately. No ambient noise, no decorative elements, no gratuitous motion.

### 2. Attention Is the Scarce Resource

The human's attention is the bottleneck, not information density. Every pixel exists to answer one question: *What needs me right now?* Priority determines prominence. Contracts needing a decision command the eye. Completed work waits patiently. Running work is nearly invisible until it isn't.

### 3. Show State, Not Activity

Don't show that agents are "busy." Show what state contracts are in, what's blocked, what succeeded, what failed. ATC controllers don't watch engines running -- they watch trajectories and conflicts. Animate state transitions, not ongoing processes.

### 4. Native and Inevitable

This should feel like it shipped with macOS. Not "styled to look native" -- actually native. System font, system colors where appropriate, system behaviors (drag, resize, keyboard shortcuts, menu bar conventions). The app should feel like it was always there.

---

## Color System

Color carries meaning. In the Control Room, color is the primary signal of contract state -- the attention priority hierarchy from the North Star.

### Attention State Colors

| Priority | State | Light Mode | Dark Mode | SwiftUI |
|----------|-------|------------|-----------|---------|
| 1 | **Needs Decision** | `#CC2936` | `#FF453A` | `.red` (system) |
| 2 | **Anomaly** | `#CC7A00` | `#FFD60A` | `.yellow` (system) |
| 3 | **Completed** | `#1D7A3E` | `#30D158` | `.green` (system) |
| 4 | **Running** | `#636366` | `#EBEBF5` (70% opacity) | `.secondary` |
| 5 | **Queued** | `#AEAEB2` | `#636366` | `.tertiary` |

Use system semantic colors. They adapt automatically to light/dark mode, accessibility settings, and increased contrast. The hex values above are reference -- always prefer the SwiftUI semantic variant.

**Usage rules:**
- State color appears on the **status indicator only** (a small dot or badge). Never flood the entire row or card with color.
- "Needs Decision" is the only state that may use a background tint (very subtle, 5-8% opacity) to pull the eye.
- Never use red for anything other than "Needs Decision." Red is reserved.
- Yellow for anomalies only -- not warnings, not caution, not "something might be wrong."

### Surface Colors

| Element | Light Mode | Dark Mode | SwiftUI |
|---------|------------|-----------|---------|
| Window background | System | System | `.background` |
| Content area | System | System | `Color(.windowBackgroundColor)` |
| Card / grouped surface | System | System | `Color(.controlBackgroundColor)` |
| Sidebar | System | System | Automatic with `NavigationSplitView` |
| Dividers | System | System | `.separator` |

Use `NSColor` semantic names via SwiftUI bridges. Don't hardcode backgrounds.

### Text Colors

| Role | SwiftUI |
|------|---------|
| Primary text | `.primary` |
| Secondary text | `.secondary` |
| Tertiary / timestamps | `.tertiary` |
| Placeholder | `.quaternary` |
| Link / action | `.accentColor` |

### Accent Color

The app accent color is **system blue** (`.accentColor`). It respects the user's System Settings accent color preference. Don't override it.

### Accessibility

- All state colors meet **WCAG AA** (4.5:1 contrast ratio) against their respective backgrounds in both light and dark mode.
- State is never communicated by color alone. Each state also has: a text label, a distinct SF Symbol, and a positional grouping (see Component Patterns).
- Support the **Increase Contrast** accessibility setting -- system colors handle this automatically when using semantic variants.
- Support **Differentiate Without Color** -- SF Symbols change shape per state, not just color.

---

## Typography

SF Pro for interface. SF Mono for code and identifiers. No other fonts.

### Type Scale

| Role | Font | Size | Weight | Usage |
|------|------|------|--------|-------|
| Window title | SF Pro | 13 | **Semibold** | Navigation bar, sidebar headers |
| Section header | SF Pro | 13 | **Medium** | Group labels ("Needs Decision", "Running") |
| Body | SF Pro | 13 | Regular | Contract descriptions, session info |
| Caption | SF Pro | 11 | Regular | Timestamps, metadata, secondary info |
| Contract ID | SF Mono | 11 | Medium | `qwer-q`, `abc-1` short IDs |
| Code / paths | SF Mono | 12 | Regular | File paths, commands, terminal output |
| Badge count | SF Pro | 11 | **Bold** | Count in section headers |
| Menu bar | SF Pro | -- | -- | System-controlled |

**Notes:**
- 13pt is the macOS standard body size. Don't go larger for regular content.
- Size communicates hierarchy: headers and body are the same size, differentiated by **weight** alone. This keeps the interface dense without feeling cramped.
- Monospace signals "machine-generated" or "machine-identifiable" content. If a human wrote it, SF Pro. If a machine produced it or it's an identifier, SF Mono.

### SwiftUI Implementation

```swift
// Standard body text
.font(.body)

// Section header
.font(.body.weight(.medium))

// Contract ID
.font(.system(.caption, design: .monospaced).weight(.medium))

// Code / terminal output
.font(.system(.callout, design: .monospaced))

// Timestamp
.font(.caption)
.foregroundStyle(.secondary)
```

---

## Spatial System

4pt base unit. All spacing is a multiple of 4.

### Spacing Scale

| Token | Value | Usage |
|-------|-------|-------|
| `xxs` | 2pt | Icon-to-label gap inside a badge |
| `xs` | 4pt | Tight padding inside compact elements |
| `sm` | 8pt | Padding inside cards, list row insets |
| `md` | 12pt | Gap between grouped elements |
| `lg` | 16pt | Section spacing, content margins |
| `xl` | 24pt | Major section breaks |
| `xxl` | 32pt | Page-level padding (sparingly) |

### Content Density

The Control Room has two density modes driven by the view context, not a user toggle:

**Overview (List/Sidebar):** Dense. Rows are compact -- 32-36pt height. Minimal padding. Maximum contracts visible. This is the ATC radar view: scan many items quickly.

**Detail (Inspector):** Spacious. Standard macOS inspector panel spacing. Full contract details, session logs, verification output. 8-12pt padding within sections.

### Layout Structure

```
┌─────────────────────────────────────────────────┐
│  Toolbar (system standard)                      │
├──────────┬──────────────────────┬───────────────┤
│          │                      │               │
│ Sidebar  │   Content Area       │  Inspector    │
│ 220pt    │   (flexible)         │  280pt        │
│          │                      │  (optional)   │
│          │                      │               │
├──────────┴──────────────────────┴───────────────┤
│  Status bar (optional, menu bar info)           │
└─────────────────────────────────────────────────┘
```

- Standard `NavigationSplitView` three-column layout.
- Sidebar width: 220pt (collapsible).
- Inspector: 280pt (togglable, hidden by default -- content area takes full width).
- Respect macOS window management: full screen, split view, Stage Manager.

---

## Motion Language

### Philosophy

Motion proves that a state change happened. It is not decoration. If removing an animation would make the user miss a state transition, keep it. If removing it changes nothing about comprehension, remove it.

### Transitions

| Event | Animation | Duration | Curve |
|-------|-----------|----------|-------|
| Contract state change | Status dot cross-fade + subtle scale pulse (1.0 -> 1.15 -> 1.0) | 300ms | `.easeOut` |
| Contract moves between groups | Row slides to new position | 350ms | `.spring(response: 0.35, dampingFraction: 0.85)` |
| New contract appears | Fade in + slide down from top of group | 250ms | `.easeOut` |
| Contract dismissed/archived | Fade out + slide left | 200ms | `.easeIn` |
| Inspector open/close | Standard sidebar animation | System | System |
| Section collapse/expand | Height animation | 250ms | `.easeInOut` |
| View transition (navigate) | System push/pop | System | System |

### Rules

- **Never animate running state.** No spinners, no pulsing dots, no progress bars for indeterminate work. A static label "Running" is sufficient. ATC radar dots don't animate when planes are flying normally.
- **Animate arrivals, not existence.** When a contract transitions to "Completed," animate the transition. Once it's there, it's static.
- **System animations for system behaviors.** Window resize, toolbar, sidebar, navigation -- use the system defaults. Don't custom-animate what macOS already handles.
- **No loading skeletons.** If data isn't ready, show nothing. The window can be empty. Skeleton UIs are a web pattern that signals "this is a web app."

### SwiftUI Implementation

```swift
// State change in a list
withAnimation(.spring(response: 0.35, dampingFraction: 0.85)) {
    // move contract to new group
}

// Status dot pulse
.scaleEffect(justChanged ? 1.15 : 1.0)
.animation(.easeOut(duration: 0.3), value: contract.status)
```

---

## Iconography

### SF Symbols

All icons use SF Symbols. No custom icons except the app icon and menu bar icon.

### Contract State Symbols

| State | Symbol | Rationale |
|-------|--------|-----------|
| Needs Decision | `exclamationmark.circle.fill` | Filled = demands attention |
| Anomaly | `exclamationmark.triangle.fill` | Warning triangle, universally understood |
| Completed | `checkmark.circle.fill` | Success, filled = final |
| Running | `circle.dotted.and.circle` | In-flight, not yet resolved |
| Queued | `circle.dashed` | Waiting, incomplete |
| Failed | `xmark.circle.fill` | Terminal failure |
| Rolling Back | `arrow.uturn.backward.circle` | Reversal in progress |
| Rolled Back | `arrow.uturn.backward.circle.fill` | Reversal complete |
| Cancelled | `minus.circle` | Neutral removal |

**Rendering:**
- All state icons use `.symbolRenderingMode(.hierarchical)` -- the primary state color fills the symbol, with automatic hierarchical depth.
- Size: 14pt in list rows, 18pt in detail view headers.
- Symbols change **shape** between states, not just color. This supports "Differentiate Without Color" accessibility.

### Sidebar / Navigation Symbols

| Item | Symbol |
|------|--------|
| All Contracts | `doc.text` |
| Sessions | `terminal` |
| Projects | `folder` |
| Settings | `gear` |

### Menu Bar Icon

The menu bar icon is a minimal, monochrome glyph. A small circle with a subtle inner dot -- suggesting a radar blip or a status indicator. It uses SF Symbols template rendering to match the system menu bar style.

- **No activity:** Monochrome (standard menu bar icon weight).
- **Needs Decision:** The dot gains the system red tint. This is the only time the menu bar icon uses color.
- **Badge count:** Standard macOS badge overlay showing the count of items needing attention.

```swift
// Menu bar
Image(systemName: "circle.circle")
    .symbolRenderingMode(.hierarchical)
    .foregroundStyle(hasDecisions ? .red : .primary)
```

---

## Component Patterns

### Contract Row (List View)

The primary repeating element. Compact, scannable, information-dense.

```
┌──────────────────────────────────────────────────────┐
│ [state icon]  Contract description text    [time ago] │
│               project-name  ·  qwer-q        12m ago │
└──────────────────────────────────────────────────────┘
```

- **Height:** 36pt (compact density).
- **State icon:** Left-aligned, 14pt, state-colored.
- **Description:** Primary text, single line, truncated with ellipsis.
- **Project name:** Secondary text, below description.
- **Contract ID:** Monospaced, secondary text, after project name separated by `·`.
- **Time:** Right-aligned, caption, tertiary color. Relative ("12m ago", "2h ago"). Absolute on hover.
- **Selection:** Standard macOS list selection (system highlight color).
- **Hover:** Subtle background change (system standard).

### Section Group (Attention Priority)

Contracts are grouped by attention state. Groups appear in priority order. Empty groups are hidden.

```
Needs Decision (2)
├── [contract row]
└── [contract row]

Completed (5)
├── [contract row]
├── [contract row]
...
```

- **Section header:** Body weight medium, with count badge.
- **Collapsible:** Click header to collapse. Collapsed state persists.
- **Empty groups hidden.** If nothing needs a decision, "Needs Decision" group doesn't exist. The interface is calm.

### Session Row (Sessions View)

```
┌──────────────────────────────────────────────────────┐
│ [cli icon]  Session title / summary        [time ago] │
│             claude-code  ·  ses_abc123        5m ago  │
└──────────────────────────────────────────────────────┘
```

- Same layout as contract rows for visual consistency.
- CLI icon: Small identifier for Claude Code / Codex / OpenCode.

### Status Badge (Inline)

A small colored dot (6pt diameter) used inline when the full icon is too heavy.

```swift
Circle()
    .fill(stateColor)
    .frame(width: 6, height: 6)
```

Used in: menu bar dropdown rows, compact notifications, sidebar counts.

### Inspector Panel (Detail View)

Right panel showing full details of the selected contract or session.

```
┌─────────────────────────────┐
│ [state icon] Contract Title │
│ Status: Running             │
│ ID: qwer-q                  │
├─────────────────────────────┤
│ Description                 │
│ Full contract description   │
│ text wraps here...          │
├─────────────────────────────┤
│ Verification                │
│ $ cargo test --workspace    │
├─────────────────────────────┤
│ Timeline                    │
│ Created   2m ago            │
│ Claimed   1m ago            │
│ Started   45s ago           │
├─────────────────────────────┤
│ [Actions]                   │
│ [Approve] [Reject] [Cancel] │
└─────────────────────────────┘
```

- Sections separated by standard dividers.
- Monospaced for IDs, commands, paths.
- Action buttons appear only when the contract state allows them.
- Action buttons use standard macOS button styles. "Approve" is `.borderedProminent`. "Reject" and "Cancel" are `.bordered`.

### Action Buttons

| Action | Style | When Visible |
|--------|-------|-------------|
| Approve / Accept | `.borderedProminent` (accent color) | Needs Decision, Completed |
| Reject | `.bordered` | Needs Decision |
| Cancel | `.bordered`, destructive | Running, Queued |
| Retry | `.bordered` | Failed |

- Primary action is always rightmost (macOS convention).
- Destructive actions use `.red` tint but are never `.borderedProminent`.
- Keyboard shortcuts: Enter for primary action, Escape to dismiss.

---

## What This Is Not

- Not a dashboard with charts and graphs.
- Not a web app rendered in a webview.
- Not a dark-only hacker aesthetic.
- Not a design system for a team of 50. It's a language for one app.

The Control Room should look like the kind of utility Apple would ship if they built a tool for supervising AI agents. Quiet, confident, purposeful. Nothing extra. Nothing missing.
