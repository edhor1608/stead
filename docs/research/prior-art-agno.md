# Prior Art: Agno Framework

**Date:** 2026-02-02
**Sources:**
- https://github.com/agno-agi/agno (framework)
- https://github.com/agno-agi/dash (example: SQL agent)
- https://docs.agno.com/

---

## What It Is

Agno is a framework for building multi-agent systems. Dash (SQL agent) is one product built on it. The interesting part is their learning and memory architecture.

---

## Core Hierarchy

```
Agent → Team → Workflow
```

- **Agents:** AI programs that reason, use tools, maintain state
- **Teams:** Multiple agents with specialized roles, shared context
- **Workflows:** Sequential caching, parallel execution, conditional branching

---

---

## Learning Architecture

**Learning Machines** capture different knowledge types:
- User profiles
- Entity memories
- Session context
- Decision logs
- Learned knowledge (transferable across users)

Configurable learning modes: always-on or agentic (decides when to learn).

---

## Memory Architecture

Multiple backends: PostgreSQL, MongoDB, Redis, SQLite

Three types:
- **User profiles** — persistent across sessions
- **Session context** — conversation continuity
- **Observations** — unstructured notes

---

## Knowledge Architecture

"Agentic RAG" with 25+ vector stores (Pinecone, Weaviate, LanceDB, etc.)

Combines:
- Semantic search
- Keyword matching
- Hybrid retrieval
- Sophisticated chunking

---

## Dash (SQL Agent Example)

Dash uses "6 Layers of Context":

1. **Table Usage** — schema and relationships
2. **Human Annotations** — business definitions
3. **Query Patterns** — validated SQL templates
4. **Institutional Knowledge** — external docs via MCP
5. **Learnings** — error patterns and fixes
6. **Runtime Context** — live schema changes

Learning loop: Query → Execute → Error? → Diagnose → Save Learning → Retry

---

## Comparison: Agno vs stead

| Aspect | Agno | stead |
|--------|------|-------|
| **What it is** | Framework to BUILD agents | Environment where agents WORK |
| **Learning storage** | DB-backed (Postgres, SQLite, etc.) | Project state itself (git) |
| **Knowledge retrieval** | RAG with vector stores | Context Generator synthesizes |
| **Learning model** | Store → Query → Apply | Transform → Inherit |
| **Human-in-loop** | Confirmations, approvals | Control Room supervision |
| **Scope** | Any agent application | Software development |
| **Primary abstraction** | Agent/Team/Workflow | Contract |

**Key insight:**

Agno and stead solve different problems:

- **Agno:** How do I BUILD an agent that learns?
- **stead:** Where do agents DO software development?

Agno could be used to build agents that work inside stead. They're potentially **complementary**, not competing.

---

## What We Can Learn

1. **Agent → Team → Workflow hierarchy.** Clean abstraction for multi-agent coordination. stead's Contract could interact with this.

2. **Learning Machine as component.** They isolated learning logic. stead's Selection Pressure is more integrated (transforms are part of contract output).

3. **Multiple memory types.** User profiles vs session context vs observations. Useful distinction.

4. **MCP integration.** They use MCP for tools and institutional knowledge. stead should be MCP-native.

5. **Multiple storage backends.** Flexibility in persistence layer. stead uses SQLite + git.

---

## Differences in Philosophy

**Agno:** Learning is a feature of the agent. The agent remembers and retrieves.

**stead:** Learning is evolution of the project. Future agents inherit a transformed environment.

**Agno:** Knowledge lives in databases, retrieved via RAG.

**stead:** Knowledge is embodied in project state — code, docs, contracts, decisions.

**Agno:** Build smarter agents.

**stead:** Build better projects that make any agent effective.

---

## Verdict

Agno is the most mature agent framework we've seen. Their learning/memory architecture is sophisticated.

But they're solving "how to build agents" — we're solving "where agents work on software."

stead's unique contribution is the paradigm: **agents transform the project, not their own memory**. The project evolves. Any agent that enters inherits that evolution.

Agno agents could work inside stead. The Contract Engine could delegate to Agno Teams. The Control Room could supervise Agno Workflows. Worth exploring as integration path.
