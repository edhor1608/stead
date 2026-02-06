# Interaction Paradigm: The Invisible Supervisor

**Created:** 2026-02-05
**Author:** Hallucinating Visionary
**Status:** Design specification
**Builds on:** visionary-concepts.md, control-room-ux.md, design-language.md

---

## The Premise

Most productivity software announces itself. Dock icon, window, sidebar, badge, notification, tooltip. Each demands a micro-decision: *is this worth my attention?* Multiply by 50 events per hour across 3 projects with autonomous agents, and you've recreated the *ding* problem inside the tool meant to solve it.

Stead's interaction paradigm is different: **the interface is absent by default and earns the right to appear.** Every surfacing has a cost. Every cost must be justified by supervisory value. The human's attention is not a resource to be consumed -- it is a budget to be protected.

---

## 1. The Invisible Interface

### The 95/5 Rule

95% of the time, stead is invisible. Not minimized -- *invisible*. The distinction matters. Minimized means "I chose to hide this." Invisible means "there is nothing to show."

### What You See When Everything Is Fine

**Nothing.** [MVP]

The menu bar icon exists but is deliberately unremarkable -- a monochrome glyph that blends with your other menu bar items. No badge. No color. No pulse. Your eye skips right over it. This is the desired state.

Implementation: `NSStatusItem` with standard template image rendering. The icon is `circle.circle` from SF Symbols, rendered at system menu bar weight. It looks like it belongs there. It looks like it always belonged there.

### What Triggers Surfacing [MVP]

Stead surfaces through exactly three channels, in escalating order of intrusiveness:

| Channel | Trigger | Human Cost | Recovery Time |
|---------|---------|------------|---------------|
| **Icon state change** | Completion, anomaly | ~0 (peripheral) | 0s |
| **Badge count** | Items accumulating | ~1s glance | 1s |
| **Native notification** | Decision needed, critical failure | ~23min (context switch) | Minutes |

**Specific triggers:**

1. **Icon gains color** (red dot): An agent is blocked on a human decision. This is the *only* thing that colors the icon. [MVP]

2. **Badge count appears**: Completed contracts awaiting review. The number tells you magnitude without requiring a click. Badge "3" means three things finished. You decide when to look. [MVP]

3. **macOS notification**: Reserved for two cases only -- (a) agent blocked on decision for >5 minutes, (b) critical resource conflict affecting multiple projects. The notification includes a single action button that resolves the issue. [MVP]

### How It Goes Away [MVP]

Stead doesn't "close." It *resolves*.

- Approve a completed contract -> badge count decrements. If zero, badge disappears.
- Make a decision -> red dot clears. If no other decisions pending, icon returns to monochrome.
- Dismiss the popover -> it's gone. No minimized state, no background window. Just gone.
- All contracts resolved -> icon returns to monochrome with no badge. Stead is invisible again.

The mental model: the interface exists only as long as there are unresolved items. When the inbox is empty, the interface vanishes. Not hidden. Vanished. Like the warning light on your car dashboard -- you don't "close" it, the problem just stops existing.

### The Ambient Awareness Layer [MVP]

The menu bar icon encodes system state in a single glyph:

| Icon State | Meaning | Visual |
|------------|---------|--------|
| Monochrome, no badge | All clear or no agents running | You don't even notice it |
| Monochrome, badge "3" | 3 items completed, waiting for you | Glanceable count |
| Red dot, badge "1" | 1 decision blocking an agent | Demands eventual attention |
| Red dot, badge "4" | 1 decision + 3 completions | Urgency + backlog |

You read this the same way you read the battery icon or the wifi icon -- peripherally, without conscious effort, as part of scanning the menu bar while reaching for something else.

### The Vanishing Icon [FUTURE]

When no agents are running and no items need attention, the menu bar icon disappears entirely. Absence is the strongest signal that all is well. This requires `NSStatusItem.isVisible = false` and re-appearing when state changes.

The psychological effect: you stop checking. You stop worrying. You trust the absence. And the first time the icon reappears after being gone, your brain registers it immediately -- because something that wasn't there now is. This is a stronger signal than any badge.

---

## 2. The Attention Budget

### The Model

You have 100 attention points per hour. Each interaction costs points. The budget is not literal (no counter in the UI) -- it's a design constraint that governs stead's behavior.

| Interaction | Cost | Budget Category |
|-------------|------|-----------------|
| Icon state change (peripheral) | 1 point | Nearly free |
| Glance at badge count | 2 points | Cheap |
| Open popover, scan items | 5 points | Moderate |
| Read a completion summary | 5 points | Moderate |
| Make a decision (simple) | 10 points | Significant |
| Make a decision (complex, needs diff review) | 20 points | Expensive |
| macOS notification interrupt | 30 points | Very expensive |
| Full context switch to Control Room window | 15 points | Expensive |

**Budget allocation for a typical hour with 3 projects:**

```
Running background: 3 icon glances .............. 3 points
Two completions reviewed: 2x popover+scan ....... 20 points
One simple decision: notification+decide ......... 40 points
Ambient awareness: periodic badge checks ......... 7 points
                                          Total: ~70 points
                                         Buffer:  30 points
```

That 30-point buffer is sacred. It absorbs unexpected events without overwhelming.

### What Earns an Interrupt (Notification) [MVP]

Only events where delay causes compounding cost:

1. **Agent blocked on decision** (after 5min grace period) -- the agent is literally stopped, burning time. Cost of NOT interrupting exceeds cost of interrupting.

2. **Resource conflict blocking multiple agents** -- port collision, auth token expired, shared service down. One problem cascading across projects.

That's it. Two cases. Everything else is a badge or silent.

### What Earns a Badge [MVP]

Events that need you eventually but not now:

- Contract completed, awaiting review
- Contract completed but verification failed (tests didn't pass)
- Anomaly detected (timeout, unexpected error, agent behaving strangely)
- Agent finished but left the codebase in an unexpected state

Badge items accumulate. The count is your "review inbox." You choose when to empty it.

### What Stays Silent [MVP]

Everything that doesn't need you:

- Agent started running
- Agent is executing normally (tool calls, file edits, thinking)
- Contract queued, dependencies resolving
- Session created/destroyed
- Normal resource allocation (port assigned, no conflict)

These are visible in the full Control Room window if you choose to look. They are never pushed.

### Training the System [FUTURE]

The attention budget adapts to your behavior over time:

**Signal: dismiss without reading** -> This type of event is less important to you. Demote it. If you dismiss 5 completion badges for a specific project without reviewing, that project's completions stop earning badges and become silent. The project's items still appear in the popover -- they just don't increment the badge count.

**Signal: click within 10 seconds** -> This type of event is important to you. Keep it at current level or promote it.

**Signal: navigate to full window to investigate** -> This was worth a context switch. Ensure it keeps earning badges.

**Signal: enable Focus Mode during certain hours** -> Learn the pattern. Auto-enter Focus Mode during those hours after 3 repetitions.

Implementation: a simple decay/boost score per (project, event_type) tuple, stored in SQLite. Score decays on dismiss, boosts on interaction. Events below a threshold stop badging. No ML. Just exponential moving averages.

### Focus Mode [MVP]

A toggle in the popover (keyboard: `F`) that says: "I'm deep in something. Don't interrupt."

When active:
- No notifications at all (not even decisions)
- Menu bar icon shows a small slash-through indicator
- Agents that need decisions queue silently
- Badge count freezes (stops updating to avoid peripheral distraction)

When deactivated:
- A catch-up summary appears in the popover: "While in Focus: 2 completed, 1 decision waiting"
- Badge count updates to current state
- Any queued decisions now surface normally

### macOS Focus Integration [MVP]

Stead respects macOS Focus modes automatically:

| macOS Focus | Stead Behavior |
|-------------|----------------|
| Do Not Disturb | No notifications. Badge still updates. |
| Work / Coding (custom) | Notifications for decisions only. |
| Sleep | Full silence. Morning briefing on wake. |
| Personal | Same as DND. |

Implementation: `UNUserNotificationCenter` already respects Focus. For badge suppression during Sleep, observe `NSWorkspace.willSleepNotification` / `didWakeNotification`.

---

## 3. Agent Negotiation (Concrete for MVP)

### Scenario 1: Two Agents Want the Same Port [MVP]

**What happens under the hood:**
1. Agent A (project picalyze) is running on port 3000.
2. Agent B (project qwer-q) tries to start and requests port 3000.
3. Stead intercepts via `stead resource claim port:3000 --for qwer-q`.
4. Stead sees port 3000 is held by picalyze.
5. Stead auto-assigns port 3001 to qwer-q and injects `PORT=3001` into its environment.

**What appears in the UI:**
Nothing. The human never knows this happened.

The resolution is logged silently in the contract metadata: "Port 3001 assigned (3000 was held by picalyze)." Visible only if you drill into the contract detail in the full window.

**If auto-resolution fails** (port range exhausted, or the port is hardcoded and can't be overridden):

The event escalates to an **Anomaly** in the popover:

```
ANOMALY
  qwer-q  Port conflict: needs 3000 (held by picalyze)
           [Assign 3001]  [Stop picalyze]
```

Two buttons. One click. Resolved.

**Design principle:** Negotiate silently. Escalate only on failure. The human sees conflicts only when automation can't handle them.

### Scenario 2: Agent Finished But Left Failing Tests [MVP]

**What happens under the hood:**
1. Agent completes its work on project picalyze.
2. Stead runs the contract's verification command: `cargo test --workspace`.
3. Tests fail. 2 passing, 1 failing.

**What appears in the UI:**

Badge count increments. The popover shows:

```
COMPLETED (needs review)
  picalyze  Auth refactor done
             3 files changed  ·  tests: 2/3 FAILED
             [View Failure]  [Retry Agent]
```

The state color is **yellow** (anomaly), not green (success). The word "FAILED" appears in the test summary. But this is still a badge, not a notification -- failing tests are bad but not time-critical.

**[View Failure]** opens the full window directly to the test output, syntax-highlighted, scrolled to the first failure.

**[Retry Agent]** creates a new contract: "Fix failing test in picalyze" with the test output as input context and the same verification command. One click to dispatch a fix.

**Design principle:** Surface the failure clearly. Offer one-click remediation. Don't panic -- failed tests aren't an emergency, they're a todo.

### Scenario 3: Agent Stuck in a Loop [MVP]

**Detection heuristics:**

1. **Token burn rate**: Agent has consumed >10x the expected tokens for the contract's complexity without producing meaningful output changes. Stead tracks token usage per contract via session adapter.

2. **Repeated tool calls**: Agent has called the same tool with the same arguments >5 times in the last 2 minutes. This catches `edit -> test -> fail -> edit same thing -> test -> fail` loops.

3. **Duration anomaly**: Contract has been executing for >3x the median duration of similar contracts (by verification command category). A 2-minute contract that's been running for 8 minutes is suspicious.

4. **No file changes**: Agent has been running for >5 minutes but the working directory shows zero file modifications. It's thinking but not acting.

**What appears in the UI:**

The contract moves to the **Anomaly** tier in the popover:

```
ANOMALY
  qwer-q  Possible loop: 8min running, no progress
           Token burn: 47k (expected ~10k)
           [View Agent Output]  [Stop & Reassign]
```

This earns a **badge**, not a notification. A looping agent is wasteful but not urgent -- it's not blocking other work, it's just burning tokens.

**[View Agent Output]** opens the full window showing the agent's recent session activity so you can diagnose what's happening.

**[Stop & Reassign]** cancels the current contract and creates a new one with the same task description plus a note: "Previous attempt looped. Consider a different approach." This gives the next agent (or the same agent in a fresh session) context about what didn't work.

**Design principle:** Detect anomalies through simple heuristics, not AI. Surface them as information, not alarms. Let the human decide whether to intervene.

---

## 4. The One-Click Resolution

### The Rule

Every attention-worthy event has exactly ONE primary action that resolves it. That action is reachable in one click from the popover, or one keystroke from the full window.

### Resolution Map [MVP]

| Event | Primary Action | Click Path | Keystroke |
|-------|----------------|------------|-----------|
| Contract completed, tests pass | **Approve** | Popover -> [Approve] | `a` |
| Contract completed, tests fail | **View Failure** | Popover -> [View Failure] | `Enter` |
| Decision needed (multiple choice) | **Choose option** | Popover -> item -> click option | `1`/`2`/`3` |
| Decision needed (yes/no) | **Yes** | Popover -> [Yes] | `y` |
| Port conflict (auto-resolve failed) | **Assign next port** | Popover -> [Assign 3001] | `Enter` |
| Agent stuck in loop | **Stop & Reassign** | Popover -> [Stop & Reassign] | `Enter` |
| Agent failed (crash/error) | **Retry** | Popover -> [Retry] | `r` |
| Multiple completions stacked | **Approve All** | Popover -> [Approve All Passing] | `A` (shift+a) |

### Popover Quick Actions [MVP]

For the most common events, the action button lives *inside* the popover item row. You don't need to open the full window.

```
┌──────────────────────────────────────┐
│  COMPLETED  (2)                      │
│  ┌──────────────────────────────────┐│
│  │ picalyze  Auth refactor         ││
│  │ 3 files · tests pass   [Approve]││
│  ├──────────────────────────────────┤│
│  │ stead  Workspace split          ││
│  │ 8 files · tests pass   [Approve]││
│  └──────────────────────────────────┘│
│                                      │
│  [Approve All Passing]               │
└──────────────────────────────────────┘
```

**[Approve All Passing]** resolves every completed contract where tests pass. One click clears your inbox. This is the power move: come back from lunch, see badge "5", open popover, one button, done.

### Decision Quick Actions [MVP]

Simple decisions surface directly in the popover:

```
┌──────────────────────────────────────┐
│  NEEDS DECISION  (1)                 │
│  ┌──────────────────────────────────┐│
│  │ qwer-q  Which auth strategy?    ││
│  │  [A: JWT]  [B: Session]  [C: OAuth]│
│  └──────────────────────────────────┘│
└──────────────────────────────────────┘
```

Click an option. Done. The popover dismisses. The agent continues. You never left your current context.

For complex decisions that need more context (code diffs, long descriptions), the popover shows:

```
│  │ qwer-q  API design choice       ││
│  │ 3 options · needs context  [Open]││
```

**[Open]** takes you to the full window, pre-navigated to that decision with all context visible.

### The Global Shortcut [MVP]

`Cmd+Shift+S` (configurable) is the single global hotkey.

- **Nothing needs attention**: Opens popover showing "All clear." (or doesn't open at all -- debatable)
- **Items pending**: Opens popover. Arrow keys to navigate, Enter to act, Escape to dismiss.
- **Already open**: Dismisses the popover.

The entire supervision loop without touching the mouse:

```
Cmd+Shift+S -> j/k to navigate -> Enter to select -> a to approve -> Esc
```

Five keystrokes. Under 3 seconds. Back to your code.

### Notification Actions [MVP]

macOS notifications include action buttons. When a notification fires (decision needed or critical failure), it carries the resolution:

```
stead -- qwer-q
Which auth strategy? JWT / Session / OAuth
[JWT]  [Session]  [OAuth]  [Open in stead]
```

You can resolve the decision *from the notification itself* without opening stead at all. The notification is the interface. Click the answer, the agent resumes, the notification clears.

For non-decision notifications:

```
stead -- picalyze
Port 3000 conflict with qwer-q
[Assign 3001]  [Open in stead]
```

One button. Resolved. Never opened the app.

---

## 5. Interaction Flows (End to End)

### Flow 1: The Perfect Day [MVP]

You start coding at 9am. Three agents are running across two projects.

- 9:00 - You glance at the menu bar. Monochrome icon. All fine.
- 9:47 - You're deep in code. You don't notice the badge "1" appear.
- 10:15 - You reach for the time display and peripherally see the badge. Badge "2" now.
- 10:15 - You open the popover. Two completions, both tests passing.
- 10:15 - Click [Approve All Passing]. Badge gone. Popover dismissed.
- 10:15 - Total time away from your code: 4 seconds.
- 11:30 - Menu bar icon turns red. A decision is needed.
- 11:35 - Notification appears (5min grace period passed): "qwer-q: pick API strategy"
- 11:35 - You click [JWT] on the notification.
- 11:35 - Agent resumes. Red dot clears. You never opened stead.
- 12:00 - Lunch. You lock your screen. macOS Sleep Focus activates.
- 13:00 - You return. Open stead popover. "While you were away: 1 completed, 1 anomaly."
- 13:00 - You approve the completion, investigate the anomaly (agent looped), click [Stop & Reassign].
- 13:01 - Inbox clear. Back to work.

Total stead interactions for the day so far: ~45 seconds. Total context switches: 3, all under 15 seconds.

### Flow 2: The Rough Patch [MVP]

Two agents conflict. One loops. One fails.

- 14:00 - Agent A and Agent B both try port 3000. Stead auto-resolves, assigns 3001 to B. You see nothing.
- 14:12 - Agent A finishes. Tests fail. Badge appears. You see it, open popover.
- 14:12 - "picalyze: Auth refactor -- tests: 2/3 FAILED". You click [View Failure].
- 14:13 - Full window opens to test output. You see the issue. Click [Retry Agent].
- 14:13 - New contract dispatched with failure context. Back to your code.
- 14:30 - Badge reappears. Agent B is flagged as anomaly: "Possible loop, 18min, no progress."
- 14:30 - You click [Stop & Reassign]. New contract with "previous attempt looped" context.
- 14:31 - Retry agent from 14:13 completes. Tests pass. Badge "1". You approve.
- 14:45 - Reassigned agent completes. Tests pass. Badge "1". You approve.

Total stead interactions: ~90 seconds. Two incidents handled. No panic, no dashboard staring, no cognitive overload.

---

## 6. Design Constraints

### Things Stead Must Never Do

1. **Never autoplay sound for completions.** Completion is good news. Good news can wait. [MVP]
2. **Never show a progress bar.** Progress bars invite watching. Watching is the opposite of supervising. [MVP]
3. **Never stack notifications.** If 3 completions happen in 2 minutes, one badge "3" -- not three separate notifications. [MVP]
4. **Never use modal dialogs.** Modals force immediate action. Supervision is asynchronous. [MVP]
5. **Never require a window to be open.** Every core supervision action works from the popover or notification. [MVP]
6. **Never animate continuously.** No spinners, no pulsing, no "breathing" indicators for running state. Static label "Running" is sufficient. Running is the normal state. Normal doesn't animate. [MVP]

### Things Stead Must Always Do

1. **Show aggregate state in one glance.** The menu bar icon answers "does anything need me?" without a click. [MVP]
2. **Provide one-action resolution.** Every attention-worthy event has a single primary action. [MVP]
3. **Respect time boundaries.** Focus Mode, macOS Focus, sleep -- stead goes fully silent when told to. [MVP]
4. **Batch intelligently.** Multiple completions in quick succession become one badge update, not N. [MVP]
5. **Degrade gracefully.** If the popover is too small for a decision, it opens the full window. If the notification can't carry an action, it links to the popover. Always a fallback. [MVP]

---

## 7. Summary: The Interaction Stack

```
Layer 0: Absence
         No agents running, no items pending.
         The icon is unremarkable. You don't think about stead.

Layer 1: Peripheral Awareness          [MVP]
         Icon state change, badge count.
         You know the state without looking.
         Cost: 1-2 attention points.

Layer 2: Glanceable Summary            [MVP]
         Popover with items needing action.
         5-second scan, one-click resolutions.
         Cost: 5-10 attention points.

Layer 3: Focused Interaction           [MVP]
         Full window for diffs, decisions, diagnostics.
         Used only when the popover isn't enough.
         Cost: 15-20 attention points.

Layer 4: Interrupt                     [MVP]
         macOS notification for blocked decisions.
         Used only when delay causes compounding cost.
         Cost: 30 attention points. Spent reluctantly.
```

Each layer exists to prevent the next one. Good peripheral awareness (Layer 1) means fewer popover opens (Layer 2). Good popover design (Layer 2) means fewer full window visits (Layer 3). Good auto-resolution means fewer notifications (Layer 4).

The entire paradigm is a **funnel of decreasing frequency and increasing cost.** Most events stay at Layer 0-1. A few reach Layer 2. Rarely do events reach Layer 3. Layer 4 is exceptional.

This is not a dashboard you check. It is a sense you develop.
