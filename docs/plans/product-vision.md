# Product Vision

## The Problem

Every tool you use was designed for a world where one human works on one thing.

Your terminal assumes it has your attention. Your browser assumes its tabs belong to one train of thought. Your IDE assumes the project open is the project you're working on.

That world is over.

Today, a developer runs three, four, five AI agents simultaneously. Each one is doing real work -- writing code, running tests, building features. And every time one finishes, it does the worst possible thing: it makes a sound.

*Ding.*

That sound is a lie. It pretends to be helpful. "Hey, something finished!" But it carries no context. It doesn't tell you which project. It doesn't tell you what changed. It doesn't restore where you were. It just *grabs your attention* -- the single most expensive resource you have -- and throws it into the void.

So you go hunting. Which terminal tab? Which browser window? Which port is that running on? Oh, that localhost is broken because two projects are fighting over port 3000. And while you're sorting that out --

*Ding.*

You've become an interrupt handler. Your brain -- the thing that's supposed to be creative, to make judgment calls, to see the big picture -- is spending its cycles on the mechanical act of *finding things*.

The existing tools don't solve this. They can't. They were designed around a metaphor that no longer holds: one human, one task, full attention. Tmux helps with terminals but adds another layer to navigate. Docker solves port conflicts but not the attention problem. Agent orchestration GUIs are just more windows to manage.

These tools organize by application. Terminal over here. Browser over there. IDE in the corner. But the developer's mental model is organized by *project*. "Show me everything about Project X" is a question no tool can answer.

## The Insight

Everyone is trying to build better tools for managing agents.

We're building something different. We're building for the *space between* the agent and the human. The moment after the ding. The five seconds where you're trying to figure out what happened, where it happened, and whether you need to care.

That space -- between notification and comprehension -- is where all productivity dies. Not in the execution. Not in the coding. In the *context switch that never completes*.

Here's what we understand that nobody else does: **the operating system's fundamental unit of organization is wrong.** It's organized by application. It should be organized by attention. Not "which app," but "which thing deserves my focus right now."

This isn't a developer tool. It's a new category. The same way the iPhone wasn't a better phone -- it was a computer that made calls -- stead isn't a better terminal or a better dashboard. It's a control room for a new kind of work: supervising autonomous systems.

The paradigm shift already happened. Agents work. Humans supervise. But every tool still assumes humans work and agents assist. We're building for the world that actually exists.

## The Product

**Stead is a control room that turns agent chaos into calm supervision.**

It does three things:

**1. Attention Priority.**
One view across every project, every agent, every CLI. Not organized by app or by project -- organized by what needs you. Decisions at the top. Anomalies next. Completed work awaiting review. Running tasks. Queued tasks. The default state is *nothing*. Silence. Calm. Items earn their way onto your screen.

**2. Context Restoration.**
When something does need you, one action restores everything. Not a ding -- a complete handoff. What the agent did. What changed. Where you left off. The terminal session. The right browser tabs. The right files. You don't *search* for context. It comes to you, fully formed.

**3. Contracts, Not Tasks.**
Agents don't work on "tasks." They execute contracts: input specification, output specification, verification criteria, rollback procedure. This is how you supervise without micromanaging. You don't watch agents type -- you define what done looks like and let the system verify it.

What stead does NOT do:
- It does not replace your terminal, browser, or IDE
- It does not run AI agents (it orchestrates the ones you already use)
- It does not manage human teams (this is not Jira)
- It does not require you to change your workflow to fit it

## The Experience

You install stead. A menu bar icon appears. Nothing else.

You open it. A clean window. Almost empty. A faint label: "No projects need attention." You feel a small relief. That's the point.

You start a Claude Code session in your terminal. Stead notices. A quiet entry appears: "qwer-q: running." You start another in a different project. Another entry. You go back to your own work.

Three minutes later, the menu bar icon shows a subtle badge. You glance at the control room. The top item, highlighted: "qwer-q: memory fix complete. 3 files changed. Tests pass. Review?"

You click it. Your terminal switches to the qwer-q session. The browser opens to the right localhost (port auto-assigned, no collision). The diff is ready. You read it, approve it, move on. Ten seconds. No hunting. No guessing.

Meanwhile, the other agent is still running. It's in the "Running" section, below. You don't think about it. It hasn't earned your attention yet.

The "wow" moment is the absence of friction. It's the first time an agent finishes and you don't feel a spike of anxiety. You don't scramble. You don't lose your train of thought. The information comes to you, organized, complete, and at the right time.

It feels like having a great assistant who never interrupts you at the wrong moment but is always ready when you turn to look.

## The Strategy

**Who uses this first?**
Solo developers running multiple AI coding agents daily. Specifically: developers using Claude Code, Codex CLI, or OpenCode across 3+ active projects. They feel the pain acutely because they're the ones who've pushed furthest into agent-driven development. They don't need convincing that the problem exists -- they live it.

**What's the wedge?**
`stead session list` -- unified visibility across every AI CLI, right now, from the terminal. No setup, no configuration, no account. Install the binary, run the command, see every agent session across Claude Code, Codex, and OpenCode in one view. That's the handshake. The first taste of "oh, someone finally built this."

From there: contracts (`stead run "fix the bug" --verify "cargo test"`), then the native Mac control room app. Each layer adds value. Each layer makes the previous one better.

**How does it grow?**
- **Phase 1: CLI** -- Contracts + unified session visibility. Developer word of mouth.
- **Phase 2: Control Room** -- Native Mac app. The visual product people screenshot and share.
- **Phase 3: Ecosystem** -- Other CLIs, other agents, other platforms. The universal adapter layer (USF) means any agent runtime can plug in.

The long-term position: stead becomes the operating layer between humans and AI agents. Not by replacing anything, but by being the thing that makes everything else work together.

One more thing.

The name. *Stead.* A farmstead. The whole place -- not a single tool in the barn. Your projects, your agents, your attention -- one home for all of it. A place of calm in the middle of productive chaos.
