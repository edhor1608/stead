# Architecture Review: stead Rust CLI v0.2.0

**Reviewer:** Linus Torvalds (simulated)
**Date:** 2026-02-05
**Scope:** All files in `rust/src/`, `rust/tests/`, `rust/Cargo.toml`

---

## 1. Module Structure

### Layout

```
src/
  main.rs           # Entry point, CLI dispatch
  lib.rs            # Pub re-exports: cli, commands, schema, storage, usf
  cli/mod.rs         # Clap argument parsing
  commands/
    mod.rs           # Barrel file
    list.rs          # List contracts
    run.rs           # Create + execute contract
    show.rs          # Show contract detail
    verify.rs        # Re-run verification
    session.rs       # Session list/show
  schema/
    mod.rs           # Barrel file
    contract.rs      # Contract type + ID gen
  storage/
    mod.rs           # Barrel file
    jsonl.rs         # JSONL read/write
  usf/
    mod.rs           # Barrel file
    schema.rs        # USF types (UniversalSession, timeline, tools)
    adapters/
      mod.rs         # SessionAdapter trait + discovery
      claude.rs      # Claude Code adapter
      codex.rs       # Codex CLI adapter
      opencode.rs    # OpenCode adapter
```

**Verdict: Acceptable structure, minor issues.**

The module tree is clean. Each module has a single, clear responsibility. No circular dependencies. The `schema/` and `storage/` modules are correctly separated. The `usf/` subtree is well-isolated.

**Issues:**
- `schema/mod.rs` and `storage/mod.rs` are pure barrel files (one line each: `mod x; pub use x::*;`). This is a smell. If there's only one file in the module, just name the file `schema.rs` and `storage.rs`. The extra directory with `mod.rs` + single file is pointless indirection. Same applies to `usf/mod.rs`.
- `commands/mod.rs` is a pure barrel file. Fine for now since there are 5 submodules.

## 2. Abstraction Quality

### The Good

**Contract type** (`schema/contract.rs`): Clean, simple state machine. `Pending -> Running -> Passed|Failed`. Methods are `new()`, `start()`, `complete()`. No over-engineering. I'd approve this.

**StorageError** (`storage/jsonl.rs`): Well-defined error enum with `thiserror`. `PermissionDenied`, `Io`, `Json`, `NotFound`. Each variant is meaningful and maps to a real failure mode.

**SessionAdapter trait** (`usf/adapters/mod.rs`): Reasonable. Four methods: `cli_type()`, `is_available()`, `base_dir()`, `list_sessions()`, `load_session()`. Simple interface, easy to implement. Not overdesigned.

### The Bad

**`run_verification()` is duplicated.** Exact same function exists in both `commands/run.rs:98-127` and `commands/verify.rs:60-89`. Character for character identical. This is a copy-paste job that should be a shared function.

**`truncate()` is duplicated FOUR times.** The same truncation logic exists in:
- `commands/list.rs:73-81` (char-based, different from others)
- `commands/session.rs:288-296` (byte-based)
- `usf/adapters/claude.rs:416-424` (byte-based)
- `usf/adapters/codex.rs:473-481` (byte-based)
- `usf/adapters/opencode.rs:407-415` (byte-based)

Five implementations of the same thing. And worse: the one in `list.rs` does it correctly using `.chars()` for UTF-8 safety, while the other four use byte slicing (`&first_line[..max_len - 3]`) which will **panic on multi-byte UTF-8 characters**. So the duplicates are not only redundant, they're buggy.

**`generate_id()` is duplicated.** Two separate implementations:
- `schema/contract.rs:93-104` (base36 timestamp + random)
- `usf/schema.rs:409-416` (hex timestamp only)

Different algorithms, same purpose. The one in `contract.rs` is better (adds randomness). The one in `usf/schema.rs` has no randomness and will produce collisions within the same millisecond.

### UniversalTool mapping

The `from_claude()`, `from_codex()`, `from_opencode()` methods on `UniversalTool` are fine as static match tables. Simple, explicit, no magic. Good.

## 3. API Surface

`lib.rs` exposes five modules: `cli`, `commands`, `schema`, `storage`, `usf`. All public.

**Problems for library consumers:**

- **Everything is public.** `cli` contains `Cli` and `Commands` which are clap derive types. A library consumer doesn't need these. The `commands` module contains functions that println to stdout directly. That's not a library API, that's a CLI implementation detail. You can't use `commands::list::execute()` as a library because it prints to stdout and returns `()`.

- **No separation between library API and CLI internals.** If this is meant to become a library (`stead-core`), the current `lib.rs` is unusable as-is. The useful library types are `Contract`, `ContractStatus`, the storage functions, and the USF types. The `cli` and `commands` modules should not be part of the library surface.

- **Command functions mix I/O with logic.** Every command function takes a `json_output: bool` and does its own formatting. The business logic (create contract, run verification, list contracts) is entangled with presentation (println, JSON formatting). A library should return data; the CLI should format it.

**What a clean lib.rs should look like:**
```rust
pub mod schema;    // Contract, ContractStatus
pub mod storage;   // JSONL read/write
pub mod usf;       // Universal Session Format types + adapters
```

The `cli` and `commands` modules belong exclusively in the binary crate.

## 4. Dependencies

```toml
clap = { version = "4", features = ["derive"] }     # CLI parsing - fine
serde = { version = "1", features = ["derive"] }     # Serialization - required
serde_json = "1"                                       # JSON - required
chrono = { version = "0.4", features = ["serde"] }   # Time - fine
thiserror = "2"                                        # Error types - fine
anyhow = "1"                                           # Error handling - fine
tokio = { version = "1", features = ["full"] }        # Async runtime - WRONG
dirs = "5"                                             # Home dir - fine
```

**tokio with `features = ["full"]` for a synchronous CLI tool.**

There is not a single `async fn` in this entire codebase. Not one `await`. Not one `.await`. The comment says "for future HTTP API". You don't add dependencies for future features. You add them when you need them. `tokio` with `features = ["full"]` pulls in the entire async runtime, signal handlers, io drivers, timers - for NOTHING.

This adds compile time, binary size, and complexity for zero benefit. Remove it.

**Versions are not pinned.** Using `"1"`, `"4"`, `"0.4"`, etc. This is fine for an application (Cargo.lock pins them). Would be a problem for a library.

## 5. Code Quality

### Naming

Good. `Contract`, `ContractStatus`, `UniversalSession`, `SessionAdapter`, `ClaudeAdapter`. Clear, descriptive, Rust-idiomatic. No Hungarian notation, no abbreviations (except `usf` which is defined as "Universal Session Format"). Method names are verbs: `execute`, `list_sessions`, `load_session`. Fields are descriptive.

### Error Handling

Mixed. The codebase uses both `thiserror` (for `StorageError`, `AdapterError`) and `anyhow` (in commands). This is actually the correct pattern: typed errors for library code, `anyhow` for application code. Good.

However, some error handling is sloppy:
- `session.rs:62` swallows the error: `Err(_) => { eprintln!(...) }` then returns `Ok(())`. A command that can't find a session should return an error, not silently succeed.
- `show.rs:38-44` has inconsistent behavior: in JSON mode, errors are printed as JSON and return `Ok(())`; in normal mode, they `bail!()`. Pick one approach.
- `session.rs:20-28` prints to stderr for invalid CLI filter and returns `Ok(())`. This should be an error.

### Memory Leak

`session.rs:218`:
```rust
.unwrap_or_else(|| format!("{:?}", call.tool).leak());
```

This calls `.leak()` on a `String`, converting it to `&'static str`. Every time a tool call is displayed without an original_tool, this leaks memory. It's a small amount per call, but it's a genuine memory leak. Use a local owned string instead.

### Comments

Mostly doc comments at the module level. Good. No comments explaining obvious code. A few "// Claude doesn't have explicit error flag" comments that explain external behavior - those are fine. No TODO spam, no commented-out code.

### Test Quality

**Unit tests:** Present in every module. The contract lifecycle test, serialization round-trips, status parsing, truncation tests - all solid. Good edge case coverage (UTF-8 in truncation, empty inputs, corrupted JSONL).

**Integration tests:** Cover the full CLI: help, version, list empty, run+list, failing verification, show, verify, JSON output, status filtering, session commands. Good.

**Missing:** No tests for the adapter parsing logic with actual session files. The Claude/Codex/OpenCode adapter tests are thin - they test deserialization of individual structs but not the actual `parse_session_file` flow. You'd need fixture files for proper testing.

## 6. Verdict

### REJECT

1. **tokio dependency.** Remove it. Zero async code, full runtime compiled in. This is dead weight. Unacceptable.

2. **`run_verification()` duplication.** Move to a shared location. Copy-paste is a maintenance bug waiting to happen.

3. **`truncate()` x5 with a UTF-8 bug in 4 of them.** Fix the byte-slicing versions to use `.chars()` or `char_indices()`. Then consolidate into one function in a shared utils module or keep one per crate boundary.

4. **Memory leak in `session.rs:218`.** `.leak()` on a format string is never acceptable in application code.

5. **Library API doesn't exist.** `lib.rs` exports CLI implementation details. Commands print to stdout. If M2 is about restructuring to lib+CLI, this is the primary architectural problem.

### ACCEPT

1. **Module structure** is clean and logical. No circular dependencies, clear responsibilities.

2. **Contract type** is well-designed. Simple state machine, proper serialization, good tests.

3. **Storage layer** is solid. Append-only JSONL, graceful corruption handling, proper error types.

4. **USF schema design** is thoughtful. The timeline model with typed entries, tool normalization across CLIs, and the adapter trait are well-done.

5. **Error handling pattern** (thiserror for library, anyhow for commands) is correct.

6. **Test coverage** is good for the core functionality. Integration tests cover the full CLI surface.

7. **No unnecessary abstractions.** No traits where functions suffice. No generics for the sake of generics. No `AbstractFactoryFactory`.

### Priority Fixes for M2 Restructuring

1. **Split into workspace: stead-core (lib) + stead-cli (bin).** Move `schema`, `storage`, `usf` to core. Move `cli`, `commands` to the binary. This is the whole point of M2.

2. **Decouple commands from I/O.** Command functions should return data, not print. The CLI layer formats output. This makes the library actually usable.

3. **Remove tokio.** Just delete the line. If you need async later, add it later.

4. **Fix the `truncate` duplication and UTF-8 bug.** One correct implementation, shared appropriately.

5. **Fix the `.leak()` memory leak.** Use an owned String.

6. **Extract `run_verification()` to shared code.** Both `run.rs` and `verify.rs` need it.

7. **Clean up barrel-file-only modules.** `schema/mod.rs` with a single `pub use contract::*` is pointless. Either inline or flatten.

---

*The code is honest, straightforward, and mostly well-structured. It doesn't pretend to be something it's not. The problems are real but fixable. The biggest architectural issue - everything in one crate with commands printing to stdout - is exactly what M2 should address. The codebase is a good starting point, not a rewrite candidate.*
