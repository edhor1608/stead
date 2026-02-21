# Developer Onboarding Experience

**Created:** 2026-02-05
**Status:** Draft

---

> Alignment note (2026-02-11): This document is onboarding UX direction.
> For canonical concept-level decisions, see `docs/plans/canonical-decisions-2026-02-11.md`.
> For cross-doc precedence, see `docs/plans/docs-authority-map.md`.

## Philosophy

The first 5 minutes with stead should feel like a revelation, not a tutorial. The developer already has the pain -- scattered agent sessions, port collisions, context-switching anxiety. Stead doesn't need to explain the problem. It needs to show, in under 60 seconds, that someone finally built the answer.

No configuration. No accounts. No YAML files. Install the binary, run one command, see everything you were missing.

---

## First 5 Minutes

### 1. Install (10 seconds)

```bash
brew install stead
```

Or for Rust developers:

```bash
cargo install stead
```

That's it. No `stead init`, no `stead configure`, no "add this to your shell profile." The binary lands in PATH. Ready.

### 2. First Run: `stead` with no arguments (the wow moment)

The developer types `stead`. What happens next is the entire pitch:

```
stead v0.3.0

  SESSIONS                                    across 3 CLIs

  Claude Code (4 active)
    picalyze      main          "Fix auth token refresh..."       3m ago
    qwer-q        feat/api      "Implement rate limiting..."     12m ago
    stead         main          "Add SQLite storage layer..."    47m ago
    meinungsmache main          "Redesign landing page..."        2h ago

  Codex CLI (1 active)
    picalyze      main          "Generate API docs..."            5h ago

  No contracts yet. Create one:
    stead run "fix the bug" --verify "cargo test"

  Control Room: brew install --cask stead-app
```

What just happened:

- stead auto-discovered every Claude Code, Codex CLI, and OpenCode session on the machine
- Grouped them by CLI, showing project name, branch, what they're working on, and recency
- No configuration needed -- it read `~/.claude/projects/`, `~/.codex/sessions/`, `~/.local/share/opencode/`
- The developer sees, for the first time ever, a single unified view of all their agent work

This is the handshake. The "oh, someone finally built this" moment.

**Why this works:** The developer didn't ask for this. They just typed `stead` and got something they didn't know they needed. That gap between expectation (a help message) and reality (instant visibility) is what makes people tweet.

### 3. Explore (`stead session show`) (30 seconds)

The developer picks a session and digs in:

```bash
stead session show claude-abc123
```

```
  Session: claude-abc123
  CLI:     Claude Code
  Project: picalyze (~/repos/picalyze)
  Branch:  main
  Model:   anthropic/claude-sonnet-4-5

  Started: 12 minutes ago
  Messages: 4 user, 6 assistant, 23 tool calls

  Summary
  "Fix auth token refresh bug -- tokens were expiring mid-request
   because the refresh check used wall clock instead of monotonic time."

  Use --full for complete timeline.
```

Now the developer has context restoration. Not a ding -- actual information about what an agent did and why. They can glance at any session and know whether it needs them.

### 4. First Contract (`stead run`) (60 seconds)

The developer tries the core workflow:

```bash
cd ~/repos/picalyze
stead run "fix the flaky test in auth_test.rs" --verify "cargo test auth"
```

```
  Contract created: qk4f-9m2
  Status: running
  Verify: cargo test auth

  Watching... (Ctrl+C to detach, contract continues)
```

The contract runs. The verification command executes automatically when the agent finishes. The developer doesn't have to remember to check -- stead checks for them.

When it completes:

```
  Contract qk4f-9m2: PASSED
  Duration: 1m 47s
  Files changed: 2 (+12 -8)
  Verification: cargo test auth  (exit 0)
```

They just experienced the core loop: define what "done" looks like, let the agent work, get a verified result.

### 5. List Everything (`stead list`) (10 seconds)

```bash
stead list
```

```
  PASSED  (1)
    picalyze  qk4f-9m2  Fix the flaky test in auth_test.rs   2m ago

  No running or pending contracts.
```

The output mirrors the attention-priority ordering from the Control Room UX. Same mental model, CLI or GUI.

---

## CLI Polish

### Default Behavior (`stead` with no args)

No arguments = the status overview shown above. This is a `stead status` by another name. The developer never needs to remember a command to get situational awareness -- they just type the tool's name.

This is deliberate. Most CLIs show `--help` with no args. That's for tools you use occasionally. Stead is something you glance at constantly. The default should be the thing you need most often: "what's going on right now?"

`stead --help` still works for discovering commands.

### `stead status` (explicit alias)

`stead status` produces identical output to bare `stead`. Exists for discoverability and for scripts, but the bare command is the intended muscle memory.

### Colored Output

Color is used surgically, not decoratively. Same philosophy as the Control Room design language: color carries meaning.

| Element | Color | Rationale |
|---------|-------|-----------|
| `PASSED` status | Green | Success, universal meaning |
| `FAILED` status | Red | Failure, demands attention |
| `RUNNING` status | Dim/gray | Background, doesn't need you |
| `PENDING` status | Dim/gray | Waiting, doesn't need you |
| `NEEDS DECISION` | Bold red | Only state that breaks focus |
| Contract ID | Dim | Machine identifier, secondary info |
| Project name | Bold white | Human anchor, primary identifier |
| Branch name | Cyan | Git convention, already in muscle memory |
| Timestamps | Dim | Metadata, low priority |
| Section headers | Bold | Structure, scannable |
| `stead run` output progress | Yellow | Active, in-flight |
| Error messages | Red | Standard convention |

**No color when piped.** If stdout isn't a TTY, all color is stripped. `stead list | grep picalyze` just works.

**Respect `NO_COLOR`.** If the `NO_COLOR` environment variable is set, disable all color. This is the [no-color.org](https://no-color.org) standard.

### Commands That Get Color Treatment

- `stead` (default status) -- full color, the showcase
- `stead list` -- status colors, project names bold
- `stead show <id>` -- status color, verification result highlighted
- `stead session list` -- CLI headers bold, project names bold
- `stead session show` -- status indicators, timeline structure
- `stead run` -- progress indicators, final pass/fail result

### Interactive Mode?

**No TUI for MVP.** A TUI (like lazygit) adds significant implementation complexity and creates a third interface to maintain alongside CLI and SwiftUI. The CLI is for quick glances and scripting. The Control Room is for interactive supervision.

If a developer wants to watch agent progress interactively, they open the Control Room (one keystroke: Cmd+Shift+S). The CLI stays simple, fast, and composable.

Revisit TUI after the Control Room ships -- if developers still want terminal-based interactivity, there's a real signal.

---

## Control Room First Launch

### Installation

```bash
brew install --cask stead-app
```

Or download `.dmg` from GitHub releases. Standard macOS app bundle.

### What Happens on First Launch

1. **Menu bar icon appears.** A small monochrome circle -- the radar blip. No dock icon (it's a menu bar utility, not a window app). This is the Control Room's permanent home.

2. **Click the icon.** The popover opens:

```
  All clear.

  No contracts yet.
  Run stead run from the CLI to get started.

  ─────────────────────────
  Sessions detected: 4 Claude Code, 1 Codex CLI
  Open Control Room    Cmd+Shift+S
```

Even with no contracts, the app immediately shows value: it found your sessions. This validates that stead is connected to your actual work, not an empty shell waiting to be configured.

3. **Open the full window** (click "Open Control Room" or Cmd+Shift+S). First-launch view:

```
  ┌──────────────────────────────────────────────────────────────┐
  │                                                              │
  │  stead                                                       │
  │                                                              │
  │  No contracts need attention.                                │
  │                                                              │
  │  stead is watching for agent sessions and contracts.         │
  │  Create your first contract from the terminal:               │
  │                                                              │
  │    stead run "fix the bug" --verify "cargo test"             │
  │                                                              │
  │  ──────────────────────────────────────────────────────────  │
  │                                                              │
  │  Sessions (5 found)                                          │
  │    Claude Code (4)  ·  Codex CLI (1)                         │
  │                                                              │
  └──────────────────────────────────────────────────────────────┘
```

The empty state is intentionally calm. "No contracts need attention" is a positive statement -- things are fine, nothing is broken, you don't need to do anything. The session count proves the app is alive and connected.

### How It Finds Data

**Zero configuration.** The Control Room and CLI share stead-core, which knows where to look:

- Contracts: `~/.stead/stead.db` (SQLite canonical runtime storage)
- Claude Code sessions: `~/.claude/projects/` (standard Claude Code location)
- Codex CLI sessions: `~/.codex/sessions/` (standard Codex location)
- OpenCode sessions: `~/.local/share/opencode/storage/` (standard OpenCode location)

No "point to your projects" step. No directory picker. No `.steadrc` file. The app reads what already exists on disk.

If a developer uses a non-standard location for any of these, a Settings panel (gear icon) allows adding custom paths. But the defaults cover 99% of installations.

### The "I Already Have Sessions" Surprise

Most developers installing stead already have Claude Code running. The app immediately shows their existing sessions. This is the "it already knows about me" moment -- the tool felt anticipatory instead of requiring setup.

---

## Integration Points

### VS Code Extension

**Deferred.** Not in MVP. Rationale:

The Control Room is the supervision layer. VS Code is where you write code. These are different activities. An extension that shows stead status in VS Code's sidebar competes with the Control Room for attention, fragmenting the very thing stead is meant to unify.

If we build one later, it would be minimal: a status bar item showing the count of items needing attention (mirroring the menu bar icon), clicking it opens the Control Room. Not a full dashboard inside VS Code.

### Terminal Integration (shell plugin)

**One small hook, high impact.** A shell integration that adds stead awareness to the prompt:

```bash
# In .zshrc (or auto-installed via `stead shell-init zsh`)
eval "$(stead shell-init zsh)"
```

What this adds:

1. **Prompt segment** -- A subtle indicator in the prompt when inside a project with active contracts:

   ```
   ~/repos/picalyze (main) [stead: 1 running, 1 passed]
   $
   ```

   This is optional and off by default. Developers who want it opt in. It uses the Starship custom command protocol if Starship is detected, or a bare `precmd` hook for vanilla zsh/bash.

2. **Completions** -- Tab completion for `stead` commands, contract IDs, session IDs. Generated at install time via clap's built-in completion generation.

### Alfred / Raycast Quick Actions

**Raycast extension (high priority, post-MVP).** Raycast is the power-user launcher on macOS. A stead extension would provide:

- `stead` keyword triggers a list of items needing attention
- Select an item to open it in the Control Room
- `stead run` keyword opens a quick contract creation form
- `stead sessions` keyword lists recent sessions

This maps perfectly to the "quick scan" use case -- the developer hits their launcher hotkey, types "stead", sees what needs them, and acts. Under 3 seconds.

**Alfred workflow** would provide the same via Alfred's script filter pattern, reading `stead list --json`.

### `stead open` (deep linking)

A command that bridges CLI and GUI:

```bash
stead open qk4f-9m2
```

Opens the Control Room and navigates directly to that contract. Works from any terminal, any context. The GUI registers a URL scheme (`stead://contract/qk4f-9m2`) so this works even if the app isn't running.

This is the glue between terminal and GUI workflows. A developer reviewing CLI output can jump to the visual detail view instantly.

---

## The Tweetable Moments

These are the experiences that make developers share stead:

1. **"I typed `stead` and it showed me every AI agent session I had running. I didn't configure anything."** -- The zero-config discovery.

2. **"My agent finished and instead of hunting through terminal tabs, I glanced at the menu bar, clicked once, and had the full diff ready."** -- Context restoration in action.

3. **"I defined a contract with a verify command. The agent worked, the tests ran automatically, and I got a pass/fail. No babysitting."** -- The contract model.

4. **"The Control Room was just... calm. Nothing screaming at me. When something needed me, it surfaced. Otherwise, silence."** -- Attention respect.

5. **"`stead list` and the Control Room show the same data the same way. I didn't have to learn two tools."** -- Unified mental model.

---

## What We Don't Do

- **No account creation.** Everything is local. No sign-up flow, no cloud sync, no telemetry opt-in. Install, use.
- **No tutorial wizard.** No "Welcome to stead! Let's walk through..." The product teaches through output, not instructions.
- **No mandatory configuration.** Every feature works out of the box. Settings exist for customization, not requirements.
- **No "getting started" that takes more than 60 seconds.** If you need a tutorial to use stead, stead failed.

---

## Progression: The Natural Adoption Path

```
Day 1:   brew install stead && stead
         "Oh wow, I can see all my sessions."

Day 1:   stead run "fix the bug" --verify "cargo test"
         "Wait, it runs the tests automatically?"

Day 2:   brew install --cask stead-app
         "This menu bar thing is actually nice."

Day 3:   *ding* -> glance at menu bar -> one click -> full context
         "I didn't lose my train of thought."

Week 1:  stead becomes muscle memory
         "How did I work without this?"

Week 2:  Tweets about it
```

The product doesn't ask for commitment upfront. Each step earns the next step. The CLI is the handshake. Contracts are the hook. The Control Room is the home.
