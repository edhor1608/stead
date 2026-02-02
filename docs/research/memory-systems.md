# Memory Systems Research

Research into existing memory/context solutions for agent-driven development.

---

## 1. supermemoryai/claude-supermemory

**Repo:** https://github.com/supermemoryai/claude-supermemory
**Stars:** 1,922 | **Language:** JavaScript | **Updated:** 2026-01-31

### What It Does

A Claude Code plugin that gives persistent memory across sessions using Supermemory's cloud API.

**Core Features:**
- Context injection on session start (memories fetched and injected as XML)
- Automatic conversation capture (tool calls and turns stored)
- Codebase indexing (architecture, patterns, conventions)
- Search skill for querying past sessions

### How It Handles Memory/Context

**Architecture:**
```
Session Start → API fetch → Format memories → Inject as <supermemory-context>
Tool Use → Capture turn → POST to Supermemory API
```

**Data Model:**
- User profile (persistent preferences, patterns)
- Recent context (what you've worked on)
- Codebase index (project architecture)

**Injection Format:**
```xml
<supermemory-context>
## User Profile (Persistent)
- Prefers TypeScript over JavaScript
- Uses Bun as package manager

## Recent Context
- Working on authentication flow
</supermemory-context>
```

### Relevance to Stead

| Aspect | Relevance | Notes |
|--------|-----------|-------|
| Memory model | Low | Cloud-based RAG retrieval — exactly what NORTH_STAR warns against |
| Plugin architecture | Medium | Shows how to hook into Claude Code sessions |
| Context injection | High | XML-wrapped context injection pattern is useful |
| Codebase indexing | Medium | Could inform context generator approach |

**Key Takeaways:**
- Plugin hooks: `context-hook.js` (session start), `observation-hook.js` (tool use), `summary-hook.js` (session end)
- Context is injected via `hookSpecificOutput.additionalContext`
- Memory is treated as retrieval problem (query → fetch → inject) — NORTH_STAR explicitly rejects this

**Verdict:** Useful for understanding Claude Code plugin architecture. Memory model is antithetical to stead's "context generator" approach.

---

## 2. supermemoryai/supermemory

**Repo:** https://github.com/supermemoryai/supermemory
**Stars:** 16,161 | **Language:** TypeScript | **Updated:** 2026-02-02

### What It Does

"AI second brain" — a hosted memory API with web app, browser extension, and MCP integration.

**Core Features:**
- Add memories from any content (URLs, PDFs, text)
- Chat with memories via natural language
- MCP integration for AI tools (Claude, Cursor)
- Integrations: Notion, Google Drive, OneDrive
- Browser extension for saving content
- Memory graph visualization

### How It Handles Memory/Context

**Memory Types:**
- Documents (URLs, files, notes)
- Memories (extracted/versioned from documents)
- Workspaces (organizational grouping)

**Memory Graph:**
- Documents → Memories with version chains
- Similarity edges between documents
- React component for visualization

**Architecture:**
```
Content → Embedding → Storage → Query → Retrieve
        ↑                       ↓
    MCP Tools              AI Context
```

### Relevance to Stead

| Aspect | Relevance | Notes |
|--------|-----------|-------|
| Memory API | Low | Still retrieval-based, cloud-hosted |
| Memory graph viz | Medium | Could inform control room UI for showing project knowledge |
| MCP pattern | High | Shows clean MCP tool design for memory operations |
| Document ingestion | Low | Stead's sources are code/contracts/git, not arbitrary documents |

**Key Takeaways:**
- Well-designed MCP tools: `add_memory`, `search_memories`, `get_context`
- Memory graph shows relationships — could inform how context generator surfaces connections
- Heavy cloud dependency — opposite of what stead needs

**Verdict:** Good MCP patterns. Memory model is external/retrieval-based — not applicable to stead's local context generation.

---

## 3. JasonDocton/lucid-memory

**Repo:** https://github.com/JasonDocton/lucid-memory
**Stars:** 61 | **Language:** Rust + Bun | **Updated:** 2026-02-01

### What It Does

Local, cognitive memory system for Claude Code. Implements ACT-R cognitive architecture for memory retrieval.

**Core Features:**
- 2.7ms retrieval, 743k memories/second
- 100% local (no cloud, no API costs)
- Cognitive model (not just vector similarity)
- Visual memory (images/videos)
- Location intuitions (spatial memory for codebase)
- Coming: Procedural memory (learned workflows)

### How It Handles Memory/Context

**Cognitive Model (ACT-R + MINERVA 2):**

Three activation sources combine to determine what surfaces:

1. **Base-level activation** (recency/frequency):
   ```
   B(m) = ln[Σ(t_k)^(-d)]
   ```
   Recent and frequent access = higher activation

2. **Probe-trace similarity** (relevance):
   ```
   A(i) = S(i)³
   ```
   Cubed similarity — weak matches contribute minimally, strong matches dominate

3. **Spreading activation** (association):
   ```
   A_j = Σ(W_i/n_i) × S_ij
   ```
   Related memories amplify each other

**Retrieval Pipeline:**
```
Query → Embed → Similarity (batch) → Nonlinear activation → Base-level → Spreading → Rank
```

**Key Data Structures:**
```rust
pub struct RetrievalCandidate {
    pub index: usize,
    pub base_level: f64,        // from access history
    pub probe_activation: f64,   // from similarity³
    pub spreading: f64,          // from associations
    pub emotional_weight: f64,   // salience multiplier
    pub total_activation: f64,   // combined
    pub probability: f64,        // retrieval probability
    pub latency_ms: f64,
}
```

**Location Intuitions:**
- Familiarity grows asymptotically: `familiarity = 1 - 1/(1 + 0.1n)`
- Context bound to location (what you were doing when you touched a file)
- Files worked on together form associative networks
- Unused knowledge fades (but well-known files have "sticky" floors)

### Relevance to Stead

| Aspect | Relevance | Notes |
|--------|-----------|-------|
| Cognitive model | HIGH | Memory as reconstruction, not retrieval — aligns with context generator |
| Local-first | HIGH | No cloud dependency, fast, cost-free |
| Location intuitions | HIGH | Directly applicable to codebase navigation |
| Spreading activation | HIGH | Decisions influencing related areas — "decisions become constraints" |
| Emotional weight | Medium | Could map to contract priority/urgency |
| Procedural memory | HIGH | Learning workflows — exactly what stead needs |

**Key Insights:**

1. **Memory isn't retrieval, it's reconstruction:**
   > "Rather than storing static records, memories evolve over time shaped by context, with details fade, essence persists"

   This mirrors NORTH_STAR's "memory is alive — a lens through which agents see the project, not a log they search"

2. **Activation over similarity:**
   Standard RAG: highest similarity wins.
   Lucid: activation combines recency, frequency, relevance, AND associations.

3. **Spreading activation = decisions as constraints:**
   When a memory activates, related memories activate too. A decision about auth architecture spreads to activate related auth code, past auth discussions, etc. This is EXACTLY "decisions become constraints that shape behavior."

4. **Location intuitions = embodied memory:**
   Claude learns codebase navigation through exposure, not explicit indexing. After working in a project, it "knows" where things are. This is "memory embodied in the agent's starting state."

**Verdict:** Most aligned with stead's context generator vision. Key concepts directly applicable.

---

## Summary: What This Means for Stead

### What NOT to do (cloud RAG approach)

Both Supermemory projects treat memory as:
- External database to query
- Retrieval problem (store facts, search facts)
- Cloud dependency with API costs
- Separate system to maintain

This is exactly what NORTH_STAR rejects.

### What TO consider (cognitive approach)

Lucid Memory demonstrates:

1. **Context generation vs retrieval:**
   - Don't store everything and search
   - Compute what's relevant NOW based on activation

2. **Activation model for context generator:**
   - Recency: Recent decisions matter more
   - Frequency: Oft-touched code is familiar
   - Associations: Related concepts surface together
   - Spreading: Decisions propagate through connections

3. **Local-first, zero-cost:**
   - Embeddings computed once
   - Retrieval is local vector math
   - No network round-trips

4. **Memory as lens, not log:**
   - Location intuitions = learned codebase navigation
   - Procedural memory = learned workflows
   - Memory shapes starting state, not query results

### Concrete Ideas for Context Generator

1. **Activation-weighted context assembly:**
   Instead of RAG "top-k similar chunks," use activation to weight what enters context:
   ```
   context = synthesize(
     recent_contracts,      // base-level: recency
     frequent_files,        // base-level: frequency
     related_decisions,     // spreading: associations
     current_task_relevant  // probe: similarity
   )
   ```

2. **Decisions as activation weights:**
   When a decision is made, boost activation of related code/contracts. "We decided to use JWT for auth" → auth code, session contracts, security decisions all get activation boost.

3. **Familiarity curves:**
   Track which parts of codebase agents have worked with. Familiar areas need less context. Novel areas need more setup.

4. **Spreading for constraint propagation:**
   Decision about database schema → spreads to API contracts → spreads to validation code. Agent starting work on API automatically sees relevant schema decisions.

---

## References

- [Supermemory Claude Plugin](https://github.com/supermemoryai/claude-supermemory)
- [Supermemory Main Repo](https://github.com/supermemoryai/supermemory)
- [Lucid Memory](https://github.com/JasonDocton/lucid-memory)
- [ACT-R Cognitive Architecture](http://act-r.psy.cmu.edu/)
- [MINERVA 2 Model](https://en.wikipedia.org/wiki/MINERVA_2)
