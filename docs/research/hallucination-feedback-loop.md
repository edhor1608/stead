# Hallucination Analysis: Feedback Loop / Learning System

**Date:** 2026-02-02
**Status:** Analysis complete

---

## The Question

The current stead model is linear:

```
Define (Contract) → Execute (Daemon) → Verify → Done
```

What's missing is learning:
- When something fails, how does the system learn to avoid that pattern?
- When something succeeds, how does that inform future approaches?
- How does the system get BETTER over time, not just run?

This is distinct from the Context Generator (which synthesizes existing knowledge). This is about ACTIVE LEARNING — the system improving itself based on outcomes.

---

## Positive

### Why a Learning System Would Help

**1. Agents repeat mistakes.**
Without feedback, an agent will hit the same failure mode in every session. It'll try the deprecated API, miss the same edge case, use the wrong pattern for this codebase. Humans accumulate intuition; agents start fresh each time.

**2. Success patterns are valuable but invisible.**
When a contract completes successfully, what made it work? Was it the approach? The context that was provided? The verification criteria? This signal currently vanishes.

**3. Project-specific knowledge compounds.**
"In this codebase, always check for null before accessing `user.profile`." This isn't in any documentation. It's learned from pain. A learning system could capture and apply this.

**4. Verification failures are rich data.**
When verification fails, you know exactly:
- What was attempted
- What was expected
- What actually happened
- Why it didn't match

This is perfect training signal if you can use it.

**5. Human corrections are gold.**
When a human intervenes — rejects output, provides guidance, fixes something — that's the highest-quality signal. The system should learn from every correction.

**6. Cost optimization.**
Learning which approaches are expensive (token-wise, time-wise) vs. efficient for this project could dramatically reduce resource usage over time.

---

## Negative

### What's Hard About This

**1. Learning systems are notoriously hard to debug.**
When something goes wrong, is it the model? The training data? The retrieval? The context window? The learning system itself? Adding a learning layer adds a whole class of inscrutable failures.

**2. Catastrophic forgetting.**
ML systems that learn continuously tend to forget old knowledge when learning new things. You'd need careful curriculum design, which is its own research problem.

**3. Feedback loops can go wrong.**
A system that learns from its own outputs can reinforce bad patterns. Early mistakes become "correct" because they look like the training data. Self-improvement can become self-destruction.

**4. What does "learning" even mean for LLMs?**
You can't retrain Claude. So "learning" means:
- Fine-tuning (not available for Claude)
- RAG/retrieval (lossy, retrieval failures)
- Prompt engineering (fragile, context-limited)
- Example selection (requires good indexing)

None of these is true learning. They're all approximations with different failure modes.

**5. Cold start problem.**
A learning system is useless until it has data. But you need the system running to generate data. This chicken-and-egg means you're worse off initially than without learning.

**6. Overfitting to Jonas.**
If the system learns from one person's workflow, it becomes Jonas-specific. That conflicts with "design for general" principle.

**7. Context Generator already does this (sort of).**
The Context Generator synthesizes relevant context from project state. Past failures are in git history. Past decisions are in the decisions log. What does a separate "learning system" add that Context Generator doesn't already provide?

**8. Over-engineering for marginal gain.**
Agents are already pretty good at avoiding repeated mistakes when given proper context. The marginal improvement from a formal learning system might not justify the complexity.

---

## The Core Tension

The Context Generator reframe was: "Don't build a memory store. Build a context generator."

But now we're asking: "How does the project state itself improve over time?"

The answer might be: **It already does.** Every decision logged, every contract completed, every verification result — these become inputs to the Context Generator. The system learns by accumulating state, and the Generator synthesizes it.

So is a "Learning System" just... more sophisticated Context Generation? Or is it something genuinely different?

---

## Radical Take

### The Obvious Approach (What You'd Build)

```
Outcome Tracker
├── Log every contract result (success/failure/rollback)
├── Capture human corrections
├── Index by similarity (embedding search)
└── Feed relevant history to future tasks

Feedback Database
├── What worked
├── What didn't
├── Why (human-labeled)
└── Queryable patterns
```

This is the "build a learning system" approach. It's what every AI memory product does. It's also what everyone is already building, and none of them work well.

### The Problem With This

**Learning is not retrieval.**

When a senior developer joins a new project, they don't search a database of "what worked before." They develop intuition through exposure. The project shapes how they think, not what they remember.

The retrieval model assumes:
1. Past experience can be atomically stored
2. Relevant experience can be retrieved at the right moment
3. Retrieved experience can be applied to new situations

Each step has failure modes. The compound failure rate is high.

### The Reframe

**Don't build a learning system. Build evolution pressure.**

In biology, learning happens two ways:
1. **Individual learning** — an organism adapts during its lifetime
2. **Evolution** — populations change across generations through selection

The AI industry is obsessed with individual learning (fine-tuning, RLHF, RAG). But agents don't have lifetimes — each session starts fresh. Individual learning doesn't apply.

What DOES apply is evolution: **variation + selection + reproduction**.

### What This Means for Stead

**Variation:** Multiple approaches can be attempted for the same contract. Agents already do this (try approach A, if it fails, try approach B). Make this explicit.

**Selection:** Verification is selection. Contracts that pass verification survive. Those that fail don't. The selection criteria are already defined — it's the verification block.

**Reproduction:** This is the key insight. Successful approaches should propagate to future contracts. Not as "memory to retrieve" but as **transformations to the project state itself**.

```
Instead of:
  Success → Store in memory → Retrieve later → Hope it applies

Do:
  Success → Transform the project → Future agents inherit transformed state
```

### Concrete Example

**Learning approach:**
Agent figures out that `npm run test` needs `NODE_ENV=test` in this project. Log this as a "lesson learned." Future agents query the lessons database, retrieve this fact, apply it.

Failure modes: Doesn't get retrieved. Retrieved but not applied. Applied incorrectly. Lesson becomes stale.

**Evolution approach:**
Agent figures out `NODE_ENV=test` is needed. It MODIFIES THE PROJECT: updates `.env.example`, adds to contract templates, fixes the test script itself. Future agents don't "remember" this — they inherit a project where the problem is already solved.

The project evolves. Knowledge becomes infrastructure.

### The Principle

**Learning isn't accumulating facts. Learning is transforming the environment.**

A senior dev doesn't remember every gotcha — they've fixed them. The codebase itself embodies their knowledge. Tests exist where bugs were found. Types exist where errors occurred. Patterns exist where chaos lived.

The project IS the memory. The question is: how do we let successful agents transform it?

---

## The Seventh Pillar: Selection Pressure

**What:** Not a learning system. A mechanism that lets successful contract executions transform the project state.

**The Reframe:**

| Traditional Approach | Stead Approach |
|---------------------|----------------|
| Log outcomes to database | Outcomes transform project state |
| Retrieve relevant lessons | Inherit an evolved project |
| Apply retrieved knowledge | Start in a better environment |
| "Remember" what worked | Embody what worked |

**How It Works:**

```
Contract Execution
├── Executes task
├── Verification passes
└── Proposes transformations (optional)

Transformation Types:
├── Documentation updates ("add gotcha to CLAUDE.md")
├── Contract template improvements
├── Verification criteria refinements
├── Environment configuration
├── Code patterns to codify
└── Decisions to log

Selection:
├── Human approves/rejects transformations
├── Approved → applied to project
├── Rejected → not applied
└── Either way, work is done
```

**Key Properties:**

1. **No separate learning system.** The project IS the learning. Every improvement is a real change to real files.

2. **Human selection.** Agents propose transformations; humans approve. This prevents runaway feedback loops and bad pattern reinforcement.

3. **Transparent.** All "learning" is visible in git. You can see what the system learned and why.

4. **Reversible.** Don't like what it learned? Revert the commit.

5. **Composable.** Works with existing pillars. Transformations compile to git (Transformation Layer). Context Generator synthesizes from improved project state.

6. **No cold start.** The project already exists. Learning starts immediately.

**What Changes:**

Contracts get an optional `transformations` output block:

```
contract {
  id: "fix-auth-bug"

  output: {
    # Primary deliverable
    code_changes: [...]
  }

  transformations: {
    # Optional: proposed improvements to project state
    docs: [
      { file: "CLAUDE.md", section: "gotchas", add: "Auth tokens expire after 1hr" }
    ]
    contracts: [
      { template: "api-work", add_verification: "check token expiry" }
    ]
    decisions: [
      { context: "Auth debugging", decision: "Always check token expiry first" }
    ]
  }

  verification: {
    # Verify primary output
    # Transformations verified by human selection
  }
}
```

**Selection Flow:**

```
Agent completes contract
        │
        ▼
Primary output verified (automated)
        │
        ▼
Transformations proposed? ─── No ──→ Done
        │
        Yes
        ▼
Human reviews transformations (Control Room)
        │
        ├── Approve → Apply to project, log in git
        │
        └── Reject → Discard, log rejection reason
        │
        ▼
      Done
```

**Why This Works:**

1. **Agents already do this implicitly.** Good agents update docs, fix root causes, leave the codebase better. This makes it explicit and verifiable.

2. **Humans already approve work.** Adding transformation approval to the review flow is minimal overhead.

3. **Git already tracks changes.** Transformations are just more commits.

4. **Context Generator already synthesizes.** Improved project state = richer context for future agents.

5. **No ML required.** No embeddings, no retrieval, no training. Just structured changes to files.

---

## Why Not a Seventh Pillar?

After this analysis, I'm not sure this IS a seventh pillar. Here's why:

**It might just be a feature of Contract Engine + Control Room.**

- Contract Engine already has outputs
- Adding optional transformation proposals is a schema extension
- Control Room already handles human decisions
- Adding transformation approval is a UI extension
- Transformation Layer already compiles changes to git

The "Selection Pressure" concept is powerful, but it might be a PATTERN that emerges from the existing six pillars, not a pillar of its own.

**Counter-argument:** The six pillars each represent a distinct category of infrastructure:
1. How work is defined (Contract Engine)
2. How work runs (Execution Daemon)
3. Identity isolation (Session Proxy)
4. Human supervision (Control Room)
5. Code changes (Transformation Layer)
6. Knowledge synthesis (Context Generator)

"How the system improves" could be a seventh category. But it could also be an emergent property of the others working together.

---

## Recommendation

**Don't add a seventh pillar.** The existing architecture already supports evolution:

1. Agents can propose transformations as part of contract output
2. Control Room shows transformations for human selection
3. Approved transformations become git commits
4. Context Generator synthesizes from improved project state

What's needed is:
- Schema support for transformation proposals in contracts
- UI for reviewing/approving transformations in Control Room
- Guidelines for what kinds of transformations are appropriate

This is a **feature set**, not a new pillar. The architecture doesn't change. The philosophy doesn't change. Agents transform the project through their work, and humans select which transformations persist.

**The insight is still valuable:** Don't build a learning database. Let successful work transform the project itself. But this insight is an APPLICATION of the existing pillars, not a new one.

---

## Summary

| Aspect | Traditional Learning | Selection Pressure |
|--------|---------------------|-------------------|
| Storage | Separate memory system | Project state itself |
| Retrieval | Query relevant history | Inherit evolved environment |
| Application | Apply retrieved lessons | Start better, not remember more |
| Visibility | Opaque embedding space | Git commits, reviewable |
| Failure mode | Wrong retrieval | Human rejects transformation |
| Maintenance | Schema, indices, cleanup | Just files in git |

**The radical take:** Agents don't need to remember what worked. They need to leave the project better than they found it. Memory is embodied in infrastructure, not stored in databases.

**Implementation:** This is a feature of Contract Engine + Control Room, not a new pillar. Add transformation proposals to contract outputs, add transformation approval to Control Room UI, let Transformation Layer compile to git.

**What stead becomes:** Not just an execution environment, but a system under selection pressure. Good patterns propagate. Bad patterns die. The project evolves toward better outcomes.
