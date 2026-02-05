# Visionary Concepts: The Control Room Reimagined

**Created:** 2026-02-05
**Author:** Hallucinating Visionary
**Status:** Provocations for discussion

---

## The Core Provocation

The Control Room as currently conceived is still a *window*. A place you go to. A thing you open. That's 2010s thinking dressed in 2020s language.

The real question: **What if supervising agents felt like breathing?** You don't think about breathing. You don't open a breathing app. It happens. And when something needs attention, your body tells you -- not by opening a dashboard, but by changing how you feel.

Stead shouldn't be software you use. It should be an **ambient intelligence layer** woven into the computing experience itself.

---

## Concept 1: The Disappearing Interface [NOW]

**Premise:** The best Control Room is one you never see.

Current plan: SwiftUI app with attention-prioritized list. But a list is still a list. You still have to look at it. The radical move: stead has **no primary window at all**.

**How it works:**
- Stead lives as a menu bar icon. Tiny. Forgettable. A dot.
- The dot's color shifts with the aggregate state of your stead (calm blue = nothing needs you, amber = something finished, red = decision needed).
- Hover: a ephemeral panel appears showing only what demands attention. Not a dashboard -- a **whisper**.
- Click: expand to full detail only for items marked "needs human." Everything else is hidden.
- When all agents are running and nothing needs you: the icon disappears entirely. **Absence is the signal that all is well.**

**Why this matters:** Current notification patterns are binary -- interrupt or silence. This creates a **gradient of awareness**. You know the state without looking. Like hearing rain outside -- you're aware of it without checking a weather dashboard.

**Implementation path:**
- macOS menu bar extra (NSStatusItem)
- SwiftUI popover for the ephemeral panel
- Color calculated from contract states in stead-core
- Already fits the planned architecture (SwiftUI + stead-core FFI)

---

## Concept 2: Attention Thermostat [NOW]

**Premise:** Agents should modulate their output based on YOUR state, not theirs.

Right now, agents produce output at their own pace regardless of whether you're deep in flow, in a meeting, eating lunch, or asleep. That's insane. Your **heating system** knows not to blast heat when you've set it to "away mode." Why don't your agents?

**How it works:**
- Stead exposes an "attention budget" -- a number from 0-100.
- 100 = "I'm actively supervising, interrupt me freely"
- 50 = "I'm working on something, batch notifications"
- 10 = "I'm away, only critical failures"
- 0 = "Do not disturb. Accumulate everything."
- Agents query this via `stead attention` before deciding how to surface results.
- macOS Focus modes automatically adjust the thermostat (coding focus = 30, meeting = 5, sleep = 0).

**The clever part:** The thermostat creates **back-pressure** on agents. When attention is low, agents don't just suppress notifications -- they accumulate context. When you return to 100, you don't get 47 individual dings. You get a single synthesized briefing: "While you were away: 3 contracts completed, 1 needs review, 2 are blocked. Here's a 4-sentence summary."

**Implementation path:**
- `stead attention get/set` CLI commands
- SQLite field for current attention level
- macOS Shortcuts integration to auto-set from Focus modes
- Notification batching logic in stead-core

---

## Concept 3: The Peripheral Vision Display [NOW]

**Premise:** Your most important sense for monitoring is peripheral vision, not focused attention.

Air traffic controllers don't stare at individual blips. They see the whole field in their peripheral vision and only focus when something moves wrong. Your desktop has massive unused peripheral real estate.

**How it works:**
- A translucent, non-interactive strip along one edge of the screen (think: the gutter of a page).
- Each active project is a colored segment. Width = relative activity. Color = state (calm/working/needs attention/error).
- The strip is **below** all windows. You see it when switching spaces, between windows, in dead zones. You never interact with it directly.
- Segments pulse subtly when state changes. Not enough to grab focus. Enough to register peripherally.
- If you want detail: hover over a segment and the ephemeral panel (Concept 1) appears for that project.

**Why this matters:** This encodes agent state in a channel humans already process unconsciously -- spatial/color patterns in peripheral vision. No context switch required. You KNOW project X is fine because that green bar has been there for an hour. You KNOW project Y needs you because the red appeared 3 minutes ago and you've been subconsciously aware of it since.

**Implementation path:**
- NSWindow with `.canBecomeKey = false`, `.level = .desktopIcon`
- CALayer segments with CoreAnimation for smooth transitions
- stead-core provides project state aggregations
- Minimal resource usage (static most of the time, only animates on state change)

---

## Concept 4: Spatial Audio Agent State [NEXT]

**Premise:** Sound can convey state without stealing visual attention.

You can tell the difference between a healthy car engine and one that's about to break down. You know when your washing machine enters the spin cycle. Sound encodes state changes in a channel that doesn't compete with your coding focus.

**How it works:**
- Each project gets a subtle ambient tone, spatialized left-to-right based on its position in your stead.
- Idle projects are silent.
- Working projects emit a quiet "hum" -- like a distant server room. Barely audible. But its absence would be noticed.
- Completion: a specific tone that encodes success/failure without words. Like a microwave's "done" beep vs. an error buzz. But elegant.
- The volume scales inversely with your attention thermostat. High attention = quiet (you're already watching). Low attention = slightly louder (you're not watching, so audio carries more of the load).

**The radical part:** After a week of use, you'd be able to close your eyes and KNOW: "Two projects running, one just finished successfully, one is idle." The same way you know a room's state by its ambient sound.

**Implementation path:**
- AVAudioEngine with spatial audio (already in macOS)
- Procedural audio generation (no fixed sound files -- tones generated from state)
- AirPods Pro spatial audio support for even more precise positioning
- Haptic feedback on Apple Watch as fallback for audio

---

## Concept 5: Agent Negotiation Protocol [NEXT]

**Premise:** Agents shouldn't collide. They should negotiate.

Port conflicts, file locks, git merge conflicts -- these happen because agents are oblivious to each other. Stead already knows about all active agents. What if it was the **mediator**?

**How it works:**
- Before an agent claims a resource (port, file, branch), it asks stead: `stead resource claim port:3000 --for project-a`
- Stead checks: is port 3000 in use by another project? If yes, it doesn't just fail -- it **negotiates**.
- "Port 3000 is held by project-b. Offering port 3001. Accept?" (To the agent, not the human.)
- For git: "Branch `main` has uncommitted changes from agent-X. Options: (a) wait for agent-X, (b) create a fork, (c) agent-X to stash and yield."
- Agents become first-class citizens in a resource economy.

**The profound implication:** This turns stead from a passive observer into an **active coordinator**. It doesn't just show you what agents are doing -- it prevents them from stepping on each other. The *ding* problem partially disappears because the collision that would have caused it never happens.

**Implementation path:**
- `stead resource claim/release/list` commands
- Resource registry in SQLite (port assignments, file locks, branch claims)
- Port allocator: per-project ranges assigned automatically
- Integrates with existing USF adapters (know which CLI is running where)

---

## Concept 6: Cognitive Load Estimation [NEXT]

**Premise:** Stead can estimate your cognitive load and act on it.

Not through brain scanning. Through signals that already exist:
- How many projects have you context-switched between in the last hour?
- How long did you stay on each?
- Are you in a Focus mode?
- How many unreviewed completions are queued?
- What time is it? (3am work patterns differ from 10am)

**How it works:**
- Stead tracks context-switch frequency (which project windows are focused).
- A simple formula produces a "load score" -- not for display, but for behavior.
- High load: agents slow down. Notifications batch. Less important completions defer to tomorrow.
- Low load: agents can be more aggressive. More parallelism. More notifications.
- End of day: stead suggests which project to start tomorrow based on what's most stale/blocked.

**The key insight:** This isn't AI/ML in the gimmicky sense. It's a few heuristics and a timer. The sophistication is in **what it does with the estimate**, not in how it estimates.

**Implementation path:**
- SQLite table tracking focus events (which project, when, how long)
- Score formula: `switches_per_hour * pending_reviews * time_of_day_factor`
- Attention thermostat (Concept 2) auto-adjusts based on load score
- CLI: `stead load` shows current estimate and what it's affecting

---

## Concept 7: The Morning Briefing [NEXT]

**Premise:** Start every work session with a synthesis, not a scramble.

You sit down at your desk. Currently: open Slack, check email, open each project, try to remember where you left off. With stead:

**How it works:**
- First interaction of the day triggers the briefing.
- Stead's context generator synthesizes overnight activity into a narrative:

```
Good morning. While you were away:

qwer-q: Memory optimization completed. Tests pass.
         3 files changed. Awaiting your review.
         [Review] [Dismiss]

picalyze: Enterprise auth hit a wall -- agent couldn't
          resolve token refresh cycle. Needs your input.
          [See details] [Assign to fresh agent]

meinungsmache: Idle since 6pm. No pending work.
               [Queue task] [Archive for now]

Load forecast: 2 reviews, 1 decision. ~30min to clear the deck.
```

**Why this matters:** This is the Context Generator from NORTH_STAR made tangible. It doesn't just list what happened -- it SYNTHESIZES. It tells you what to do first and how long it'll take. It transforms the chaotic "what happened while I was gone" into a calm, prioritized start.

**Implementation path:**
- Trigger: first `stead` command of the day (or Control Room open after >4h idle)
- Context generator: query contracts by status, sessions by recency
- Template-based narrative (not LLM -- deterministic, fast, trustworthy)
- Rendered in the ephemeral panel (Concept 1) or terminal

---

## Concept 8: Generative Interface [FUTURE]

**Premise:** The UI should be different every time because your context is different every time.

A static dashboard with fixed panels assumes your needs are constant. They're not. Monday morning after vacation looks nothing like Friday afternoon mid-sprint. What if the interface itself adapted?

**How it works:**
- Stead's interface is generated from state, not designed statically.
- Zero pending work? The interface is literally empty. A blank calm space.
- Three critical reviews? The entire screen is those three reviews. Nothing else.
- Deep in one project? That project expands to fill everything. Others shrink to dots.
- The layout, density, even typography adjust based on cognitive load estimate (Concept 6).

**The radical implication:** There are no "views" to navigate. No tabs, no sidebar, no settings page. The interface is a **function of your current state**. It's not that the content changes -- the *structure* changes.

**This connects to an old idea:** Remember when iOS removed the static app grid with the App Library? That was a tiny step toward generative interface. Stead takes it further -- there IS no static interface. It's generated fresh every time, optimized for exactly what you need right now.

**Implementation path (speculative):**
- SwiftUI's declarative nature actually enables this well
- State machine in stead-core decides "interface mode" (empty/focused/review/overloaded)
- Each mode has different SwiftUI view hierarchies
- Transitions are animated so it feels organic, not jarring

---

## Concept 9: The Agent Economy [FUTURE]

**Premise:** When multiple agents want your attention, they should bid for it.

Not with money. With **urgency * importance * staleness * your-investment**.

**How it works:**
- Each pending item has an "attention score" computed from:
  - Urgency: how long has it been waiting? (exponential decay)
  - Importance: does it block other work? (dependency graph)
  - Investment: how much of YOUR time is already in this project? (sunk cost is real for humans)
  - Freshness: did the agent just finish, or has this been sitting for days?
- Items compete for limited screen real estate. Only the top N surface.
- This creates natural priority without you manually triaging.

**The interesting dynamic:** Agents effectively "earn" your attention by producing high-value output that blocks other work. An agent that completed a task blocking 3 others naturally rises above an agent that completed an isolated task. The system encodes project management logic without being a project management tool.

**Implementation path:**
- Score formula in stead-core, computed on each query
- Dependency graph from contract `blocks`/`blockedBy` fields
- Time-based decay functions
- Already fits the "attention priority" ordering from the current plan

---

## Concept 10: Temporal Spaces [FUTURE]

**Premise:** What if you could "rewind" your stead to any point in time?

Every agent session, every contract state change, every context switch is already logged. What if you could scrub through time like a video timeline?

**How it works:**
- A timeline scrubber shows the history of your stead.
- Drag it back to 3pm yesterday: see exactly what was running, what was pending, what you were focused on.
- Not just history -- **replay**. "What if I had reviewed this first instead of that?" Stead can simulate the cascade.
- This turns hindsight into foresight: "Last time I had 3 agents running and ignored the blocked one, it cost me 2 hours. This time, stead warns me."

**Implementation path (speculative):**
- Event sourcing in SQLite (already implicit in contract state changes)
- Temporal query: `stead at "2026-02-04T15:00:00"`
- Timeline view in SwiftUI
- Simulation engine for "what if" scenarios

---

## Concept 11: The Invisible Hand [FUTURE]

**Premise:** Stead should take actions you'd approve of without asking.

Not autonomy. **Predictability**. Like a good assistant who refills your coffee because they saw you're in the zone, not because you asked.

**How it works:**
- Stead learns patterns: "When project-A's tests pass, Jonas always reviews within 5 minutes."
- After enough repetitions: "Project-A tests passed. Auto-opening review context in 5... 4... 3..." (with visible countdown and cancel).
- Patterns it could learn:
  - "After a completion, you always check the diff first" -> auto-open diff
  - "You always assign port 3100 to picalyze" -> auto-assign without asking
  - "You never review at night" -> defer all reviews to morning briefing
  - "When two agents conflict on a port, you always give it to the older session" -> auto-resolve

**The key constraint:** Every automated action must be:
1. Reversible (undo within 10 seconds)
2. Visible (you see what happened)
3. Predictable (follows a pattern you'd recognize)
4. Deferrable (you can say "never do this again")

**Implementation path (speculative):**
- Pattern mining on SQLite event history
- Rule engine: `IF pattern_confidence > 0.9 AND action_is_reversible THEN auto_execute`
- "Stead suggestions" log where you can approve/reject learned patterns
- Starts fully manual, earns autonomy over time

---

## Concept 12: Agent Presence [FUTURE]

**Premise:** Agents should feel like they're *there*, not like background processes.

When a colleague is working at the desk next to you, you have ambient awareness of their presence. The typing sounds. The occasional muttering. The shift in chair when they hit a blocker. You know their state without looking. Agents have none of this.

**How it works (speculative):**
- Each active agent gets a subtle "presence indicator" -- not a status badge, but something that conveys *aliveness*.
- Perhaps a very faint animation -- like breathing -- that speeds up when the agent is actively executing and slows when it's waiting.
- Stalled agents go still. Failed agents dim.
- In a spatial computing context (Vision Pro): agents are entities in your peripheral space. You sense their presence like you'd sense a colleague.

**Why this matters:** It transforms agents from abstract task runners into something your brain can model as "someone working alongside me." This isn't anthropomorphization for its own sake -- it's leveraging the social cognition circuits humans already have. Your brain is incredibly good at tracking the state of nearby entities. Let it.

---

## Synthesis: The Vision Stack

These concepts aren't independent. They form layers:

```
Layer 4: Autonomy        [FUTURE]
         The Invisible Hand, Agent Economy
         "Stead acts on your behalf"

Layer 3: Prediction       [FUTURE]
         Generative Interface, Temporal Spaces, Agent Presence
         "Stead anticipates your needs"

Layer 2: Awareness        [NEXT]
         Spatial Audio, Agent Negotiation, Cognitive Load, Morning Briefing
         "Stead understands your state"

Layer 1: Subtlety         [NOW]
         Disappearing Interface, Attention Thermostat, Peripheral Vision
         "Stead is present without demanding attention"
```

**The progression:** Start by being invisible (Layer 1). Then become aware (Layer 2). Then anticipate (Layer 3). Then act (Layer 4). Each layer requires the trust built by the previous one.

**The anti-pattern to avoid:** Jumping straight to Layer 4 (autonomy) without earning trust through Layers 1-3. Every "AI assistant" that tries to act autonomously without first proving it understands you will be turned off within a week.

---

## What This Means for the Current Plan

The existing M5 (SwiftUI Control Room MVP) can be reimagined:

**Instead of:** A window with a contract list grouped by status.

**Start with:**
1. Menu bar icon with state-encoded color (Concept 1) -- this IS the MVP.
2. Ephemeral popover showing only "needs attention" items.
3. Attention level tied to macOS Focus modes (Concept 2).
4. Peripheral vision strip as an optional layer (Concept 3).

This is less code than a full SwiftUI app, delivers more value, and establishes the paradigm: **stead is ambient, not an app you open.**

The full window view becomes a drill-down, not the primary experience. You only open it when you want to audit or explore. 95% of the time, the menu bar dot tells you everything.

---

## The One-Line Vision

**Stead is not software you use. It is a sense you develop.**
