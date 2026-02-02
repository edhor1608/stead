# Decision: Contract Schema Format

## Context

Contracts are stead's core abstraction for agent work. They define:
- Input specification (what the agent receives)
- Output specification (what it must produce)
- Verification criteria (executable checks, not descriptions)
- Rollback procedure (recovery path on failure)
- Dependencies (other contracts that must complete first)
- Resources (ports, sessions, file locks needed)

Key constraints:
- **Agents are primary consumers** - Claude Code, etc. parse and execute these
- **Control room UI generates contracts** - humans don't write raw schema
- **Must be typed** - agents need unambiguous structure
- **Verification must be executable** - not prose acceptance criteria

Question: What format best fits agent consumption?

## Decision

**TypeScript-native schema with JSON serialization.**

Specifically:
1. **Schema definition**: TypeScript interfaces (the "type system")
2. **Contract instances**: JSON conforming to those interfaces (the "data")
3. **Verification**: TypeScript predicates compiled to JavaScript (the "checks")
4. **Storage**: JSONL files (append-only, one contract per line)

The contract engine validates JSON instances against TypeScript interfaces at runtime using a schema validator (like Zod or Valibot).

## Rationale

**Agents already think in TypeScript.**

Claude Code's internal reasoning naturally produces TypeScript-like structures. When asked to "define a contract," it generates interface-shaped descriptions. Fighting this is wasted effort.

**Separation of concerns:**
- TypeScript interfaces = the contract *language* (what's possible)
- JSON instances = contract *data* (specific contracts)
- JavaScript predicates = verification *logic* (how to check)

**Why not alternatives:**

| Format | Why not |
|--------|---------|
| Pure JSON Schema | Verbose, poor ergonomics, stringly-typed |
| Protocol Buffers | Overkill, tooling overhead, poor ecosystem fit |
| Custom DSL | Maintenance burden, agents must learn new syntax |
| YAML | Ambiguous parsing, whitespace-sensitive, no types |
| S-expressions | Unfamiliar, no tooling, poor editor support |

**Why TypeScript specifically:**
- Claude Code already uses TypeScript for tool definitions
- First-class IDE support (autocomplete, validation)
- Predicates are just functions - no special verification language
- Compiles away completely - runtime is pure JSON + JS

**Why JSONL for storage:**
- Append-only semantics match contract lifecycle
- Easy to stream, parse incrementally
- Git-friendly (line-based diffs)
- Standard format, any language can read

## Examples

### Schema Definition (TypeScript)

```typescript
// contracts/schema.ts

interface ContractInput {
  type: string;
  data: unknown;
  constraints?: Record<string, unknown>;
}

interface ContractOutput {
  type: string;
  schema: unknown;  // JSON Schema or TypeScript type reference
  artifacts?: string[];  // file paths produced
}

interface Verification {
  predicate: string;  // JavaScript function as string, or reference
  timeout_ms?: number;
  retry?: { max: number; delay_ms: number };
}

interface Rollback {
  strategy: 'git_reset' | 'restore_snapshot' | 'custom';
  custom?: string;  // JavaScript function if strategy is 'custom'
}

interface Resource {
  type: 'port' | 'session' | 'file_lock' | 'env';
  spec: unknown;
}

interface Contract {
  id: string;
  version: number;
  created_at: string;  // ISO 8601

  input: ContractInput;
  output: ContractOutput;

  verification: Verification;
  rollback: Rollback;

  dependencies?: string[];  // contract IDs
  resources?: Resource[];

  metadata?: {
    project: string;
    agent?: string;
    human_summary?: string;  // for control room display only
  };
}
```

### Contract Instance (JSON)

```json
{
  "id": "qwer-q/fix-memory-leak/001",
  "version": 1,
  "created_at": "2026-02-02T14:30:00Z",

  "input": {
    "type": "codebase_context",
    "data": {
      "files": ["src/memory/*.ts"],
      "issue": "Memory grows unbounded when processing large datasets"
    },
    "constraints": {
      "no_new_dependencies": true,
      "preserve_public_api": true
    }
  },

  "output": {
    "type": "code_change",
    "schema": {
      "modified_files": "string[]",
      "test_coverage": "boolean"
    },
    "artifacts": ["src/memory/pool.ts", "src/memory/pool.test.ts"]
  },

  "verification": {
    "predicate": "(result) => result.test_coverage && result.modified_files.length > 0",
    "timeout_ms": 30000
  },

  "rollback": {
    "strategy": "git_reset"
  },

  "dependencies": ["qwer-q/setup-test-env/001"],

  "resources": [
    { "type": "port", "spec": { "range": [3200, 3299] } },
    { "type": "file_lock", "spec": { "paths": ["src/memory/*"] } }
  ],

  "metadata": {
    "project": "qwer-q",
    "agent": "claude-code",
    "human_summary": "Fix memory leak in dataset processing"
  }
}
```

### Verification Predicate (JavaScript)

```javascript
// For complex verification, predicates can be external files
// contracts/verify/qwer-q/memory-fix.js

export default async function verify(result, context) {
  // Check tests pass
  const testResult = await context.run('bun', ['test', 'src/memory/']);
  if (testResult.exitCode !== 0) return false;

  // Check no memory growth
  const memCheck = await context.run('bun', ['run', 'benchmark:memory']);
  const baseline = context.input.constraints.memory_baseline;
  return memCheck.peakMemory <= baseline * 1.1;  // 10% tolerance
}
```

### Storage (JSONL)

```
{"id":"qwer-q/fix-memory-leak/001","version":1,"created_at":"2026-02-02T14:30:00Z","input":{...},"output":{...},...}
{"id":"qwer-q/fix-memory-leak/001","version":2,"created_at":"2026-02-02T15:45:00Z","input":{...},"output":{...},...}
{"id":"picalyze/add-export/001","version":1,"created_at":"2026-02-02T16:00:00Z","input":{...},"output":{...},...}
```

## Trade-offs

### What we gain

- **Zero translation cost** - agents output what the schema expects
- **Type safety** - IDE catches errors before runtime
- **Familiar tooling** - TypeScript, JSON, JavaScript are universal
- **Executable verification** - predicates are code, not prose
- **Composable** - contracts reference each other naturally
- **Auditable** - JSONL provides append-only history

### What we give up

- **Not language-agnostic** - tied to JS/TS ecosystem (acceptable: most agent tooling is JS)
- **Predicates can be arbitrary code** - security consideration for untrusted contracts
- **No visual representation** - control room must render JSON (but that's its job)
- **Schema evolution requires care** - version field exists for this

### Mitigations

- **Security**: Predicates run in sandboxed context, timeout enforced
- **Language lock-in**: JSON is the data format; only verification requires JS
- **Schema evolution**: Version field + migration scripts + backwards-compatible changes

## Open Questions (for later)

- Should predicates be a restricted subset of JavaScript?
- How do we handle predicate dependencies (npm packages)?
- Should there be a "dry run" mode that validates without executing?

---

*Decision made: 2026-02-02*
