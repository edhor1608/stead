# GitHub Research: Workflow Engines & Agent Orchestration

Research conducted 2026-02-02 for the stead project.

**Goal**: Find existing solutions for workflow engines, task orchestration, and agent coordination that could inform stead's "operating environment for agent-driven development" with contract-based execution.

---

## Tier 1: Most Relevant to Stead

### Restate
**Repo**: https://github.com/restatedev/restate

**What it does**: Low-latency durable execution engine for workflows, event-driven handlers, and stateful orchestration. Single binary, no separate infrastructure. TypeScript, Go, Python, Rust support.

**Key insights**:
- "Virtual state machines" concept - machines wake up, react to events, sleep, store state durably across failures
- XState integration - they built a bridge to run XState actors on Restate's durable event loop
- Each state transition is a separate invocation, saved durably
- K/V state attached to requests, written back on completion

**What we could use**:
- Their approach to making state machines persistent without complex infrastructure
- The "wake/react/sleep" pattern for long-running processes
- How they handle exactly-once semantics

**Limitations for stead**:
- Server-centric - requires running their binary
- Not designed for local-first/desktop scenarios
- Overkill for simple agent task coordination

---

### DBOS Transact
**Repo**: https://github.com/dbos-inc/dbos-transact-ts

**What it does**: Lightweight durable workflows built on Postgres. Just a library - no separate orchestrator. Decorate functions to make them durable.

**Key insights**:
- "Add durable workflows in just a few lines of code" - minimal ceremony
- Uses Postgres as the durability layer (most apps already have it)
- Workflow management via CLI, web UI, or programmatic API
- Durable queues that integrate with workflows

**What we could use**:
- The "just a library" philosophy - no new infrastructure
- Using existing Postgres for state (or SQLite for local)
- Their API design for minimal boilerplate

**Limitations for stead**:
- Requires Postgres (though SQLite equivalent possible)
- Designed for backend services, not desktop/local apps
- No agent-specific primitives

---

### XState v5
**Repo**: https://github.com/statelyai/xstate

**What it does**: Actor-based state management & orchestration. Zero dependencies. Works frontend and backend.

**Key insights**:
- Actors are the main abstraction, not just state machines
- Can model with promises, observables, reducers, callbacks - all orchestrated via events
- Visual tools (Stately editor) for designing state machines
- "Invoked" vs "spawned" actors - finite vs dynamic collections

**What we could use**:
- Battle-tested actor model implementation in TypeScript
- Their event-driven architecture patterns
- Potentially the entire state machine layer
- Visual debugging/design tools concept

**Limitations for stead**:
- No built-in durability (pair with Restate or build our own)
- In-memory by default - loses state on crash
- Not agent-aware

---

### Graphile Worker + graphile-saga
**Repos**:
- https://github.com/graphile/worker
- https://github.com/ben-pr-p/graphile-saga

**What it does**: High-performance Postgres job queue. graphile-saga adds saga pattern with automatic rollback.

**Key insights**:
- Under 3ms latency from schedule to execute
- Uses LISTEN/NOTIFY and SKIP LOCKED for performance
- graphile-saga: if one transaction fails, automatically executes compensating transactions
- Can run in same process or horizontally scale

**What we could use**:
- Postgres-native job queue (trigger.dev uses it internally)
- Saga pattern implementation for compensation/rollback
- The "can run embedded or scale out" architecture

**Limitations for stead**:
- Requires Postgres
- No workflow DSL - just jobs
- graphile-saga is less maintained

---

### Effect-TS
**Repo**: https://github.com/Effect-TS/effect

**What it does**: Build production-ready TypeScript applications with structured concurrency, fibers, and comprehensive error handling.

**Key insights**:
- Fibers = lightweight virtual threads managed by Effect runtime
- Structured concurrency: child fibers auto-terminate when parent terminates
- No orphan processes - "auto supervision" built in
- Composable concurrency primitives (Semaphore, Queue, Deferred)

**What we could use**:
- Structured concurrency model for agent task trees
- Fiber lifecycle management patterns
- Error handling and resource cleanup patterns

**Limitations for stead**:
- Steep learning curve - different programming paradigm
- Overkill for simple task coordination
- Not designed for durability/persistence

---

## Tier 2: Worth Understanding

### Trigger.dev
**Repo**: https://github.com/triggerdotdev/trigger.dev

**What it does**: Open-source background jobs platform. Checkpoint-resume system for long-running serverless tasks.

**Key insights**:
- Checkpoint-Resume: tasks pause, save state, resume - pay only for execution time
- Uses CRIU for container checkpointing
- Postgres + Graphile Worker under the hood
- Steps are independently retried with idempotency keys

**What we could learn**:
- Their step decomposition model
- Idempotency key patterns for exactly-once
- How they built durability on standard infrastructure

**Limitations**:
- Cloud-focused, not local-first
- Serverless model doesn't map to desktop agents

---

### Inngest
**Repo**: https://github.com/inngest/inngest

**What it does**: Event-driven workflow orchestration. Steps as building blocks, events as triggers.

**Key insights**:
- `step.run()` creates durable steps that persist across failures
- Events trigger multiple functions (fan-out pattern)
- Built-in rate limiting, batching, prioritization
- Jobs can sleep for days/weeks durably

**What we could learn**:
- Event-driven architecture for task coordination
- Their step DSL is clean and intuitive
- Durable sleep patterns

**Limitations**:
- Cloud-hosted primary (self-host possible but complex)
- Event model might be overkill for synchronous agent tasks

---

### Hatchet
**Repo**: https://github.com/hatchet-dev/hatchet

**What it does**: Background tasks platform built on Postgres. Queue + DAG orchestrator + durable execution in one.

**Key insights**:
- "General-purpose task orchestration" - queue, DAG, durable execution, or all three
- Multiple queueing strategies: FIFO, LIFO, Round Robin, Priority
- Durable events log to Postgres - resume exactly where you left off
- TypeScript, Python, Go SDKs

**What we could learn**:
- How they unified queue/DAG/durability models
- Their approach to fairness and rate limiting
- Postgres-backed durability patterns

**Limitations**:
- Server architecture - not embeddable
- Newer project, less battle-tested

---

### Windmill
**Repo**: https://github.com/windmill-labs/windmill

**What it does**: Turn scripts into webhooks, workflows, UIs. 13x faster than Airflow.

**Key insights**:
- Scripts become building blocks automatically
- Supports TypeScript, Python, Go, PHP, Bash, SQL, Rust, or any Docker image
- Git sync - develop locally, deploy via sync
- nsjail for multi-tenant security

**What we could learn**:
- Script-to-workflow transformation patterns
- Their approach to language-agnostic execution
- VS Code extension for local development

**Limitations**:
- Full platform, not embeddable
- Overkill for agent coordination
- AGPL license requires open-sourcing derivatives

---

### Temporal
**Repo**: https://github.com/temporalio/sdk-typescript

**What it does**: The OG durable execution platform. Fork of Uber's Cadence.

**Key insights**:
- Complete execution history and audit trails
- Exactly-once execution guarantees
- Mature, battle-tested at scale (Uber, Netflix, etc.)
- Strong typing with TypeScript SDK

**What we could learn**:
- Their replay/history model for durability
- How they handle long-running workflows
- Compensation/saga patterns

**Limitations**:
- Heavy infrastructure - requires running Temporal server
- Complex operational overhead
- Not suitable for embedded/local use

---

## Tier 3: Agent-Specific Projects

### CrewAI
**Repo**: https://github.com/crewAIInc/crewAI

**What it does**: Multi-agent orchestration with role-playing autonomous agents.

**Key insights**:
- Agents have roles, goals, backstories
- Hierarchical processes with manager agents
- Task delegation and validation patterns
- Built from scratch (not on LangChain)

**Relevance to stead**:
- Shows how to model agent specialization
- Task delegation patterns between agents
- Less relevant for contract-based execution model

---

### LangGraph
**Repo**: https://github.com/langchain-ai/langgraph

**What it does**: Low-level agent orchestration with graphs. Durable execution for agents.

**Key insights**:
- Graph-based workflow definition
- Human-in-the-loop primitives built in
- Checkpointing and state persistence
- Supervisor and swarm patterns available

**Relevance to stead**:
- Good patterns for agent state management
- Human-in-the-loop could map to contract verification
- Python-only (no TypeScript)

---

### Microsoft Agent Framework (AutoGen successor)
**Repo**: https://github.com/microsoft/agent-framework

**What it does**: Building, orchestrating, deploying AI agents. Merging AutoGen + Semantic Kernel.

**Key insights**:
- Graph-based workflows connecting agents and functions
- Streaming, checkpointing, time-travel capabilities
- Cross-language (.NET and Python)
- Magentic-One: state-of-art multi-agent team

**Relevance to stead**:
- Enterprise patterns for agent coordination
- Graph-based workflow model
- No TypeScript yet

---

### Claude-Flow
**Repo**: https://github.com/ruvnet/claude-flow

**What it does**: Multi-agent swarms for Claude Code. 60+ specialized agents.

**Key insights**:
- Self-learning from task execution
- Consensus patterns for fault tolerance
- Native Claude Code support via MCP
- Swarm intelligence patterns

**Relevance to stead**:
- Closest to our agent coordination problem
- Shows what's possible with Claude Code
- May be over-engineered for actual use

---

## Tier 4: Supporting Patterns

### Design-by-Contract TypeScript
**Repo**: https://github.com/JanMalch/ts-code-contracts

**What it does**: Preconditions, postconditions, invariants in TypeScript.

**Relevance**: Our "contract-based execution" could use similar patterns for task validation.

---

### BullMQ Flows
**Repo**: https://github.com/taskforcesh/bullmq

**What it does**: Redis-based job queue with parent/child dependencies.

**Key insights**:
- `FlowProducer` for hierarchical job dependencies
- `waiting-children` state for parent jobs
- Battle-tested at scale

**Relevance**: Job dependency patterns, though Redis requirement is heavy.

---

### DSPy
**Repo**: https://github.com/stanfordnlp/dspy

**What it does**: "Programming—not prompting—language models." Declarative LLM pipelines.

**Key insights**:
- Signatures define input/output behavior declaratively
- Automatic prompt optimization
- Modular, composable LLM operations

**Relevance**: Could inform how we define agent task contracts declaratively.

---

## Key Takeaways for Stead

### What exists:
1. **Durable execution** is a solved problem (Temporal, Restate, Trigger.dev, DBOS)
2. **State machines** are well-supported (XState)
3. **Job queues with dependencies** work (BullMQ, Graphile)
4. **Agent orchestration** is emerging but immature (CrewAI, LangGraph)

### What's missing:
1. **Local-first durable execution** - everything assumes servers
2. **Agent task contracts** - no one defines tasks with pre/postconditions
3. **Desktop integration** - all tools are server/cloud-focused
4. **Unified workspace model** - agents, terminals, browsers all separate

### Architecture insights:
1. **Postgres is the universal backend** - DBOS, Hatchet, Trigger.dev all use it
2. **SQLite could work locally** - same patterns, embedded
3. **XState + durability layer** - promising combination (Restate showed this)
4. **Event-driven is the pattern** - everything converges on events/messages

### What stead could uniquely provide:
1. Local-first durable execution (SQLite-backed)
2. Contract-based task definitions (pre/post conditions)
3. Desktop workspace integration (not just backend services)
4. Agent-aware primitives (not retrofitted onto job queues)

---

## Repos to Watch

| Repo | Why |
|------|-----|
| restatedev/restate | Best-in-class durable execution architecture |
| dbos-inc/dbos-transact-ts | Minimal library approach, Postgres patterns |
| statelyai/xstate | Actor model foundation |
| ben-pr-p/graphile-saga | Saga/compensation patterns |
| Effect-TS/effect | Structured concurrency patterns |

---

## Next Research

- [ ] Deep dive into Restate's XState integration
- [ ] Analyze DBOS's decorator-based API design
- [ ] Study Effect-TS fiber model for task trees
- [ ] Look at how graphile-saga implements compensation
- [ ] Explore SQLite as durability layer (vs Postgres)
