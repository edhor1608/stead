# Control Room UI Exploration

Date: 2026-02-02

## The Core Metaphor

The control room metaphor isn't just branding - it fundamentally changes design decisions. The key insight: **you're not managing work, you're supervising autonomous systems**.

| Dashboard Mindset | Control Room Mindset |
|-------------------|----------------------|
| "What tasks are in progress?" | "What's the current system state?" |
| "Which task should I work on next?" | "Does anything require my intervention?" |
| "How much work is left?" | "Are all systems nominal?" |
| "Update the status" | "Acknowledge the alert" |

Real-world control rooms - air traffic control, NASA mission control, nuclear power plants - share these characteristics:
- Information is organized by **urgency and system status**, not by project hierarchy
- Operators scan for anomalies, not for tasks to pick up
- The default state is "everything is fine" - the UI's job is to surface when it isn't
- Attention is a scarce resource the interface must respect

---

## Information Architecture

### Primary Organization: By Attention Priority

The fundamental question: **What needs my attention right now?**

Proposed hierarchy (top to bottom):

1. **Needs Decision** - Blocked on human input
2. **Anomalies** - Running but unexpected (errors, timeouts, unusual patterns)
3. **Completed** - Finished, awaiting review/approval
4. **Running** - Active execution (background awareness)
5. **Queued** - Scheduled to run (lowest priority)

This inverts the typical task board where "To Do" is prominent. In supervision mode, queued work is boring - it's handled. What matters is deviation from expected flow.

### Secondary Organization: By Project

Within each attention tier, group by project. But project is secondary to status. A blocked item in Project A appears before a running item in Project A.

This means:
- No "Project A view" as the default
- Instead: cross-project view organized by supervision needs
- Project filtering available but not primary

### Information Density Considerations

Air traffic control research (see [MEFISTO project](https://www.ercim.eu/publication/Ercim_News/enw32/paterno.html)) emphasizes that high-stakes interfaces need to balance:
- **Overview**: See everything at once
- **Detail**: Drill into specific items
- **Context**: Understand relationships

The solution is **progressive disclosure**:
- First glance: counts and status indicators
- Scan: one-line summaries with key metadata
- Focus: full detail panel with logs, previews, actions

---

## Attention Direction

### The Urgency Gradient

Not all "needs attention" items are equal. Design needs to communicate urgency without alarm fatigue.

**Urgency levels:**

| Level | Meaning | Visual Treatment | Example |
|-------|---------|------------------|---------|
| Critical | Blocking other work, time-sensitive | Red accent, persistent indicator | Auth token expired, blocking 3 agents |
| Decision | Needs human input, not time-critical | Yellow/amber accent | "3 options for API design - pick one" |
| Review | Completed work awaiting verification | Blue accent | PR ready, tests passed |
| Info | FYI, no action required | Subtle/gray | "Agent completed, results available" |

Research on notification design ([Attelia system](https://www.researchgate.net/publication/279530024_Attelia_Reducing_User's_Cognitive_Load_due_to_Interruptive_Notifications_on_Smart_Phones)) shows that breaking attention at task boundaries reduces cognitive load by 46%. The control room should:
- Surface critical items immediately
- Batch non-critical updates
- Provide a "catch up" summary when you return to the interface

### Avoiding Alarm Fatigue

From [FAA decision support tool guidelines](https://hf.tc.faa.gov/publications/2019-atc-decision-support-tool/full_text.pdf):
- False positives destroy trust
- Constant alerts lead to ignored alerts
- The system state should be obvious without reading alerts

Design implications:
- Default state is calm/empty in "needs attention" areas
- Use progressive severity (subtle before loud)
- Track alert history to catch repeated false positives

### The "What Changed" Problem

Per [Smashing Magazine's real-time dashboard research](https://www.smashingmagazine.com/2025/09/ux-strategies-real-time-dashboards/):
> "Repeat visitors are generally interested in new information. Showing what has changed and just highlighting it can considerably reduce the clutter."

Control room implications:
- Clear "new since last visit" indicators
- Change highlighting on return
- Optional "summary since last session" view

---

## Interaction Patterns for Supervision

### The Supervision Loop

Typical workflow:

```text
1. Scan overview (5 seconds)
2. Identify items needing attention (if any)
3. Drill into specific item
4. Take action or dismiss
5. Return to overview
```

This is different from task management:
```text
1. Pick task from queue
2. Work on task
3. Mark complete
4. Repeat
```

UI implications:
- Overview must be scannable in seconds
- Drill-down must be instant
- Actions should be available in context (not buried in menus)
- "Return to overview" is the primary navigation action

### Action Vocabulary

What can a human do in supervision mode?

| Action | Meaning | When |
|--------|---------|------|
| **Approve** | Accept completed work | After verification |
| **Reject** | Send back for retry | Failed verification |
| **Decide** | Provide input for blocked agent | Escalation queue |
| **Cancel** | Stop execution | Runaway or wrong direction |
| **Pause** | Temporarily halt | Need more context |
| **Resume** | Continue paused work | Ready to proceed |
| **Escalate** | Flag for deeper review | Beyond current authority |
| **Dismiss** | Acknowledge, no action needed | Informational items |

These should be the primary UI actions - prominent, accessible, keyboard-navigable.

### Keyboard-First Design

Control rooms prioritize speed. From [PagerDuty UX patterns](https://university.pagerduty.com/incident-dashboard):
> "Implement keyboard shortcuts for common actions like blocking an IP or escalating an incident, which can speed up responses significantly."

Essential shortcuts:
- `j/k` or arrows: Navigate items
- `Enter`: Open detail
- `Escape`: Return to overview
- `a`: Approve
- `r`: Reject
- `d`: Decide/respond
- `c`: Cancel
- `?`: Show all shortcuts

---

## Multi-Project Without Overwhelm

### The Core Challenge

With 5+ active projects, naive approaches fail:
- Tabs per project: Can't see cross-project state
- Single list: Overwhelmed by volume
- Dashboard per project: Requires constant switching

### Proposed Solution: Attention-First, Project-Second

The default view shows ALL projects, organized by attention tier:

```text
┌─────────────────────────────────────────────┐
│ NEEDS DECISION (2)                          │
│ ┌─────────────────────────────────────────┐ │
│ │ picalyze: API rate limit strategy?      │ │
│ │ qwer-q: Memory format migration          │ │
│ └─────────────────────────────────────────┘ │
├─────────────────────────────────────────────┤
│ REVIEW READY (3)                            │
│ ┌─────────────────────────────────────────┐ │
│ │ meinungsmache: Rebranding PR ready      │ │
│ │ qwer-q: Protocol fix complete           │ │
│ │ picalyze: Auth flow refactor done       │ │
│ └─────────────────────────────────────────┘ │
├─────────────────────────────────────────────┤
│ RUNNING (4)                    ▼ collapsed  │
├─────────────────────────────────────────────┤
│ QUEUED (12)                    ▼ collapsed  │
└─────────────────────────────────────────────┘
```

Lower-priority sections are collapsed by default. Expand on demand.

### Project Indicators

Each item shows project identity but doesn't organize by it:
- Color coding per project (consistent across sessions)
- Project name prefix on each item
- Quick filter: "Show only picalyze" (temporary focus mode)

### Focus Mode

For deep work on one project:
- Toggle to single-project view
- Everything else fades/hides
- Exit returns to cross-project view
- Notifications from other projects still surface (but marked as "other project")

---

## The Decision Queue

### What "Needs Human Decision" Looks Like

This is the most critical UI component. Agents will escalate when they:
- Hit ambiguity in requirements
- Need to choose between multiple valid approaches
- Encounter errors they can't resolve
- Reach explicit checkpoints (design review, etc.)

### Decision Item Structure

Each item needs:

```text
┌───────────────────────────────────────────────────────┐
│ [picalyze] API Rate Limiting Strategy                 │
│                                                       │
│ Context: Implementing enterprise feature. Hit API     │
│ rate limits. Three possible approaches:               │
│                                                       │
│ Options:                                              │
│ ┌─────────────────────────────────────────────────┐   │
│ │ ○ Exponential backoff (simple, may be slow)     │   │
│ │ ○ Request queue with batching (more complex)    │   │
│ │ ○ Upgrade API tier (costs $200/mo)              │   │
│ └─────────────────────────────────────────────────┘   │
│                                                       │
│ [More context] [View related code]                    │
│                                                       │
│         [Choose Option] [Need More Info] [Cancel]     │
└───────────────────────────────────────────────────────┘
```

Key elements:
- **Clear title**: What's being decided
- **Context**: Why the agent escalated
- **Options**: Structured choices when applicable
- **Evidence links**: Related code, docs, previous decisions
- **Actions**: Primary (decide), secondary (request clarification, cancel)

### Decision Types

| Type | UI Treatment |
|------|--------------|
| **Multiple choice** | Radio buttons or cards |
| **Yes/No** | Two prominent buttons |
| **Text input** | Text field with submit |
| **Approval** | "Approve" / "Reject" / "Request changes" |
| **Open-ended** | Chat-style interface to converse with agent |

### Decision History

Every decision should be logged and accessible:
- What was decided
- When
- What context was provided
- What happened after

This creates an audit trail and helps refine agent behavior over time.

---

## Previews and Artifacts

### Types of Preview Content

Agents produce artifacts that need human review:

| Artifact | Preview Need |
|----------|--------------|
| Code changes | Diff viewer |
| UI changes | Screenshots, deployed preview |
| API changes | Request/response examples |
| Docs | Rendered markdown |
| Tests | Pass/fail summary, coverage |
| Deployments | Link to environment |

### Preview UX Patterns

**Inline previews** - Show immediately without navigation:
- Thumbnail screenshots
- Diff summary (files changed, insertions/deletions)
- Test result badges

**Expanded previews** - One click to see more:
- Full diff viewer (split view, per [diff2html](https://diff2html.xyz/))
- Full-size screenshots
- Deployment links that open in new tab

**Comparison previews** - For UI changes:
- Before/after slider
- Side-by-side
- Overlay diff

### Diff Viewer Considerations

From code review UI research:
- Split view (old left, new right) is standard
- Syntax highlighting essential
- Line-level comments if feedback is needed
- Collapse unchanged sections
- Show file-level overview (which files changed, how much)

React-based options: [react-diff-viewer](https://github.com/praneshr/react-diff-viewer), [git-diff-view](https://github.com/MrWangJustToDo/git-diff-view)

### Screenshot Management

For UI work, agents should capture:
- Full page screenshot
- Specific component screenshots
- Interaction sequences (click states, etc.)

Display considerations:
- Thumbnail grid for multiple screenshots
- Lightbox for full-size viewing
- Annotation capability (circle issues, add notes)
- Comparison mode for before/after

---

## Real-Time Update Strategy

### Technology Choice: SSE Over WebSockets

Based on [2025 research comparing SSE vs WebSockets](https://medium.com/codetodeploy/why-server-sent-events-beat-websockets-for-95-of-real-time-cloud-applications-830eff5a1d7c):

> "For 90% of dashboards, crypto prices, stock tickers, notifications - SSE is better and simpler."

Why SSE for control room:
- Primarily server-to-client updates (agent status changes)
- Client-to-server is infrequent (human decisions)
- Better firewall compatibility
- Automatic reconnection
- Works with HTTP/2 multiplexing

WebSockets would only be needed if:
- Implementing real-time chat with agents
- Streaming agent output live (terminal-style)

Recommendation: **SSE for status updates, standard HTTP for actions**

### Update Frequency

| Data Type | Update Strategy |
|-----------|----------------|
| Status changes | Push immediately (SSE) |
| Log output | Batch every 1-2 seconds |
| Metrics (tokens, cost) | Poll every 30 seconds |
| Screenshots | Push on completion |

### Optimistic UI

When human takes action:
1. Update UI immediately (optimistic)
2. Send action to backend
3. Confirm or rollback on response

This makes the interface feel responsive even with network latency.

### Connection Health

Show connection status subtly:
- Green indicator when connected
- Yellow when reconnecting
- Red with "reconnecting..." message when down
- Manual refresh fallback

---

## Platform Decision: Desktop vs Web

### Options Evaluated

| Platform | Pros | Cons |
|----------|------|------|
| **Web only** | Zero install, always updated, works everywhere | No offline, no system integration, tab gets lost |
| **Electron** | Full system access, mature ecosystem | 120MB+ install, high memory usage |
| **Tauri** | Small (3-10MB), low memory, Rust backend | Smaller ecosystem, WebView quirks |
| **Native** | Best performance, best OS integration | 3x development effort (mac/win/linux) |

### Recommendation: Tauri + Web

Based on [2025 Tauri vs Electron comparisons](https://www.gethopp.app/blog/tauri-vs-electron):

> "A productivity app team reported that switching from Electron to Tauri reduced their installer size from 120MB to 8MB and cut cold-start time by 70%."

Hybrid approach:
1. **Primary**: Tauri desktop app
   - Lives in system tray/menu bar
   - Global hotkey to surface
   - Native notifications
   - Offline capability
   - Small footprint

2. **Secondary**: Web version
   - For quick access from anywhere
   - For users who don't want to install
   - Shares 95% of code (same frontend)

### Why Desktop Matters for Control Room

The control room needs to be:
- Always accessible (global hotkey)
- Visible at a glance (menu bar indicator)
- Interruptive when needed (native notifications)
- Not lost in browser tabs

Web apps can't do these things well. A lightweight desktop wrapper (Tauri) provides them without the bloat of Electron.

### Desktop App Behavior

- **Menu bar presence**: Icon shows aggregate status (green/yellow/red)
- **Global hotkey**: Cmd+Shift+S to show/hide
- **Notifications**: Native OS notifications for escalations
- **Background**: Runs minimized, receives updates via SSE
- **Offline**: Cache last known state, queue actions

---

## Hard UX Problems

### 1. The "Always Something" Problem

If agents are always running, there's always activity. The UI must distinguish:
- Normal activity (ignorable)
- Noteworthy activity (glanceable)
- Required activity (must attend)

Without this, the control room becomes another source of noise.

**Solution direction**: Strong visual hierarchy, aggressive default collapsing, attention budget concept.

### 2. Context Switching Cost

When a decision is needed, the human has zero context. They were doing something else.

**Solution direction**: Each escalation must include sufficient context inline. Don't require reading logs. Provide "what you need to know" summary at the top of every decision item.

### 3. Trust Calibration

How much should humans trust agent output? Under-trust = too much review overhead. Over-trust = mistakes slip through.

**Solution direction**:
- Confidence indicators from agents
- Track historical accuracy
- Adaptive verification (more scrutiny for new/uncertain agents)
- "Spot check" mode for random deep review

### 4. Notification Interrupt Budget

Research shows [notifications have massive cognitive cost](https://netpsychology.org/the-neuroscience-of-notifications-why-you-cant-ignore-them/):
> "It takes an average of 23 minutes to fully return to a task after an interruption."

**Solution direction**:
- Configurable interrupt thresholds
- Batch non-critical notifications
- "Focus mode" that suppresses everything except critical
- Daily/weekly summary digests

### 5. Multi-Agent Coordination Visibility

With multiple agents working simultaneously, dependencies and conflicts become complex.

**Solution direction**:
- Dependency graph visualization
- Conflict detection and alerting
- Resource contention indicators
- "What's blocking what" view

### 6. Scale Problem

5 projects is manageable. 50 projects? 500?

**Solution direction**:
- Hierarchical grouping (org > team > project)
- Smart defaults based on recent activity
- Search and filter as primary navigation at scale
- Delegation (some projects auto-approve certain decisions)

### 7. The "I Was Gone for a Day" Problem

What happened while I was away?

**Solution direction**:
- Session-based "what's new" summary
- Audit log with filtering
- "Rewind" capability to see state at any point
- Email/Slack summary for extended absences

---

## Design Principles Summary

1. **Attention is scarce**: Default to showing nothing in attention areas. Earn screen presence.

2. **Status over lists**: Organize by "what needs me" not by "what exists."

3. **Context at point of need**: Decisions include all necessary context inline.

4. **Keyboard-first**: Power users navigate without mouse.

5. **Progressive disclosure**: Summary first, detail on demand.

6. **Calm technology**: The best control room is one you can ignore when things are working.

7. **Trust but verify**: Make verification easy, not mandatory for everything.

8. **Cross-project by default**: Single project is a filter, not a mode.

---

## Next Steps

1. **Wireframe the core views**
   - Multi-project overview
   - Decision queue item
   - Verification/preview flow

2. **Define the data model**
   - What's an "item" (contract? agent run?)
   - Status states and transitions
   - Metadata required for each view

3. **Prototype the attention system**
   - How agents signal urgency
   - How urgency translates to UI treatment
   - How humans configure their attention budget

4. **Technical spike: Tauri + SSE**
   - Validate the architecture
   - Test notification flow
   - Measure memory/performance

---

## References

### Control Room Design
- [MEFISTO: ATC User Interface Design](https://www.ercim.eu/publication/Ercim_News/enw32/paterno.html)
- [FAA Decision Support Tool Design](https://hf.tc.faa.gov/publications/2019-atc-decision-support-tool/full_text.pdf)
- [NASA Human Integration Design Handbook](https://www.dau.edu/cop/hsi/documents/nasa-human-integration-design-handbook)
- [JPL Human Centered Design Group](https://hi.jpl.nasa.gov/)

### Dashboard UX
- [Smashing Magazine: UX Strategies for Real-Time Dashboards](https://www.smashingmagazine.com/2025/09/ux-strategies-real-time-dashboards/)
- [UXPin: Dashboard Design Principles](https://www.uxpin.com/studio/blog/dashboard-design-principles/)
- [Pencil & Paper: Dashboard UX Patterns](https://www.pencilandpaper.io/articles/ux-pattern-analysis-data-dashboards/)

### Attention & Notifications
- [Attelia: Reducing Cognitive Load from Notifications](https://www.researchgate.net/publication/279530024_Attelia_Reducing_User's_Cognitive_Load_due_to_Interruptive_Notifications_on_Smart_Phones)
- [ACM: Attuning Notification Design to User Goals](https://dl.acm.org/doi/10.1145/636772.636800)
- [Neuroscience of Notifications](https://netpsychology.org/the-neuroscience-of-notifications-why-you-cant-ignore-them/)

### Incident Management
- [PagerDuty Incident Dashboard](https://university.pagerduty.com/incident-dashboard)
- [Datadog Incident Management](https://docs.datadoghq.com/monitors/incident_management/)

### Remote Supervision
- [Teleoperation GUI for Automated Vehicles](https://www.mdpi.com/2414-4088/9/8/78)
- [Human-Machine Interface Design for Autonomous Vehicles](https://www.sciencedirect.com/science/article/pii/S2405896316322418)

### Technical
- [SSE vs WebSockets 2025](https://medium.com/codetodeploy/why-server-sent-events-beat-websockets-for-95-of-real-time-cloud-applications-830eff5a1d7c)
- [Tauri vs Electron Performance](https://www.gethopp.app/blog/tauri-vs-electron)
- [React Diff Viewer](https://github.com/praneshr/react-diff-viewer)
