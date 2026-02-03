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

**Rust-native schema with JSON serialization.**

Specifically:
1. **Schema definition**: Rust structs (the "type system")
2. **Contract instances**: JSON conforming to those structs (the "data")
3. **Verification**: Shell commands + expression assertions (the "checks")
4. **Storage**: JSONL files (append-only, one contract per line)

The contract engine validates JSON instances against Rust structs at compile time via serde deserialization.

## Rationale

**Agents output structured data; Rust consumes it naturally.**

Claude Code and other agents produce JSON-structured outputs. Rust with serde handles JSON deserialization with compile-time type checking, ensuring contract data is always valid before processing begins.

**Separation of concerns:**
- Rust structs = the contract *language* (what's possible)
- JSON instances = contract *data* (specific contracts)
- Shell commands + assertions = verification *logic* (how to check)

**Why not alternatives:**

| Format | Why not |
|--------|---------|
| Pure JSON Schema | Verbose, poor ergonomics, stringly-typed |
| Protocol Buffers | Overkill, tooling overhead, poor ecosystem fit |
| Custom DSL | Maintenance burden, agents must learn new syntax |
| YAML | Ambiguous parsing, whitespace-sensitive, no types |
| S-expressions | Unfamiliar, no tooling, poor editor support |
| TypeScript/JS | Runtime type errors, heavier deployment, no single binary |

**Why Rust specifically:**
- Single binary distribution - no runtime dependencies to manage
- Compile-time guarantees - schema violations caught before runtime
- Performance - fast contract validation and execution
- Memory safety - no crashes from null pointers or buffer overflows

**Why JSONL for storage:**
- Append-only semantics match contract lifecycle
- Easy to stream, parse incrementally
- Git-friendly (line-based diffs)
- Standard format, any language can read

## Examples

### Schema Definition Example (Rust)

```rust
// src/contracts/schema.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractInput {
    #[serde(rename = "type")]
    pub input_type: String,
    pub data: serde_json::Value,
    #[serde(default)]
    pub constraints: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractOutput {
    #[serde(rename = "type")]
    pub output_type: String,
    pub schema: serde_json::Value,
    #[serde(default)]
    pub artifacts: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Verification {
    pub command: String,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub assertions: Option<Vec<String>>,
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max: u32,
    pub delay_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackStrategy {
    GitReset,
    RestoreSnapshot,
    Custom,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rollback {
    pub strategy: RollbackStrategy,
    #[serde(default)]
    pub custom: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Port,
    Session,
    FileLock,
    Env,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    #[serde(rename = "type")]
    pub resource_type: ResourceType,
    pub spec: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub project: String,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub human_summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Contract {
    pub id: String,
    pub version: u32,
    pub created_at: String,  // ISO 8601

    pub input: ContractInput,
    pub output: ContractOutput,

    pub verification: Verification,
    pub rollback: Rollback,

    #[serde(default)]
    pub dependencies: Option<Vec<String>>,
    #[serde(default)]
    pub resources: Option<Vec<Resource>>,

    #[serde(default)]
    pub metadata: Option<ContractMetadata>,
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
      "files": ["src/memory/*.rs"],
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
    "artifacts": ["src/memory/pool.rs", "src/memory/pool_test.rs"]
  },

  "verification": {
    "command": "cargo test -p memory",
    "timeout_ms": 30000,
    "assertions": [
      "output.test_coverage == true",
      "output.modified_files.len() > 0"
    ]
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

### Verification Example (Shell Command + Assertions)

```yaml
# Verification runs a shell command and evaluates assertions against agent output
verification:
  command: "cargo test -p memory"
  timeout_ms: 30000
  assertions:
    - "output.test_coverage == true"
    - "output.modified_files.len() > 0"
```

For complex verification, the command can be a custom script:

```yaml
verification:
  command: "./scripts/verify-memory-fix.sh"
  timeout_ms: 60000
  assertions:
    - "exit_code == 0"
    - "output.peak_memory_mb <= input.constraints.memory_baseline * 1.1"
```

**Expression language:** Assertions use an expression evaluator (like [cel-rust](https://github.com/clarkmcc/cel-rust) or [rhai](https://rhai.rs/)) for inspecting the agent's structured output. The evaluator has access to:
- `output` - the agent's structured JSON output
- `input` - the original contract input
- `exit_code` - command exit code
- `stdout` / `stderr` - command output

### Storage (JSONL)

```jsonl
{"id":"qwer-q/fix-memory-leak/001","version":1,"created_at":"2026-02-02T14:30:00Z","input":{...},"output":{...},...}
{"id":"qwer-q/fix-memory-leak/001","version":2,"created_at":"2026-02-02T15:45:00Z","input":{...},"output":{...},...}
{"id":"picalyze/add-export/001","version":1,"created_at":"2026-02-02T16:00:00Z","input":{...},"output":{...},...}
```

## Trade-offs

### What we gain

- **Single binary** - no runtime dependencies, easy distribution
- **Compile-time safety** - schema violations caught at build time, not runtime
- **Performance** - fast contract parsing and validation
- **Memory safety** - no null pointer crashes or buffer overflows
- **Familiar data format** - JSON is universal; only the engine is Rust
- **Executable verification** - shell commands are language-agnostic
- **Composable** - contracts reference each other naturally
- **Auditable** - JSONL provides append-only history

### What we give up

- **No visual representation** - control room must render JSON (but that's its job)
- **Schema evolution requires care** - version field exists for this

### Mitigations

- **Learning curve**: serde derive macros hide most complexity; schema changes are infrequent
- **Compile time**: Release builds are one-time; development uses cargo check / cargo test
- **Schema evolution**: Version field + migration scripts + backwards-compatible changes
- **Verification flexibility**: Shell commands can invoke any language (Python, Node, etc.) if needed

## Open Questions (for later)

- Which expression evaluator? cel-rust (Google's CEL) vs rhai (Rust-native scripting)
- Should assertions support async operations (HTTP calls, file checks)?
- Should there be a "dry run" mode that validates without executing?
- How do we handle verification scripts that need specific toolchains?

---

*Decision made: 2026-02-02*
*Updated: 2026-02-03 - Changed from TypeScript to Rust*
