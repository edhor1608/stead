# Contract Engine Exploration

Date: 2026-02-02

## Overview

The Contract Engine is the foundational component of stead. It replaces traditional task tracking with a transaction-like model designed for agent execution rather than human coordination.

**Core insight**: Agents don't need tasks — they need contracts. A contract specifies:
- What the agent receives (input)
- What the agent must produce (output)
- How to verify completion (verification)
- What to do on failure (rollback)

This is closer to database transactions and saga patterns than Jira tickets.

---

## 1. Contract Schema

### MVP Schema

```typescript
interface Contract {
  // Identity
  id: string;                    // Unique identifier (ULID for sortability)
  version: number;               // Optimistic concurrency control

  // Lifecycle
  status: ContractStatus;
  createdAt: Date;
  updatedAt: Date;
  claimedAt?: Date;
  completedAt?: Date;

  // Ownership
  owner?: AgentId;               // Who claimed it
  project?: ProjectId;           // Which project it belongs to

  // Specification
  input: ContractInput;
  output: ContractOutput;
  verification: VerificationSpec;
  rollback?: RollbackSpec;

  // Dependencies
  blockedBy: ContractId[];       // Must complete before this can start
  blocks: ContractId[];          // Contracts waiting on this one

  // Resources
  resources: ResourceClaim[];    // What it needs access to

  // Metadata
  metadata: Record<string, unknown>;
}

interface ContractInput {
  // Typed data the agent receives
  schema: JSONSchema;            // What shape the input must be
  data: unknown;                 // The actual input data

  // References to external resources
  files?: FilePath[];            // Files to read
  context?: ContextRef[];        // Other contracts, docs, etc.
}

interface ContractOutput {
  // What the agent must produce
  schema: JSONSchema;            // What shape the output must be

  // Artifact specifications
  artifacts?: ArtifactSpec[];    // Files, commits, deployments, etc.
}

interface VerificationSpec {
  type: 'automated' | 'human' | 'hybrid';

  // Automated checks
  checks?: VerificationCheck[];

  // Human review requirements
  humanReview?: {
    required: boolean;
    reviewer?: UserId;
    criteria: string[];          // What the human should verify
  };

  // Timeout
  timeout?: Duration;            // How long verification can take
}

interface VerificationCheck {
  name: string;
  type: 'test' | 'lint' | 'typecheck' | 'custom';
  command?: string;              // Shell command to run
  script?: string;               // Inline script
  expectedExitCode?: number;
  expectedOutput?: string | RegExp;
}

interface RollbackSpec {
  // What to undo if the contract fails
  type: 'none' | 'automatic' | 'manual' | 'compensating';

  // For automatic rollback
  commands?: string[];           // Commands to run

  // For compensating transactions
  compensatingContract?: ContractId;

  // For manual rollback
  instructions?: string;         // Human instructions
}

interface ResourceClaim {
  type: 'file' | 'directory' | 'port' | 'service' | 'api';
  path?: string;                 // For file/directory
  port?: number;                 // For port
  service?: string;              // For service/api
  access: 'read' | 'write' | 'exclusive';
}

type ContractStatus =
  | 'pending'      // Created, waiting for dependencies
  | 'ready'        // Dependencies met, can be claimed
  | 'claimed'      // Agent has claimed it
  | 'executing'    // Work in progress
  | 'verifying'    // Verification in progress
  | 'completed'    // Successfully done
  | 'failed'       // Failed, may need rollback
  | 'rolling_back' // Rollback in progress
  | 'rolled_back'  // Rollback complete
  | 'cancelled';   // Manually cancelled
```

### Full Vision Schema (additions)

```typescript
interface Contract {
  // ... MVP fields ...

  // Hierarchy
  parent?: ContractId;           // For sub-contracts
  children?: ContractId[];       // Sub-contracts spawned during execution

  // Execution constraints
  constraints: {
    maxRetries?: number;
    retryDelay?: Duration;
    deadline?: Date;             // Hard deadline
    priority?: number;           // 0-100, higher = more urgent
    estimatedDuration?: Duration;
  };

  // Observability
  traces: ExecutionTrace[];      // What happened during execution
  metrics: ContractMetrics;      // Performance data

  // Cost tracking
  cost: {
    tokens?: number;             // LLM tokens used
    compute?: Duration;          // CPU time
    estimated?: Money;           // Estimated cost in $
    actual?: Money;              // Actual cost
  };
}

interface ExecutionTrace {
  timestamp: Date;
  type: 'log' | 'state_change' | 'checkpoint' | 'error';
  message: string;
  data?: unknown;
}
```

### Design Rationale

**Why JSON Schema for input/output?**
- Standard, well-tooled format
- Agents can validate their own work
- Enables automated verification
- Human-readable but machine-processable

**Why separate verification from output?**
- Output says *what* to produce
- Verification says *how to check* it was produced correctly
- Some verification needs humans, some is automated
- Verification can be more complex than output shape (e.g., "code compiles AND passes tests AND doesn't break existing functionality")

**Why explicit rollback?**
- Not all work is reversible
- Different failures need different responses
- Compensating transactions (saga pattern) may be needed
- Humans need clear instructions when manual rollback is required

---

## 2. State Machine

### Lifecycle States

```
                    ┌──────────────┐
                    │   pending    │
                    └──────┬───────┘
                           │ dependencies_met
                           ▼
                    ┌──────────────┐
          ┌────────│    ready     │────────┐
          │        └──────┬───────┘        │
          │               │ claim          │ cancel
          │               ▼                ▼
          │        ┌──────────────┐  ┌──────────────┐
          │        │   claimed    │  │  cancelled   │
          │        └──────┬───────┘  └──────────────┘
          │               │ start
          │               ▼
          │        ┌──────────────┐
          │        │  executing   │◄─────┐
          │        └──────┬───────┘      │
          │               │              │ retry
          │    ┌──────────┼──────────┐   │
          │    │          │          │   │
          │    │ complete │ fail     │   │
          │    ▼          ▼          │   │
          │ ┌────────┐ ┌────────┐    │   │
          │ │verify- │ │ failed │────┼───┘
          │ │  ing   │ └───┬────┘    │
          │ └───┬────┘     │         │
          │     │          │ rollback│
          │     │          ▼         │
          │     │   ┌────────────┐   │
          │     │   │rolling_back│   │
          │     │   └─────┬──────┘   │
          │     │         │          │
          │     │         ▼          │
          │     │   ┌────────────┐   │
          │     │   │rolled_back │   │
          │     │   └────────────┘   │
          │     │
          │     │ verify_pass / verify_fail
          │     ▼
          │ ┌──────────┐
          │ │completed │  (verify_pass)
          │ └──────────┘
          │     or
          │ ┌──────────┐
          └►│  failed  │  (verify_fail)
            └──────────┘
```

### State Transitions

```typescript
type ContractEvent =
  | { type: 'DEPENDENCIES_MET' }
  | { type: 'CLAIM'; agentId: AgentId }
  | { type: 'UNCLAIM' }
  | { type: 'START' }
  | { type: 'COMPLETE'; output: unknown }
  | { type: 'FAIL'; error: Error }
  | { type: 'VERIFY_PASS' }
  | { type: 'VERIFY_FAIL'; reason: string }
  | { type: 'RETRY' }
  | { type: 'ROLLBACK' }
  | { type: 'ROLLBACK_COMPLETE' }
  | { type: 'CANCEL' };

const transitions: Record<ContractStatus, Partial<Record<ContractEvent['type'], ContractStatus>>> = {
  pending: {
    DEPENDENCIES_MET: 'ready',
    CANCEL: 'cancelled',
  },
  ready: {
    CLAIM: 'claimed',
    CANCEL: 'cancelled',
  },
  claimed: {
    START: 'executing',
    UNCLAIM: 'ready',
    CANCEL: 'cancelled',
  },
  executing: {
    COMPLETE: 'verifying',
    FAIL: 'failed',
    CANCEL: 'cancelled',
  },
  verifying: {
    VERIFY_PASS: 'completed',
    VERIFY_FAIL: 'failed',
  },
  failed: {
    RETRY: 'executing',
    ROLLBACK: 'rolling_back',
    CANCEL: 'cancelled',
  },
  rolling_back: {
    ROLLBACK_COMPLETE: 'rolled_back',
    FAIL: 'failed', // Rollback itself failed
  },
  rolled_back: {},   // Terminal
  completed: {},     // Terminal
  cancelled: {},     // Terminal
};
```

### Guards and Actions

```typescript
// Guards: conditions that must be true for transition
const guards = {
  canClaim: (contract: Contract, agentId: AgentId) => {
    return !contract.owner && contract.status === 'ready';
  },

  canRetry: (contract: Contract) => {
    const maxRetries = contract.constraints?.maxRetries ?? 3;
    return contract.retryCount < maxRetries;
  },

  dependenciesMet: (contract: Contract, allContracts: Map<ContractId, Contract>) => {
    return contract.blockedBy.every(id => {
      const dep = allContracts.get(id);
      return dep?.status === 'completed';
    });
  },
};

// Actions: side effects on transition
const actions = {
  onClaim: (contract: Contract, agentId: AgentId) => {
    contract.owner = agentId;
    contract.claimedAt = new Date();
  },

  onComplete: (contract: Contract, output: unknown) => {
    contract.completedAt = new Date();
    // Trigger verification
    // Notify blocked contracts
  },

  onFail: (contract: Contract, error: Error) => {
    contract.traces.push({
      timestamp: new Date(),
      type: 'error',
      message: error.message,
      data: error,
    });
  },
};
```

### Timeouts and Heartbeats

```typescript
interface ExecutionMonitor {
  // Claimed contracts must start within timeout
  claimTimeout: Duration;        // Default: 5 minutes

  // Executing contracts must send heartbeats
  heartbeatInterval: Duration;   // Default: 30 seconds
  heartbeatTimeout: Duration;    // Default: 2 minutes

  // If heartbeat missed, contract becomes "stale"
  onStale: (contract: Contract) => void;
  // Options:
  // 1. Release back to 'ready' for another agent
  // 2. Mark as 'failed'
  // 3. Notify human
}
```

---

## 3. Dependencies Between Contracts

### Dependency Types

```typescript
interface Dependency {
  type: 'blocks' | 'requires_output' | 'resource_conflict';
  contractId: ContractId;
}

// blocks: B cannot start until A completes
// requires_output: B needs A's output as input
// resource_conflict: B needs a resource A is using exclusively
```

### Dependency Resolution

```typescript
class DependencyGraph {
  private contracts: Map<ContractId, Contract>;
  private edges: Map<ContractId, Set<ContractId>>;  // blockedBy -> blocks

  // Check if contract can transition to 'ready'
  canBeReady(contractId: ContractId): boolean {
    const contract = this.contracts.get(contractId);
    if (!contract || contract.status !== 'pending') return false;

    return contract.blockedBy.every(depId => {
      const dep = this.contracts.get(depId);
      return dep?.status === 'completed';
    });
  }

  // Get all contracts that become ready when this one completes
  getUnblockedContracts(completedId: ContractId): ContractId[] {
    const blocked = this.edges.get(completedId) ?? new Set();
    return [...blocked].filter(id => this.canBeReady(id));
  }

  // Detect cycles (invalid state)
  detectCycles(): ContractId[][] {
    // Tarjan's algorithm or similar
    // Returns array of cycles (each cycle is array of contract IDs)
  }

  // Topological sort for execution planning
  getExecutionOrder(): ContractId[] {
    // Kahn's algorithm
    // Returns contracts in valid execution order
  }
}
```

### Output Passing

When contract B requires output from contract A:

```typescript
interface OutputPassingContract extends Contract {
  input: {
    // Reference to another contract's output
    from: {
      contractId: ContractId;
      path?: string;  // JSONPath to specific field
    };
  };
}

// When A completes, its output is validated and stored
// When B starts, A's output is injected as B's input
// Type compatibility is checked at contract creation time
```

### Cascading Failures

```typescript
// When a contract fails, what happens to contracts that depend on it?
type FailurePropagation =
  | 'block'     // Dependent contracts stay 'pending' forever
  | 'fail'      // Dependent contracts are marked 'failed'
  | 'notify'    // Dependent contracts get an event, decide themselves
  | 'retry';    // Retry the failed contract automatically

interface FailurePolicy {
  propagation: FailurePropagation;
  maxCascadeDepth?: number;  // Limit how far failures propagate
  notifyHuman?: boolean;      // Alert human on cascading failure
}
```

---

## 4. Agent Claiming and Execution

### Claiming Protocol

```typescript
// Agent claims a contract
interface ClaimRequest {
  agentId: AgentId;
  contractId: ContractId;
  capabilities?: string[];  // What the agent can do
}

interface ClaimResponse {
  success: boolean;
  contract?: Contract;
  error?: 'already_claimed' | 'not_ready' | 'capability_mismatch';
}

// Optimistic locking to prevent race conditions
async function claimContract(req: ClaimRequest): Promise<ClaimResponse> {
  const contract = await db.contracts.findById(req.contractId);

  if (contract.status !== 'ready') {
    return { success: false, error: 'not_ready' };
  }

  if (contract.owner) {
    return { success: false, error: 'already_claimed' };
  }

  // Atomic update with version check
  const updated = await db.contracts.updateOne(
    { id: req.contractId, version: contract.version },
    {
      status: 'claimed',
      owner: req.agentId,
      claimedAt: new Date(),
      version: contract.version + 1,
    }
  );

  if (updated.modifiedCount === 0) {
    // Race condition - another agent claimed first
    return { success: false, error: 'already_claimed' };
  }

  return { success: true, contract: await db.contracts.findById(req.contractId) };
}
```

### Work Claiming Strategies

```typescript
// How agents find work
type ClaimStrategy =
  | 'polling'        // Agent periodically checks for ready contracts
  | 'push'           // Engine pushes contracts to agents
  | 'work_stealing'; // Agents steal from each other's queues

interface PollingConfig {
  interval: Duration;
  filter?: {
    project?: ProjectId;
    priority?: { min: number };
    estimatedDuration?: { max: Duration };
  };
}

interface PushConfig {
  agentId: AgentId;
  capabilities: string[];
  maxConcurrent: number;
}
```

### Execution Lifecycle

```typescript
// What an agent does during execution
interface AgentExecutor {
  // Start executing the contract
  start(contract: Contract): Promise<void>;

  // Send progress updates
  heartbeat(contractId: ContractId, progress?: Progress): Promise<void>;

  // Report intermediate results (for long-running contracts)
  checkpoint(contractId: ContractId, data: unknown): Promise<void>;

  // Submit final output
  complete(contractId: ContractId, output: unknown): Promise<void>;

  // Report failure
  fail(contractId: ContractId, error: Error): Promise<void>;

  // Spawn sub-contracts
  spawn(parentId: ContractId, contracts: ContractSpec[]): Promise<ContractId[]>;
}

interface Progress {
  percent?: number;        // 0-100
  message?: string;
  eta?: Duration;
}
```

### Sub-Contract Spawning

Agents can spawn sub-contracts during execution:

```typescript
// Parent contract: "Implement feature X"
// Agent realizes it needs to:
// 1. Update database schema
// 2. Add API endpoint
// 3. Update frontend

const subContracts = await executor.spawn(parentId, [
  {
    input: { schema: 'users', changes: [...] },
    output: { artifacts: [{ type: 'migration' }] },
    verification: { checks: [{ type: 'test', command: 'bun run test:db' }] },
  },
  {
    input: { endpoint: '/api/users', spec: {...} },
    output: { artifacts: [{ type: 'file', path: 'src/api/users.ts' }] },
    blockedBy: [/* schema migration */],
  },
  // ...
]);

// Parent contract doesn't complete until all sub-contracts complete
```

---

## 5. Verification

### Verification Flow

```
Contract completes
       │
       ▼
┌─────────────────┐
│ Output Schema   │──► Invalid ──► FAIL
│ Validation      │
└────────┬────────┘
         │ Valid
         ▼
┌─────────────────┐
│ Automated Checks│──► Any fail ──► FAIL
│ (tests, lints)  │
└────────┬────────┘
         │ All pass
         ▼
┌─────────────────┐
│ Human Review    │──► Required? ─► Queue for human
│ (if required)   │       │
└────────┬────────┘       │ Not required
         │                │
         │◄───────────────┘
         ▼
┌─────────────────┐
│    COMPLETED    │
└─────────────────┘
```

### Automated Verification

```typescript
interface VerificationRunner {
  // Run all automated checks
  async verify(contract: Contract, output: unknown): Promise<VerificationResult> {
    const results: CheckResult[] = [];

    // 1. Schema validation
    const schemaValid = validateSchema(contract.output.schema, output);
    if (!schemaValid.valid) {
      return { passed: false, reason: 'schema_invalid', details: schemaValid.errors };
    }

    // 2. Run each check
    for (const check of contract.verification.checks ?? []) {
      const result = await this.runCheck(check, contract, output);
      results.push(result);

      // Fail fast or collect all?
      if (!result.passed && contract.verification.failFast) {
        break;
      }
    }

    const allPassed = results.every(r => r.passed);
    return {
      passed: allPassed,
      checks: results,
    };
  }

  async runCheck(check: VerificationCheck, contract: Contract, output: unknown): Promise<CheckResult> {
    switch (check.type) {
      case 'test':
        return this.runCommand(check.command!);
      case 'typecheck':
        return this.runCommand('bun run typecheck');
      case 'lint':
        return this.runCommand('bun run lint');
      case 'custom':
        return this.runScript(check.script!, { contract, output });
    }
  }
}
```

### Human-Required Verification

```typescript
interface HumanVerification {
  contractId: ContractId;
  assignedTo?: UserId;
  criteria: string[];          // What human should check
  deadline?: Date;

  // Outcome
  approved?: boolean;
  feedback?: string;
  approvedAt?: Date;
  approvedBy?: UserId;
}

// Human review queue
interface ReviewQueue {
  // Get pending reviews
  getPending(userId?: UserId): Promise<HumanVerification[]>;

  // Submit review decision
  submit(contractId: ContractId, decision: ReviewDecision): Promise<void>;

  // Escalate if not reviewed in time
  escalate(contractId: ContractId): Promise<void>;
}

interface ReviewDecision {
  approved: boolean;
  feedback?: string;
  requestChanges?: ChangeRequest[];
}
```

### Verification Caching

For efficiency, cache verification results:

```typescript
interface VerificationCache {
  // Key: hash of (contract input, output, verification spec)
  // Value: verification result

  // If same input produces same output with same checks,
  // skip re-running verification

  invalidateOn: ['file_change', 'dependency_change', 'check_change'];
}
```

---

## 6. Rollback and Compensation

### Rollback Types

```typescript
type RollbackStrategy =
  | 'git_reset'         // Reset to commit before contract started
  | 'file_restore'      // Restore specific files from backup
  | 'command'           // Run rollback commands
  | 'compensating'      // Create compensating contract
  | 'manual'            // Human performs rollback
  | 'none';             // No rollback possible

interface RollbackContext {
  contract: Contract;
  error: Error;

  // State before contract started
  checkpoint?: {
    gitCommit?: string;
    files?: FileSnapshot[];
    dbState?: unknown;
  };

  // What the contract actually changed
  changes?: {
    files?: FileChange[];
    commits?: string[];
    deployments?: DeploymentId[];
  };
}
```

### Automatic Rollback

```typescript
class RollbackExecutor {
  async rollback(context: RollbackContext): Promise<RollbackResult> {
    const spec = context.contract.rollback;

    if (!spec || spec.type === 'none') {
      return { performed: false, reason: 'no_rollback_spec' };
    }

    switch (spec.type) {
      case 'automatic':
        // Run rollback commands
        for (const cmd of spec.commands ?? []) {
          await this.runCommand(cmd);
        }
        break;

      case 'git_reset':
        // Reset to checkpoint commit
        if (context.checkpoint?.gitCommit) {
          await this.gitReset(context.checkpoint.gitCommit);
        }
        break;

      case 'compensating':
        // Create and execute compensating contract
        await this.createCompensatingContract(context);
        break;

      case 'manual':
        // Queue for human
        await this.queueManualRollback(context);
        break;
    }

    return { performed: true };
  }
}
```

### Compensating Transactions (Saga Pattern)

When simple rollback isn't possible, use compensating transactions:

```typescript
// Original contract: "Deploy to production"
// If it fails after some steps completed...

const compensatingContract: ContractSpec = {
  input: {
    originalContractId: failedContract.id,
    completedSteps: ['build', 'push_image', 'update_k8s'],
    failedAt: 'health_check',
  },
  output: {
    // System should be in state as if original never ran
    artifacts: [],
  },
  verification: {
    checks: [
      { type: 'custom', script: 'verify-rollback.ts' },
    ],
  },
};

// Compensating contract knows what was done and undoes it
// in reverse order (like database transaction rollback)
```

### Checkpointing

For long-running contracts, create checkpoints:

```typescript
interface Checkpoint {
  contractId: ContractId;
  timestamp: Date;

  // What was the state at this point?
  state: {
    gitCommit?: string;
    progress?: Progress;
    intermediateOutput?: unknown;
  };

  // Can we resume from here?
  resumable: boolean;
}

// On failure, rollback to last checkpoint instead of start
// Reduces wasted work
```

---

## 7. API Surface

### REST API

```typescript
// Contracts
POST   /contracts                    // Create contract
GET    /contracts                    // List contracts (with filters)
GET    /contracts/:id                // Get contract
PATCH  /contracts/:id                // Update contract (limited fields)
DELETE /contracts/:id                // Cancel contract

// Claiming
POST   /contracts/:id/claim          // Claim contract
POST   /contracts/:id/unclaim        // Release contract
POST   /contracts/:id/start          // Start execution
POST   /contracts/:id/heartbeat      // Send heartbeat
POST   /contracts/:id/checkpoint     // Create checkpoint
POST   /contracts/:id/complete       // Submit completion
POST   /contracts/:id/fail           // Report failure

// Verification
POST   /contracts/:id/verify         // Trigger verification
GET    /contracts/:id/verification   // Get verification status
POST   /contracts/:id/review         // Submit human review

// Sub-contracts
POST   /contracts/:id/spawn          // Create sub-contracts
GET    /contracts/:id/children       // Get sub-contracts

// Queries
GET    /contracts/ready              // Contracts ready for claiming
GET    /contracts/blocked            // Blocked contracts (with reasons)
GET    /contracts/failed             // Failed contracts
GET    /projects/:id/contracts       // Contracts for a project
```

### WebSocket API (Real-time)

```typescript
// Subscribe to contract events
ws.subscribe('contract.*.status')     // All status changes
ws.subscribe('contract.{id}.*')       // All events for specific contract
ws.subscribe('project.{id}.ready')    // New ready contracts in project

// Event types
interface ContractEvent {
  type: 'status_change' | 'claimed' | 'progress' | 'completed' | 'failed';
  contractId: ContractId;
  timestamp: Date;
  data: unknown;
}
```

### CLI

```bash
# Create contract from spec file
stead contract create --file contract.yaml

# List ready contracts
stead contract list --status ready --project myproject

# Claim and execute (for testing)
stead contract claim <id>
stead contract complete <id> --output output.json
stead contract fail <id> --error "reason"

# View contract graph
stead contract graph --project myproject

# Watch contracts in real-time
stead contract watch --project myproject
```

### Agent SDK

```typescript
import { ContractEngine, Agent } from '@stead/sdk';

const engine = new ContractEngine({ url: 'http://localhost:3000' });
const agent = new Agent({ id: 'my-agent', engine });

// Register capabilities
agent.register({
  capabilities: ['code', 'test', 'deploy'],
  maxConcurrent: 3,
});

// Handle contracts
agent.on('contract', async (contract) => {
  try {
    await agent.start(contract.id);

    // Do work...
    await agent.heartbeat(contract.id, { percent: 50 });

    const output = await doWork(contract.input);
    await agent.complete(contract.id, output);

  } catch (error) {
    await agent.fail(contract.id, error);
  }
});

// Start listening for work
agent.listen();
```

---

## 8. Integration with Claude Code Tasks

### Current Claude Code Task System

Claude Code has a session-scoped task system:
- `TaskCreate`: Create tasks with subject, description
- `TaskUpdate`: Update status, add dependencies, set owner
- Tasks are ephemeral (don't persist across sessions)
- Good for within-session orchestration, not persistent tracking

### Integration Strategy

**Option A: Replace Tasks entirely**
- Contract Engine becomes the task backend
- Claude Code uses contracts instead of tasks
- Full persistence, verification, rollback

**Option B: Bridge layer**
- Tasks remain for quick, session-scoped work
- Contracts for persistent, verified work
- Bridge converts tasks to contracts when needed

**Option C: Contracts as super-tasks**
- Tasks are lightweight contracts (no verification, no rollback)
- When you need more guarantees, upgrade to full contract
- Gradual adoption path

### Recommended: Option B (Bridge) for MVP

```typescript
// In Claude Code agent
import { ContractBridge } from '@stead/claude-code';

// Create task (ephemeral, quick)
const task = await TaskCreate({
  subject: 'Quick fix',
  description: 'Fix the typo',
});

// Or create contract (persistent, verified)
const contract = await ContractBridge.create({
  input: { file: 'src/app.ts', issue: 'typo' },
  output: { artifacts: [{ type: 'file', path: 'src/app.ts' }] },
  verification: { checks: [{ type: 'typecheck' }] },
});

// Bridge can upgrade task to contract
await ContractBridge.upgrade(task.id, {
  verification: { type: 'human', criteria: ['code review'] },
});
```

### CLAUDE.md Contract Definitions

Allow defining contracts in CLAUDE.md:

```yaml
# In CLAUDE.md
contracts:
  code_change:
    input:
      schema: { type: 'object', properties: { files: { type: 'array' } } }
    verification:
      checks:
        - type: typecheck
        - type: lint
        - type: test
    rollback:
      type: git_reset

  deploy:
    input:
      schema: { type: 'object', properties: { environment: { type: 'string' } } }
    verification:
      type: hybrid
      checks:
        - type: custom
          command: 'bun run test:e2e'
      humanReview:
        required: true
        criteria:
          - "Verify staging deployment works"
          - "Check metrics dashboards"
    rollback:
      type: compensating
```

---

## 9. Hard Problems

### 1. Schema Evolution

**Problem**: Contracts have typed input/output. What happens when schemas change?

**Approaches**:
- Version schemas explicitly
- Backward compatibility requirements
- Migration contracts that transform data between versions
- Schema registry with compatibility checking

### 2. Long-Running Contracts

**Problem**: Some work takes hours or days. How to handle?

**Approaches**:
- Checkpointing with resumability
- Sub-contract decomposition
- Heartbeat with generous timeouts
- Human escalation on stall

### 3. Non-Deterministic Verification

**Problem**: AI output isn't deterministic. Same input can produce valid but different outputs.

**Approaches**:
- Verification checks properties, not exact values
- Multiple valid output schemas
- Semantic similarity checks
- Human verification for ambiguous cases

### 4. Resource Contention

**Problem**: Multiple contracts want the same resource (file, port, etc.)

**Approaches**:
- Explicit resource claims in contract spec
- Resource locking with deadlock detection
- Queuing with priority
- Resource virtualization (containers, worktrees)

### 5. Cascading Rollbacks

**Problem**: Contract A completes, B depends on A and starts, then A's verification fails. What happens to B?

**Approaches**:
- Wait for verification before allowing dependents to start
- Cascading rollback (expensive but correct)
- Eventual consistency model
- Human decision point

### 6. Agent Capability Matching

**Problem**: Not all agents can execute all contracts. How to match?

**Approaches**:
- Capability tags on agents and contracts
- Agent self-assessment before claiming
- Learning from success/failure rates
- Human assignment for ambiguous cases

### 7. Cost Tracking Accuracy

**Problem**: Token counts are available, but cost attribution is complex when sub-agents spawn.

**Approaches**:
- Hierarchical cost aggregation
- Budget limits on contracts
- Cost estimation before execution
- Kill switch on budget exceeded

### 8. Partial Success

**Problem**: Contract produces 80% of expected output. Is it complete?

**Approaches**:
- All-or-nothing (strict)
- Partial completion with handoff
- Quality scores with thresholds
- Human judgment for edge cases

### 9. Verification Flakiness

**Problem**: Tests pass sometimes, fail sometimes. Verification becomes unreliable.

**Approaches**:
- Retry verification N times
- Quarantine flaky checks
- Track flakiness metrics
- Require human verification for flaky contracts

### 10. State Synchronization

**Problem**: Contract engine state vs actual world state can diverge.

**Approaches**:
- Periodic reconciliation jobs
- Idempotent operations
- Audit logs for forensics
- Manual correction tools

---

## 10. MVP Definition

### What to Build First

**Week 1-2: Core Engine**
- Contract schema (simplified)
- State machine with transitions
- SQLite storage
- Basic REST API (create, get, list, update status)

**Week 3-4: Agent Integration**
- Claim/unclaim flow
- Heartbeat monitoring
- Simple verification (command execution)
- CLI for manual testing

**Week 5-6: Claude Code Bridge**
- SDK for Claude Code agents
- Task to Contract conversion
- Basic sub-contract spawning

### MVP Contract Schema (Simplified)

```typescript
interface MVPContract {
  id: string;
  status: 'pending' | 'ready' | 'claimed' | 'executing' | 'completed' | 'failed';

  // Simplified input/output
  input: Record<string, unknown>;
  expectedOutput: string;  // Description, not schema

  // Simple verification
  verifyCommand?: string;  // Shell command that returns 0 on success

  // Dependencies
  blockedBy: string[];

  // Ownership
  owner?: string;

  // Timestamps
  createdAt: Date;
  updatedAt: Date;
}
```

### MVP API

```
POST   /contracts              // Create
GET    /contracts              // List (with ?status=ready filter)
GET    /contracts/:id          // Get
POST   /contracts/:id/claim    // Claim
POST   /contracts/:id/complete // Complete (runs verification)
POST   /contracts/:id/fail     // Fail
```

### MVP Verification

```typescript
// Just run a command and check exit code
async function verify(contract: MVPContract): Promise<boolean> {
  if (!contract.verifyCommand) return true;

  const result = await runCommand(contract.verifyCommand, { timeout: 60000 });
  return result.exitCode === 0;
}
```

### What MVP Doesn't Include

- Human verification queue
- Rollback/compensation
- Resource claims
- Cost tracking
- Real-time WebSocket
- Schema validation
- Checkpointing

These come in v2 after MVP proves the model.

---

## 11. Open Questions

1. **Storage**: SQLite for MVP, but what for production? PostgreSQL? Durable object storage?

2. **Distribution**: Single node for MVP, but how to scale? Separate workers? Event sourcing?

3. **Identity**: How do agents identify themselves? API keys? OAuth? Something else?

4. **Multitenancy**: One engine per project? Shared engine with project isolation?

5. **Observability**: What metrics matter? How to expose them?

6. **Security**: Contracts can run arbitrary commands. Sandboxing?

---

## References

Research sources:
- [Temporal Saga Pattern](https://temporal.io/blog/saga-pattern-made-easy)
- [Microsoft Saga Pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/saga)
- [XState Statecharts](https://stately.ai/docs/state-machines-and-statecharts)
- [Claude Code Task System](https://dev.to/bhaidar/the-task-tool-claude-codes-agent-orchestration-system-4bf2)
- [Compensation Transaction Patterns](https://orkes.io/blog/compensation-transaction-patterns/)
- [Modeling Saga as State Machine](https://dzone.com/articles/modelling-saga-as-a-state-machine)
