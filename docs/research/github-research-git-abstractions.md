# GitHub Research: Git Abstractions & Transformation Tools

Date: 2026-02-02

Research for stead's "transformation layer above git" — where changes are expressed as transformations (not diffs) that compile to git commits.

---

## 1. Semantic Diff Tools (AST-Aware)

### Difftastic
**Repo:** https://github.com/Wilfred/difftastic

**What it does:** Structural diff tool that compares files based on syntax. Understands exactly which pieces of syntax have changed and highlights them in context. Knows when whitespace matters vs. when it's just indentation.

**What we can learn:**
- Uses tree-sitter for parsing (30+ languages supported)
- Git-compatible — can be used as `git difftool`
- Shows "unchanged but moved" code differently from "changed"

**Limitations:** Read-only diff visualization. Doesn't generate transformations, just better diffs.

---

### Diffsitter
**Repo:** https://github.com/afnanenayet/diffsitter

**What it does:** Creates semantically meaningful diffs that ignore formatting. Computes diff on AST rather than text content.

**What we can learn:**
- Pure AST-to-AST diffing approach
- Language support tied to tree-sitter grammars

**Limitations:** Same as difftastic — visualization only.

---

### GumTree
**Repo:** https://github.com/GumTreeDiff/gumtree

**What it does:** Syntax-aware diff with move detection. Research-backed (ICSE paper "Fine-grained and Accurate Source Code Differencing").

**What we can learn:**
- Edit actions are always syntax-aligned
- Detects moved/renamed elements (not just insert/delete)
- Can export changes as structured data (XML, N-Triples)
- Has `gumtree-spoon-ast-diff` for Java-specific analysis

**Limitations:** Java-focused. Research shows 20-29% inaccurate mappings. Complex to integrate.

---

### GumTree-Spoon-AST-Diff
**Repo:** https://github.com/SpoonLabs/gumtree-spoon-ast-diff

**What it does:** Computes AST difference between two Spoon Java ASTs. Fine-tuned with hyper-optimization.

**What we can learn:**
- AST diff tuned specifically for one language produces better results
- Integration with transformation libraries (Spoon)

**Limitations:** Java only.

---

### SemanticDiff
**URL:** https://semanticdiff.com/

**What it does:** VS Code extension and GitHub integration. Distinguishes relevant vs irrelevant changes. Hides whitespace/formatting noise, highlights refactorings.

**What we can learn:**
- Product validation: people want semantic diffs enough to pay
- GitHub integration is possible (they did it)

**Limitations:** Commercial, closed source. Can't fork or extend.

---

## 2. Code Transformation Frameworks

### jscodeshift (Facebook)
**Repo:** https://github.com/facebook/jscodeshift

**What it does:** JavaScript codemod toolkit. Runner that executes transforms across multiple files. Wraps recast for AST-to-AST transformation while preserving style.

**What we can learn:**
- jQuery-like fluent API for AST traversal: `.find().replaceWith().toSource()`
- Preserves original formatting where possible (via recast)
- Parallel execution across files — scales to large codebases
- The "codemod" pattern: define a transform, run it everywhere

**Key insight for stead:** Transformations are first-class. You don't describe "what changed" — you describe "the operation".

**Limitations:** JavaScript/TypeScript only.

---

### recast
**Repo:** https://github.com/benjamn/recast

**What it does:** JavaScript AST transformer with **nondestructive pretty-printing**. Key property: `recast.print(recast.parse(source)).code === source`

**What we can learn:**
- Shadow copy of AST with `.original` references enables knowing what changed
- Only reprints modified parts
- Falls back to pretty printer when it can't preserve formatting

**Key insight for stead:** The identity guarantee is powerful. You can transform code and get a minimal diff.

**Limitations:** JavaScript only. Pretty printer can trigger unexpectedly.

---

### LibCST (Instagram)
**Repo:** https://github.com/Instagram/LibCST

**What it does:** Python CST (Concrete Syntax Tree) parser that preserves formatting. Lossless representation including comments, whitespace, parentheses.

**What we can learn:**
- Visitor pattern with `visit_*` (read-only) and `leave_*` (can modify) methods
- Used by Google Cloud libraries for major version migration codemods
- Battle-tested at scale (SeatGeek, Instawork use cases documented)

**Key insight for stead:** CST is better than AST for transformations that need to preserve human style.

**Limitations:** Python only.

---

### ts-morph
**Repo:** https://github.com/dsherret/ts-morph

**What it does:** TypeScript Compiler API wrapper for static analysis and programmatic code changes.

**What we can learn:**
- Rich navigation API (`.getClasses()`, `.getModules()`, etc.)
- `transform` method for recursive AST modification
- ts-morph-playground for prototyping transforms in browser

**Limitations:** TypeScript/JavaScript only.

---

### Spoon (INRIA)
**Repo:** https://github.com/INRIA/spoon

**What it does:** Java metaprogramming library for AST analysis and transformation. Supports Java up to version 20.

**What we can learn:**
- Design principle: "metamodel close to language concepts"
- Complete and sound model — text version is semantically equivalent
- Processor pattern for composable transformations

**Limitations:** Java only.

---

### ast-grep
**Repo:** https://github.com/ast-grep/ast-grep

**What it does:** CLI for code structural search, lint, and rewriting. Uses tree-sitter. Patterns look like code (isomorphic).

**What we can learn:**
- Pattern matching syntax is intuitive: write code-like patterns
- YAML config for rules — no programming required for simple transforms
- Rust + multi-core = very fast

**Key insight for stead:** Patterns-as-code is more approachable than AST manipulation.

**Limitations:** Pattern matching, not general transformation. No inter-file analysis.

---

### Comby
**Repo:** https://github.com/comby-tools/comby

**What it does:** Language-agnostic structural search and replace. Understands blocks, strings, comments.

**What we can learn:**
- Works on ANY language by understanding delimiters, not grammar
- Template syntax: `if (:[condition])` matches any condition
- Interactive review mode for safe application

**Key insight for stead:** You don't always need full AST. Structural matching is often enough.

**Limitations:** Limited semantic awareness. No type information.

---

### Semgrep
**Repo:** https://github.com/semgrep/semgrep

**What it does:** Semantic grep + autofix. Patterns look like code. Rule-based with YAML config.

**What we can learn:**
- Autofix via AST manipulation, not text replacement
- Constant propagation and semantic equivalence matching
- 2000+ community rules — proves the model works at scale
- Registry/marketplace model

**Limitations:** Focused on security/linting. Autofix is rule-scoped, not arbitrary transformation.

---

### Putout
**Repo:** https://github.com/coderaiser/putout

**What it does:** Pluggable JavaScript linter + transformer. "ESLint superpower replacement." Declarative codemods.

**What we can learn:**
- Plugin architecture for extensibility
- Babel AST based
- Combines linting and transformation in one tool

**Limitations:** JavaScript ecosystem only.

---

## 3. Git Abstraction Layers & Alternative VCS

### Jujutsu (jj)
**Repo:** https://github.com/jj-vcs/jj

**What it does:** Git-compatible VCS. Abstraction layer over git backend. On track to replace Google's internal VCS.

**Key innovations:**
- Every operation on repo is recorded and can be undone
- Modify a commit → all descendants auto-rebase
- Conflict resolution propagates through descendants
- No staging area — working copy is always a commit
- Branches are just bookmarks, not first-class

**What we can learn:**
- Git as storage backend works — you get all interop for free
- Operations can be first-class (not just commits)
- Undo at operation level is powerful
- "Working copy is always a commit" simplifies mental model

**Key insight for stead:** Jujutsu proves you can build a better UX on top of git without replacing it.

**Limitations:** Still commit-based, not transformation-based. No semantic awareness.

---

### Sapling (Meta)
**URL:** https://sapling-scm.com/

**What it does:** Meta's fork of Mercurial with Git backend. Stacked diffs as first-class workflow.

**Key innovations:**
- Sparse checkouts for massive monorepos
- Interactive tooling purpose-built for scale
- Mature (production at Meta for years)

**What we can learn:**
- Stacked diffs workflow is battle-tested at scale
- Git interop means teammates can use stock git
- UX can be dramatically different while keeping git as backend

**Limitations:** Doesn't address semantic transformations.

---

### Pijul
**URL:** https://pijul.org/

**What it does:** True commutative patches. Order of applying patches doesn't matter. Conflicts "travel with" patches.

**Key innovations:**
- Patch commutation is mathematically principled
- Conflict resolution is recorded, not repeated
- Rebasing brittleness eliminated by design

**What we can learn:**
- Theoretical foundation matters for correctness
- Pijul's "patches as first-class with commutation" is the closest to "transformations"
- The interop story being weak is why adoption suffers

**Key insight for stead:** Pijul's theory is closest to what we want. Implementation/adoption is the problem.

**Limitations:** Limited ecosystem. Git interop is early.

---

### Darcs
**URL:** https://darcs.net/

**What it does:** Patch-based VCS. Patches can be reordered (commuted). Repository is a partially-ordered set of patches.

**What we can learn:**
- Original patch theory implementation
- Patches as first-class, not commits
- Commutation enables cherry-picking without history

**Key insight for stead:** Darcs proved the patch model 20 years ago. Failed on performance, not concepts.

**Limitations:** Performance issues at scale. Haskell makes contribution hard.

---

### gitoxide
**Repo:** https://github.com/GitoxideLabs/gitoxide

**What it does:** Pure Rust implementation of git. Idiomatic, fast, safe.

**What we can learn:**
- Clean Rust API for git operations
- Modular crates (gix-config, gix-object, etc.)
- Could be a foundation for building git-compatible tooling

**Limitations:** Aims for git compatibility, not new abstractions.

---

### GitButler
**Repo:** https://github.com/gitbutlerapp/gitbutler

**What it does:** Virtual branches — multiple branches applied to working directory simultaneously. Drag changes between branch lanes.

**Key innovations:**
- Work on multiple features in parallel without switching
- Assigns hunks to virtual branches, commits only owned hunks
- Since you start from a merged state, all branches merge cleanly by construction
- `gitbutler/workspace` branch holds the union of all applied branches

**What we can learn:**
- Virtual branches = working backwards from desired merge state
- Hunk ownership is tracked, not just file ownership
- Can push to GitHub even though exact tree never existed on disk

**Key insight for stead:** "Start from the merge, extract branches" is inverse of normal git mental model.

**Limitations:** Still diff-based, not transformation-based.

---

## 4. Stacked Diffs / Patch Workflows

### git-branchless
**Repo:** https://github.com/arxanas/git-branchless

**What it does:** High-velocity workflow for git. Stacked diffs, automatic descendant rebase, undo at operation level.

**What we can learn:**
- Changeset evolution (borrowed from Mercurial)
- Discourages branches/stashes in favor of commits for everything
- `git next` / `git prev` for stack navigation

**Limitations:** Still commit-based.

---

### ghstack
**Repo:** https://github.com/ezyang/ghstack

**What it does:** Submit stacks of diffs to GitHub as separate PRs.

**What we can learn:**
- Each commit becomes a separate PR
- Automation around GitHub PR model

**Limitations:** GitHub-specific. Workflow tool, not abstraction.

---

### Graphite
**URL:** https://graphite.dev/

**What it does:** Stacked PRs for GitHub. CLI + web dashboard.

**What we can learn:**
- `gt modify` — edit any branch in stack, auto-restack above
- Merging stacked PRs in sequence
- Product traction proves demand for stacked workflow

**Limitations:** Proprietary. GitHub-only.

---

### git-absorb
**Repo:** https://github.com/tummychow/git-absorb

**What it does:** `git commit --fixup` but automatic. Figures out which commit each hunk should be fixed up into.

**What we can learn:**
- Uses patch commutation to find the right parent commit
- Port of Facebook's `hg absorb`

**Key insight for stead:** Patch commutation enables automatic placement of changes.

**Limitations:** For fixups only, not general transformation.

---

## 5. Merge Conflict Resolution

### Git rerere
**Built into git:** `git config rerere.enabled true`

**What it does:** Reuse Recorded Resolution. Remembers how you resolved conflicts and applies same resolution next time.

**What we can learn:**
- Conflict resolution CAN be automated if it's been done before
- Three-way merge between (old conflict, old resolution, new conflict)

**Key insight for stead:** Resolution history is valuable. Don't throw it away.

**Limitations:** Only works for identical conflicts. No semantic awareness.

---

### Syncwright
**Repo:** https://github.com/NeuBlink/syncwright

**What it does:** AI-powered conflict resolution. CLI + GitHub Action. Uses Claude for context-aware resolution.

**What we can learn:**
- LLM can resolve conflicts with semantic understanding
- Confidence scoring for when to apply vs. ask human

**Limitations:** Depends on LLM quality. Not deterministic.

---

## 6. Refactoring Detection

### RefactoringMiner
**Repo:** https://github.com/tsantalis/RefactoringMiner

**What it does:** Detects refactorings in Java git history. Since v3, also generates AST diffs.

**What we can learn:**
- Refactorings can be extracted from commits after the fact
- Web viewer for AST diffs overlaid with refactoring info
- Chrome extension for GitHub commit view

**Key insight for stead:** If you can detect refactorings post-hoc, you can maybe express them declaratively.

**Limitations:** Java only. Detection, not transformation.

---

## 7. Operational Transformation

### JOT (JSON Operational Transformation)
**Repo:** https://github.com/JoshData/jot

**What it does:** OT on JSON data model. Operations know how to rebase against each other.

**What we can learn:**
- "Rebase" is transformation-aware: new operation is created
- Not all rebases are possible without human intervention (= merge conflict)
- Operations act like git merge, not git rebase (history preserved)

**Key insight for stead:** OT theory has solved "how operations compose" mathematically.

**Limitations:** JSON data model, not code.

---

## 8. Structured Patch Formats

### Google diff-match-patch
**Repo:** https://github.com/google/diff-match-patch

**What it does:** High-performance diff library. Multiple languages.

**What we can learn:**
- Fuzzy patching — can apply patch even if context has shifted
- Semantic cleanup to make diffs human-readable

**Limitations:** Text-based, not semantic.

---

### patch-package
**Repo:** https://github.com/ds300/patch-package

**What it does:** Patches to npm dependencies stored as diff files. Auto-applied on install.

**What we can learn:**
- Patches as first-class artifacts (stored in repo)
- `postinstall` hook applies patches automatically

**Key insight for stead:** Patches can be versioned and shared. Not just transient.

**Limitations:** npm-specific. Text diffs.

---

## Summary: Key Insights for Stead

### 1. Transformation Representation

Multiple valid approaches:
- **AST manipulation** (jscodeshift, LibCST, Spoon) — most precise, language-specific
- **Structural patterns** (Comby, Semgrep) — language-agnostic but less precise
- **Patch theory** (Pijul, Darcs) — mathematical foundation, poor adoption

**Recommendation:** Start with structural patterns (Comby-style) for MVF. Add AST support per language later.

### 2. Git Compatibility is Non-Negotiable

Jujutsu and Sapling prove: you can build dramatically different UX while using git as storage backend. Pijul's weak interop is why it failed to gain adoption.

**Recommendation:** Compile to git commits. Never break `git status`, `git log`, `git push`.

### 3. Commutation is the Key Concept

From Pijul, Darcs, git-absorb: if you know when patches commute, you can:
- Reorder without conflicts
- Auto-place fixups
- Cherry-pick cleanly

**Recommendation:** Track which transformations commute. Use this for conflict resolution.

### 4. Conflict Resolution Can Be Semantic

rerere + Syncwright show: conflicts don't have to be manual. If you express changes as transformations (not diffs), "re-run the transformation" is a valid conflict resolution strategy.

**Recommendation:** Transformations are re-executable. On conflict, re-run against new base.

### 5. Virtual Branches Are Interesting

GitButler's "start from merge, extract branches" is the inverse of normal git. Closer to how agents might work: make all the changes, then organize them into commits.

**Recommendation:** Consider "workspace → transformation extraction → commits" flow.

### 6. Stacked Diffs Have Market Validation

Graphite, ghstack, git-branchless all exist because stacked diffs are better for review. Agents produce small, atomic changes naturally.

**Recommendation:** First-class support for stacked transformations that become stacked PRs.

---

## Tools to Investigate Further

| Tool | Why |
|------|-----|
| Jujutsu | Closest to "modern git" — study their operation model |
| Pijul | Closest to "transformation theory" — study their math |
| GitButler | Closest to "virtual work extraction" — study their hunk ownership |
| Comby | Closest to "language-agnostic transformation" — potential integration |
| tree-sitter | Foundation for any AST work — difftastic/ast-grep both use it |

---

## References

- [Difftastic](https://github.com/Wilfred/difftastic)
- [GumTree](https://github.com/GumTreeDiff/gumtree)
- [jscodeshift](https://github.com/facebook/jscodeshift)
- [recast](https://github.com/benjamn/recast)
- [LibCST](https://github.com/Instagram/LibCST)
- [ts-morph](https://github.com/dsherret/ts-morph)
- [Spoon](https://github.com/INRIA/spoon)
- [ast-grep](https://github.com/ast-grep/ast-grep)
- [Comby](https://github.com/comby-tools/comby)
- [Semgrep](https://github.com/semgrep/semgrep)
- [Jujutsu](https://github.com/jj-vcs/jj)
- [Sapling](https://sapling-scm.com/)
- [Pijul](https://pijul.org/)
- [gitoxide](https://github.com/GitoxideLabs/gitoxide)
- [GitButler](https://github.com/gitbutlerapp/gitbutler)
- [git-branchless](https://github.com/arxanas/git-branchless)
- [ghstack](https://github.com/ezyang/ghstack)
- [Graphite](https://graphite.dev/)
- [git-absorb](https://github.com/tummychow/git-absorb)
- [RefactoringMiner](https://github.com/tsantalis/RefactoringMiner)
- [JOT](https://github.com/JoshData/jot)
- [patch-package](https://github.com/ds300/patch-package)
