# Exploration: Transformation Layer for Agent-Driven Version Control

Date: 2026-02-02

## The Core Idea

Git is the "assembly language" of version control. Agents need a higher-level language that compiles to git.

**Don't replace git. Build a layer above it.**

- Agents work in a structured workspace (not raw files)
- Changes are expressed as transformations, not diffs
- The layer compiles down to git commits for interop
- Merge conflicts are resolved by re-running transformations, not manual resolution

---

## 1. What Would "Transformations" Look Like vs Diffs?

### The Problem with Diffs

Git diffs are **textual patches** — they describe changes as "remove these lines, add these lines at this position." This representation:

- Loses semantic meaning (a rename looks like delete + add)
- Breaks on minor context changes
- Can't distinguish intent from implementation
- Makes merge conflicts inevitable when changes touch nearby lines

### Transformation Types

A transformation layer would capture **intent and semantic operations**, not textual changes.

| Category | Examples |
|----------|----------|
| **Structural** | `rename(function, old_name, new_name)`, `move(function, from_file, to_file)`, `extract(method, from_function)` |
| **Content** | `modify(function_body, new_implementation)`, `add(parameter, function, param_def)` |
| **Organizational** | `create(file, path)`, `delete(file)`, `reorganize(imports)` |
| **Composite** | `refactor(extract_interface, class)`, `apply(pattern, visitor, to_class)` |
| **Raw** | `patch(file, unified_diff)` — fallback for unstructured changes |

### Transformation Schema (Draft)

```typescript
interface Transformation {
  id: string;
  type: TransformationType;
  timestamp: number;
  agent_id: string;

  // What this transformation operates on
  target: {
    type: 'file' | 'function' | 'class' | 'module' | 'symbol';
    path: string;
    identifier?: string;  // e.g., function name
    range?: { start: Position; end: Position };
  };

  // The semantic operation
  operation: {
    kind: string;  // 'rename', 'move', 'modify', 'add', 'delete', etc.
    params: Record<string, unknown>;
  };

  // For verification and replay
  preconditions: Condition[];  // State that must be true before applying
  postconditions: Condition[];  // State that must be true after applying

  // Human-readable intent
  intent: string;  // "Extract authentication logic into separate module"

  // Dependency chain
  depends_on: string[];  // Other transformation IDs

  // The actual implementation (for compilation to git)
  implementation: {
    patches: UnifiedDiff[];  // What actually gets applied to files
    metadata?: Record<string, unknown>;
  };
}
```

### Key Insight: Dual Representation

Every transformation has two representations:
1. **Semantic** — the intent and operation (for replay, conflict resolution)
2. **Implementation** — the actual patches (for applying to files, compiling to git)

The semantic layer enables intelligent conflict resolution. The implementation layer ensures we always have a concrete fallback.

---

## 2. How Would the Structured Workspace Differ from a File Tree?

### Current Reality: File Tree

Agents see the same thing humans see:
```text
src/
  auth/
    login.ts
    session.ts
  api/
    routes.ts
```

This is a serialized view. The agent must parse files to understand structure.

### Structured Workspace: AST-Aware Layer

The workspace would expose **semantic structure**, not just files:

```typescript
interface StructuredWorkspace {
  // Query by semantic type
  functions: Map<QualifiedName, FunctionNode>;
  classes: Map<QualifiedName, ClassNode>;
  modules: Map<Path, ModuleNode>;
  types: Map<QualifiedName, TypeNode>;

  // Relationships
  callGraph: DirectedGraph<FunctionNode>;
  dependencyGraph: DirectedGraph<ModuleNode>;
  typeHierarchy: DirectedGraph<TypeNode>;

  // Semantic search
  find(query: SemanticQuery): Node[];

  // The underlying files (always accessible)
  files: Map<Path, FileContent>;
}
```

### Benefits for Agents

1. **Query by intent** — "Find all functions that call `authenticate()`" instead of grep
2. **Understand impact** — "What breaks if I rename this?"
3. **Parallel safety** — Know which transformations can run concurrently
4. **Verification** — Check pre/postconditions before and after transforms

### Implementation Approaches

**Option A: Full AST Persistence**
- Parse all source files, maintain full AST
- Similar to JetBrains MPS projectional editing
- Pro: Complete semantic understanding
- Con: Language-specific parsers needed, high complexity

**Option B: Index + On-Demand Parsing**
- Maintain lightweight index (symbols, locations, basic relationships)
- Parse specific files/regions when needed
- Pro: Simpler, works with existing parsers (tree-sitter)
- Con: Index can drift from reality

**Option C: Hybrid with LSP**
- Use Language Server Protocol for semantic queries
- LSP servers already exist for most languages
- Pro: Reuse existing infrastructure
- Con: LSP designed for IDE, not batch operations

**Recommended: Option B with LSP fallback**
- Lightweight symbol index (like ctags on steroids)
- Tree-sitter for fast, incremental parsing
- LSP for complex queries when needed
- Always keep raw files as ground truth

---

## 3. How Would Transformations Compile to Git Commits?

### The Compilation Process

```text
┌─────────────────┐     ┌──────────────────┐     ┌────────────────┐
│ Transformations │────▶│  Transformation  │────▶│  Git Commits   │
│   (semantic)    │     │    Compiler      │     │   (patches)    │
└─────────────────┘     └──────────────────┘     └────────────────┘
```

### Compilation Strategies

**Strategy 1: One Transform = One Commit**
```text
Transform: rename(function, 'auth', 'authenticate')
    ↓
Commit: "Rename function 'auth' to 'authenticate'"
        + metadata in commit message or notes
```

Pro: Fine-grained history, easy to trace
Con: Noisy git log, many small commits

**Strategy 2: Squash to Logical Units**
```text
Transforms: [rename, move, modify, modify, modify]
    ↓
Commit: "Refactor auth module: extract and rename"
        + transformation log as commit note
```

Pro: Cleaner git history
Con: Loses individual transform traceability

**Strategy 3: Hybrid with Metadata Branch**
```text
main branch:     Squashed commits (human-readable)
.stead branch:   Full transformation log (machine-readable)
```

Pro: Best of both worlds
Con: Extra branch to maintain

### Preserving Transformation Metadata

Git commit messages are limited. Options for storing transformation data:

1. **Git notes** — `git notes add -m <json>` — detached, can be lost
2. **Trailer lines** — `Transform-Id: abc123` — in commit message, limited size
3. **Metadata branch** — full JSON logs on separate branch — survives all operations
4. **External store** — database/file outside git — requires sync

**Recommended: Metadata branch + trailers**
- Commit messages include `Transform-Id: <id>` trailer
- Full transformation JSON stored in `.stead/transforms/` on a metadata branch
- Can reconstruct transformation history from either

### Compilation Example

```text
Input: Transform {
  type: 'rename',
  target: { type: 'function', path: 'src/auth.ts', identifier: 'login' },
  operation: { kind: 'rename', params: { newName: 'authenticate' } },
  intent: "Rename login to authenticate for clarity"
}

Output:
  1. Apply patches to workspace files
  2. Generate commit:
     ---
     refactor(auth): rename login to authenticate

     Rename login to authenticate for clarity

     Transform-Id: t_abc123
     Transform-Type: rename
     ---
  3. Update .stead/transforms/t_abc123.json on metadata branch
```

---

## 4. How Would Conflict Resolution via Re-Running Work?

### The Problem with Git Merge Conflicts

Git conflicts occur when:
- Two branches modify the same lines
- Context lines have shifted

Git's resolution: "Here are both versions, human figure it out."

### Transformation-Based Resolution

Instead of merging patches, **replay transformations on the merged base**.

```text
        base
       /    \
      A      B
       \    /
       merge?

Git approach:
  - Apply A's patches to base
  - Apply B's patches to A's result
  - Conflict if patches overlap

Transform approach:
  - Merge base states
  - Replay A's transformations on merged base
  - Replay B's transformations on merged base
  - Transformations are semantic, so they adapt
```

### Why This Works

Consider a rename conflict:

```text
Base:    function login() { ... }
Branch A: Renames login → authenticate
Branch B: Modifies login body

Git: CONFLICT — both touched login()
Transform:
  1. Apply A's rename: login → authenticate
  2. Apply B's modify to 'authenticate' (name resolved semantically)
  → No conflict
```

The transformation knows it operates on a **function identity**, not a **line range**.

### Conflict Categories

| Category | Resolution Strategy |
|----------|---------------------|
| **Structural + Content** | Apply structural first, then content adapts |
| **Same target, different ops** | May compose (rename + modify) or conflict (delete + modify) |
| **True semantic conflict** | Two agents changed same function body differently → requires decision |
| **Ordering conflict** | A depends on B, but B depends on A → cycle detection |

### True Conflicts Still Need Decisions

Not all conflicts disappear. When two agents modify the same function body with different implementations:

```typescript
Base:    function calc() { return x + y; }
Agent A: function calc() { return x * y; }  // Multiply
Agent B: function calc() { return x - y; }  // Subtract
```

This is a **true semantic conflict** — both want different behavior. Resolution options:

1. **Human decision** — escalate to control room
2. **Priority rules** — later transform wins, or higher-priority agent wins
3. **Test-based** — run tests, pick version that passes
4. **Compositional** — if possible, combine (rare for same-target modifications)

### Resolution Algorithm (Sketch)

```python
def resolve_transforms(base, transforms_a, transforms_b):
    merged_base = git_merge(base)  # Standard git merge of base state

    # Sort all transforms by dependency, timestamp
    all_transforms = topological_sort(transforms_a + transforms_b)

    result = merged_base
    conflicts = []

    for transform in all_transforms:
        # Check if transform can apply
        if transform.preconditions_met(result):
            result = transform.apply(result)
        else:
            # Preconditions failed — try to adapt
            adapted = transform.adapt_to(result)
            if adapted:
                result = adapted.apply(result)
            else:
                conflicts.append(transform)

    return result, conflicts
```

### Research Reference: Pijul's Approach

[Pijul](https://pijul.org/model/) uses category theory (pushouts) to define merges mathematically. Key insight: patches that commute can be applied in any order with the same result. Pijul's patches are designed to commute whenever semantically possible.

Our transformation layer could adopt similar principles:
- Define which transformation types commute
- Non-commuting transforms on same target = conflict
- Commuting transforms = order-independent merge

---

## 5. What Metadata Would Be Tracked Beyond Git?

### Core Metadata Categories

**1. Transformation Provenance**
```typescript
interface TransformProvenance {
  transform_id: string;
  created_at: number;
  agent_id: string;
  contract_id?: string;  // Link to stead contract system
  parent_transforms: string[];

  // Why this transform was created
  trigger: 'contract' | 'manual' | 'refactor' | 'fix' | 'test';
  intent: string;
}
```

**2. Semantic Relationships**
```typescript
interface SemanticRelationships {
  // What this transform affects
  affects: {
    files: string[];
    symbols: string[];
    modules: string[];
  };

  // What depends on this transform
  dependents: string[];

  // Related transforms (same contract, same feature)
  related: string[];
}
```

**3. Verification State**
```typescript
interface VerificationState {
  transform_id: string;

  // Pre/post condition checks
  preconditions_checked: boolean;
  postconditions_checked: boolean;

  // Test results
  tests_run: TestResult[];

  // Type checking
  typecheck_passed: boolean;

  // Human review
  reviewed_by?: string;
  review_status?: 'approved' | 'rejected' | 'pending';
}
```

**4. Resource Tracking**
```typescript
interface ResourceUsage {
  transform_id: string;

  tokens_used: number;
  compute_time_ms: number;
  api_calls: number;
  cost_usd: number;
}
```

### Storage Strategy

| Data Type | Storage Location | Rationale |
|-----------|------------------|-----------|
| Transform definitions | `.stead/transforms/` (git) | Version with code |
| Verification state | `.stead/verification/` (git) | Track what's been verified |
| Resource usage | External DB | High-frequency updates |
| Agent sessions | External DB | Not code-related |
| Semantic index | Local cache | Derived, can rebuild |

---

## 6. How Would This Interop with GitHub/GitLab PRs?

### The Fundamental Constraint

GitHub/GitLab see **git commits and diffs**. They don't understand transformations.

**The transformation layer must be invisible to external platforms.**

### PR Creation Flow

```text
┌───────────────────┐
│ Agent works with  │
│ transformations   │
└────────┬──────────┘
         │ compile
         ▼
┌───────────────────┐
│ Git commits       │
│ (with metadata)   │
└────────┬──────────┘
         │ push
         ▼
┌───────────────────┐
│ GitHub PR         │
│ (normal diffs)    │
└───────────────────┘
```

### Enhancing PR Experience

Even though GitHub sees commits, we can add value:

**1. PR Description Generation**
```markdown
## Changes

This PR contains 5 transformations:

1. `rename(function, 'login', 'authenticate')` — Clarify function purpose
2. `extract(module, 'auth', from: 'utils')` — Separate concerns
3. `modify(function_body, 'authenticate')` — Add token refresh
4. ...

## Affected Areas
- `src/auth/` (3 files)
- `src/api/routes.ts` (1 function)

## Verification
- [x] Type check passed
- [x] Unit tests passed
- [x] Integration tests passed
```

**2. Review Assistance**
- Link to transformation details
- Show semantic diff (not just line diff)
- Highlight "this is a rename, ignore the noise"

**3. Comment Intelligence**
When reviewer comments on line 47, map back to:
- Which transformation created this change
- What the intent was
- Suggest response based on transformation context

### Merge Behavior

When PR is merged on GitHub:

1. Transformation layer detects merge
2. Validates that git state matches expected transformation output
3. If mismatch (manual edits on GitHub): reconcile or flag
4. Update transformation metadata branch

### GitHub Actions Integration

```yaml
# .github/workflows/stead-verify.yml
on: pull_request

jobs:
  verify-transformations:
    runs-on: ubuntu-latest
    steps:
      - uses: stead/verify-action@v1
        with:
          # Verify transforms compile to actual diff
          verify-compilation: true
          # Run semantic conflict check
          check-conflicts: true
```

---

## 7. How Would CI/CD See These Changes?

### The Simple Answer

**CI/CD sees normal git commits.** The transformation layer is transparent.

```text
┌─────────────────┐     ┌─────────────────┐
│ Transformation  │────▶│  Git Commits    │
│    Layer        │     │                 │
└─────────────────┘     └────────┬────────┘
                                 │
                                 ▼
                        ┌─────────────────┐
                        │   CI/CD         │
                        │ (GitHub Actions,│
                        │  Jenkins, etc.) │
                        └─────────────────┘
```

### Enhanced CI/CD Integration

But we can provide **additional signals** to CI/CD:

**1. Transformation-Aware Test Selection**
```yaml
# Only run tests affected by these transformations
- name: Run affected tests
  run: stead test --affected-by-transforms
```

Instead of running all tests, run tests that:
- Are in files touched by transformations
- Test functions modified by transformations
- Depend on modified symbols

**2. Semantic Change Detection**
```yaml
# Skip CI for pure renames/moves
- name: Check if substantive
  id: check
  run: |
    if stead transforms --only-structural; then
      echo "skip=true" >> $GITHUB_OUTPUT
    fi
```

**3. Verification Status**
```yaml
# Require pre/postconditions verified
- name: Verify transformations
  run: stead verify --strict
```

**4. Rollback Information**
If CI fails, provide rollback guidance:
```text
Transform t_abc123 (modify: authenticate) likely caused failure.
To rollback: stead revert t_abc123
```

---

## 8. What Are the Hard Problems?

### Problem 1: Language Diversity

Different languages have different:
- AST structures
- Refactoring semantics
- What "rename" or "move" means

**Mitigation:**
- Start with one language (TypeScript?)
- Abstract transformation types, language-specific implementations
- Use tree-sitter for parsing (supports 100+ languages)
- Accept that some transforms will be language-specific

### Problem 2: Imperfect Parsing

Real code has:
- Syntax errors (partial edits)
- Macros, templates, generated code
- Build-time transformations

**Mitigation:**
- Fall back to text patches when semantic fails
- Mark transforms as "degraded" when using fallback
- Track which areas of codebase have good vs poor semantic coverage

### Problem 3: Transform Composition Complexity

Transforms interact in complex ways:
- A moves function X
- B renames X
- C modifies X
- Order matters, combinatorics explode

**Mitigation:**
- Define clear composition rules
- Detect conflicts early (before applying)
- Use dependency graph to constrain ordering
- Accept that some conflicts require human decision

### Problem 4: State Drift

Git can be modified outside the transformation layer:
- Manual commits
- Other tools
- Direct file edits

**Mitigation:**
- Detect drift (compare expected vs actual state)
- Provide "reconcile" command to sync
- Mark drifted transforms as "invalidated"
- Don't fight it — humans/other tools will always edit directly

### Problem 5: Adoption Barrier

Developers have git muscle memory. New concepts are friction.

**Mitigation:**
- Transparent mode: agents use transforms, git output looks normal
- Humans can ignore layer entirely, just use git
- Value proposition must be clear and immediate
- Start with agent-to-agent workflows, not human-facing

### Problem 6: Metadata Maintenance

Transformation metadata can:
- Get out of sync with code
- Grow indefinitely
- Slow down operations

**Mitigation:**
- Compact old transforms (keep recent, summarize old)
- Garbage collect unreferenced transforms
- Store derived data in local cache, not git
- Periodic "rebase" of transformation history

### Problem 7: Merge at Platform Boundary

When code leaves stead (PR merged on GitHub by human), we lose control:
- Human edits during review
- Squash merge changes commit structure
- Manual conflict resolution

**Mitigation:**
- Detect and handle external changes gracefully
- "Import" external changes as raw patch transforms
- Accept that boundary crossings introduce uncertainty

---

## Prior Art and Related Systems

### Pijul
[Pijul](https://pijul.org/) — Patch-based VCS using category theory (pushouts) for mathematically sound merges. Key insight: patches can be designed to commute.

### JetBrains MPS
[MPS](https://www.jetbrains.com/mps/) — Projectional editor that manipulates AST directly. No parsing needed. Demonstrates structured editing at scale.

### Operational Transformation (OT)
[OT](https://ot.js.org/) — Real-time collaboration via operation transformation. Operations adjust to prior operations. Used in Google Docs.

### CRDTs
[Yjs](https://github.com/yjs/yjs), [Automerge](https://automerge.org/) — Conflict-free replicated data types. Concurrent edits merge automatically. Foundation of collaborative editors.

### Semantic Diff Tools
[SemanticDiff](https://semanticdiff.com/) — Understands code structure, hides irrelevant changes (whitespace, formatting), highlights renames and moves.

### AI Merge Tools
[MergeResolver](https://mergeresolver.github.io/), VS Code 1.105's AI merge — Use AST analysis and ML to suggest conflict resolutions. GitLab's AI Merge Agent claims 85% success rate.

### Moderne/OpenRewrite
[Moderne](https://www.moderne.ai/) — Lossless Semantic Tree (LST) for programmatic code transformation. Captures type info, dependencies, formatting.

---

## Proposed Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                     TRANSFORMATION LAYER                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │  Transform  │  │  Workspace  │  │  Conflict   │            │
│  │   Engine    │  │   Index     │  │  Resolver   │            │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘            │
│         │                │                │                    │
│         └────────────────┼────────────────┘                    │
│                          │                                      │
│  ┌───────────────────────┴───────────────────────┐             │
│  │              Core Abstraction                  │             │
│  │  - Transform types (rename, move, modify...)  │             │
│  │  - Semantic targets (function, class, module) │             │
│  │  - Pre/postconditions                         │             │
│  │  - Dependency tracking                        │             │
│  └───────────────────────┬───────────────────────┘             │
│                          │                                      │
│  ┌───────────────────────┴───────────────────────┐             │
│  │              Language Adapters                 │             │
│  │  - TypeScript  - Python  - Go  - Rust  - ...  │             │
│  │  - Tree-sitter parsing                        │             │
│  │  - LSP integration                            │             │
│  └───────────────────────────────────────────────┘             │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                        COMPILATION                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Transforms  ──▶  Patches  ──▶  Git Commits                    │
│                                                                 │
│  Metadata stored in .stead/ (on metadata branch)               │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                         GIT LAYER                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Standard git repository                                        │
│  - Normal commits (with Transform-Id trailers)                 │
│  - Push to GitHub/GitLab                                       │
│  - CI/CD sees normal git                                       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Proof of Concept
- Single language (TypeScript)
- Basic transforms: rename, move, modify
- Manual compilation to git
- No conflict resolution

### Phase 2: Core Engine
- Transform engine with pre/postconditions
- Workspace index (tree-sitter based)
- Automatic compilation to git commits
- Simple conflict detection

### Phase 3: Conflict Resolution
- Transform replay on merge
- Adapt transforms to new base
- True conflict escalation to human/control room

### Phase 4: Multi-Language
- Python, Go, Rust adapters
- Language-agnostic transform types
- Fallback to text patches

### Phase 5: Platform Integration
- GitHub PR enhancement
- CI/CD hooks
- Control room integration

---

## Open Questions

1. **Granularity:** What's the right size for a transform? Function-level? Statement-level?

2. **History:** How much transform history to keep? Compact after merge to main?

3. **Human edits:** When human edits directly, how to represent as transforms?

4. **Verification:** How strict should pre/postconditions be? Fail hard or warn?

5. **Performance:** At what repo size does the semantic index become a bottleneck?

6. **Adoption:** Should stead enforce transforms, or allow mixed workflows?

---

## Conclusion

The transformation layer is conceptually sound and addresses real problems with agent-driven development:

- **Git's diff model** doesn't capture semantic intent
- **Merge conflicts** are often false positives from a semantic perspective
- **Agent parallelism** needs better coordination than git branches
- **Traceability** from intent to implementation is valuable

The key insight is maintaining **dual representations**: semantic transforms for agents and intelligent operations, compiled patches for git compatibility.

Hard problems exist (language diversity, state drift, composition complexity) but are manageable with pragmatic fallbacks and gradual adoption.

**Recommended next step:** Build a minimal proof of concept for TypeScript rename/move transforms, compile to git commits, verify the model works before investing in full implementation.
