# Rust-Swift FFI Comparison for stead

**Date:** 2026-02-05
**Status:** Complete
**Purpose:** Evaluate FFI approaches for exposing stead-core (Rust) to Control Room (SwiftUI)

## What We Need to Bridge

From the current codebase, these are the types the Swift UI will consume:

**Structs:**
- `Contract` (6 fields, incl. `Option<String>`, `Option<DateTime>`)
- `SessionSummary` (8 fields, incl. `Option<String>`)
- `UniversalSession` (6 fields, nested structs, `Vec<TimelineEntry>`)
- Various nested structs: `SessionSource`, `ProjectInfo`, `GitInfo`, `ModelInfo`, etc.

**Enums:**
- `ContractStatus` (4 variants, fieldless)
- `CliType` (4 variants, fieldless)
- `TimelineEntry` (5 variants, each with struct data)
- `UniversalTool` (13 variants, fieldless)

**Return types:**
- `Vec<Contract>`, `Vec<SessionSummary>`
- `Option<Contract>`, `Option<UniversalSession>`
- `String`, `Option<String>`

**Not needed (initially):**
- Async FFI (monolith, no server, all calls are synchronous)
- Callbacks from Rust to Swift
- Multi-language bindings (Swift only)
- Mutable references across FFI boundary

---

## Option 1: swift-bridge

**Repository:** [chinedufn/swift-bridge](https://github.com/chinedufn/swift-bridge)
**Stars:** ~1,000 | **Latest:** v0.1.58 (Dec 2025) | **Maintainer:** Single (chinedufn)

### How It Works

You define a bridge module with `#[swift_bridge::bridge]` macro. At build time, `swift-bridge-build` (build.rs) or `swift-bridge-cli` parses bridge modules and generates Swift + C FFI code.

```rust
#[swift_bridge::bridge]
mod ffi {
    #[swift_bridge(swift_repr = "struct")]
    struct ContractSummary {
        id: String,
        task: String,
        status: String,
    }

    extern "Rust" {
        fn list_contracts() -> Vec<ContractSummary>;
        fn get_contract(id: String) -> Option<ContractSummary>;
    }
}
```

### Type Support

| Type | Support | Notes |
|------|---------|-------|
| String | Yes | Bridged as `RustString` / Swift `String` |
| &str | Yes | Bridged as `RustStr` |
| Vec<T> | Yes | Bridged as `RustVec<T>` |
| Option<T> | Yes | Maps to Swift Optional |
| Result<T,E> | Yes | Maps to Swift throws |
| Transparent structs | Yes | `swift_repr = "struct"` (copy-on-write) |
| Transparent enums | Yes | Tuple and struct variants supported |
| Opaque types | Yes | Heap-allocated, accessed by reference |
| Box<dyn FnOnce> | Yes | Boxed closures |
| Tuples | Yes | (A, B, C, ...) |
| HashMap | No | Not directly supported |
| DateTime | No | Must convert to String or i64 |
| serde_json::Value | No | Must serialize to String |

### Transparent Structs

Structs with `swift_repr = "struct"` are copy-on-write in Swift. Fields are directly accessible but immutable. For mutable access, use `swift_repr = "class"` (heap-allocated).

Limitation: all fields must be types swift-bridge knows how to bridge. Nested custom types require their own bridge declarations.

### Transparent Enums

Supports both fieldless enums and enums with tuple/struct variants:
```rust
enum BarCode {
    Upc(i32, i32, i32, i32),
    QrCode { code: String },
}
```

### Async Support

Supports `async fn` in both directions. Not needed for stead.

### Build Integration

Three options:
1. **Xcode + Cargo** - build.rs generates Swift files, Xcode project includes them
2. **swiftc + Cargo** - pure command-line, no Xcode project needed
3. **Swift Packages** - can be packaged as SwiftPM dependency

XCFramework creation is documented but requires manual scripting.

### Strengths

- **Swift-focused** - purpose-built for Rust <-> Swift, feels natural
- **Zero-overhead** - no serialization, direct memory layout bridging
- **Good type coverage** - String, Vec, Option, Result, structs, enums all work
- **Inspired by cxx** - familiar bridge module pattern
- **Swift 6 compatible** - MSSV is Swift 6.0

### Weaknesses

- **Single maintainer** - bus factor of 1 (chinedufn)
- **~1K stars** - smaller community, fewer Stack Overflow answers
- **Documentation gaps** - book is self-described as "work-in-progress"
- **No HashMap support** - must convert to Vec of tuples or use opaque types
- **All bridged types must be declared** - every struct/enum crossing FFI needs explicit bridge module declaration
- **DateTime handling** - chrono types must be manually converted (String or timestamp)

---

## Option 2: UniFFI (Mozilla)

**Repository:** [mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs)
**Stars:** ~4,100 | **Backed by:** Mozilla | **Used in:** Firefox (mobile + desktop)

### How It Works

Two approaches:

**Proc-macro (recommended):**
```rust
#[derive(uniffi::Record)]
pub struct ContractSummary {
    pub id: String,
    pub task: String,
    pub status: ContractStatusFFI,
}

#[derive(uniffi::Enum)]
pub enum ContractStatusFFI {
    Pending,
    Running,
    Passed,
    Failed,
}

#[uniffi::export]
fn list_contracts() -> Vec<ContractSummary> { ... }

#[uniffi::export]
fn get_contract(id: String) -> Option<ContractSummary> { ... }
```

**UDL (legacy, still supported):**
Define types in a `.udl` file, implement in Rust. More boilerplate, but language-agnostic definition.

The proc-macro approach is newer and eliminates the need for separate UDL files.

### Type Support

| Type | Support | Notes |
|------|---------|-------|
| String | Yes | Native mapping |
| Vec<T> | Yes | Maps to Swift Array |
| Option<T> | Yes | Maps to Swift Optional |
| Result<T,E> | Yes | Maps to Swift throws |
| Records (structs) | Yes | `#[derive(uniffi::Record)]` - maps to Swift struct |
| Enums | Yes | `#[derive(uniffi::Enum)]` - maps to Swift enum |
| Enums with data | Yes | Variants with named fields |
| Objects (classes) | Yes | `#[derive(uniffi::Object)]` - ref-counted, mutable |
| HashMap<K,V> | Yes | Maps to Swift Dictionary |
| Callbacks | Yes | Foreign trait implementations |
| Async | Yes | Maps to Swift async/await |
| DateTime | No | Convert to String or timestamp |
| serde_json::Value | No | Must serialize to String |
| Custom types | Yes | Via `uniffi::custom_type!` macro for newtype wrappers |

### Records vs Objects

- **Records** (`uniffi::Record`): Value types. All fields public. Maps to Swift `struct`. Passed by value (copied).
- **Objects** (`uniffi::Object`): Reference types. Arc-wrapped. Maps to Swift `class`. Methods via `#[uniffi::export]` on `impl` blocks. Suitable for stateful services.

### Multi-Language Support

Generates bindings for:
- Swift (production quality)
- Kotlin (production quality)
- Python (production quality)
- Ruby (experimental)
- C# (3rd party)
- Go (3rd party)

Not needed for stead now, but de-risks future Linux/Windows native UIs.

### Build Integration

- `uniffi-bindgen-swift` CLI generates Swift files + modulemap + C headers
- [cargo-swift](https://github.com/antoniusnaumann/cargo-swift) (264 stars) automates XCFramework + Swift Package generation
- `cargo swift package` produces a ready-to-use Swift Package from Rust code

### Strengths

- **Mozilla-backed** - production-tested in Firefox, real funding
- **4x larger community** - 4.1K stars, many contributors
- **Better documentation** - comprehensive user guide
- **cargo-swift** - one-command XCFramework + Swift Package generation
- **Richer type support** - HashMap, custom types, foreign traits
- **Proc-macro approach** - annotate Rust types directly, no separate bridge file
- **Multi-language option** - could target other platforms later
- **Battle-tested** - ships to millions of Firefox users

### Weaknesses

- **Not yet 1.0** - API may change between minor versions
- **Slightly more overhead** - uses serialization for some types (vs swift-bridge's direct layout)
- **More dependencies** - larger dependency tree
- **Proc-macro is newer** - some edge cases vs mature UDL path
- **Swift 6 partial** - async code doesn't fully conform to Sendable yet

---

## Option 3: Manual C FFI (cbindgen)

**Approach:** `#[no_mangle] pub extern "C" fn` + cbindgen for header generation + manual Swift wrappers

### How It Works

```rust
// Rust side
#[repr(C)]
pub struct FFIContractSummary {
    pub id: *const c_char,
    pub task: *const c_char,
    pub status: u8,
}

#[no_mangle]
pub extern "C" fn stead_list_contracts(out_len: *mut usize) -> *mut FFIContractSummary {
    // allocate, fill, return pointer
}

#[no_mangle]
pub extern "C" fn stead_free_contracts(ptr: *mut FFIContractSummary, len: usize) {
    // deallocate
}
```

```swift
// Swift side (manual wrapper)
class SteadBridge {
    func listContracts() -> [ContractSummary] {
        var len: Int = 0
        let ptr = stead_list_contracts(&len)
        defer { stead_free_contracts(ptr, len) }
        // convert C structs to Swift structs...
    }
}
```

cbindgen auto-generates the C header from `#[repr(C)]` types and `extern "C"` functions.

### Type Support

Everything is possible but everything is manual:
- Strings: `*const c_char` + manual allocation/deallocation
- Vectors: pointer + length pairs, manual memory management
- Options: sentinel values or nullable pointers
- Enums: integer tags + manual mapping
- Structs: `#[repr(C)]` with only C-compatible field types

### Strengths

- **Maximum control** - exact memory layout, zero abstraction overhead
- **Zero dependencies** - no third-party build tools
- **Stable** - C ABI never changes
- **Educational** - understand exactly what crosses the boundary

### Weaknesses

- **Enormous boilerplate** - every type needs: `#[repr(C)]` struct, conversion functions, free functions, Swift wrapper
- **Memory safety burden** - manual allocation/deallocation, dangling pointers, use-after-free
- **No Vec/String/Option mapping** - all manual with raw pointers
- **Maintenance nightmare** - every Rust type change requires updating 3 places (Rust FFI, C header, Swift wrapper)
- **Error-prone** - one mismatched pointer type = undefined behavior

For stead's ~15 types with nested structs and enums, manual C FFI would require hundreds of lines of unsafe boilerplate with no tooling to catch mistakes.

---

## Comparison Matrix

| Criteria | swift-bridge | UniFFI | Manual C FFI |
|----------|-------------|--------|-------------|
| **Setup complexity** | Medium (build.rs) | Medium (proc-macro + bindgen) | High (cbindgen + manual wrappers) |
| **String** | Yes (RustString) | Yes (native) | Manual (*const c_char) |
| **Vec<T>** | Yes (RustVec) | Yes (Array) | Manual (ptr + len) |
| **Option<T>** | Yes (Optional) | Yes (Optional) | Manual (sentinel/nullable) |
| **Structs** | Yes (transparent) | Yes (Record) | Manual (#[repr(C)]) |
| **Enums (fieldless)** | Yes | Yes | Manual (u8 tag) |
| **Enums (with data)** | Yes | Yes | Manual (tagged union) |
| **HashMap** | No | Yes | Manual |
| **Nested types** | Yes (each declared) | Yes (each annotated) | Manual |
| **XCFramework** | Manual scripting | cargo-swift automation | Manual scripting |
| **Community** | ~1K stars, 1 maintainer | ~4.1K stars, Mozilla-backed | N/A |
| **Documentation** | Incomplete book | Comprehensive guide | Scattered |
| **Boilerplate** | Low (bridge module) | Low (proc-macros) | Very high |
| **Performance** | Best (zero-copy) | Good (minimal overhead) | Best (if done right) |
| **Safety** | Safe (generated code) | Safe (generated code) | Unsafe (manual) |
| **Multi-language** | Swift only | Swift, Kotlin, Python, Ruby | Any (C ABI) |

---

## Recommendation for stead

**UniFFI** is the clear winner for this project.

### Rationale

1. **Type coverage matches our needs exactly.** We need structs (Contract, SessionSummary), fieldless enums (ContractStatus, CliType), Vec returns, and Option types. UniFFI handles all of these with simple proc-macro annotations. swift-bridge also handles these, but UniFFI additionally supports HashMap (useful if ModelInfo.config needs bridging later).

2. **cargo-swift eliminates XCFramework pain.** Running `cargo swift package` produces a ready-to-use Swift Package that Xcode can consume directly. swift-bridge requires manual scripting for XCFramework generation.

3. **Mozilla backing reduces risk.** UniFFI is production-tested in Firefox, has 4x the community, and multiple maintainers. swift-bridge's single-maintainer risk is real for a project we'll depend on long-term.

4. **Proc-macro approach is clean.** Annotating existing Rust types with `#[derive(uniffi::Record)]` or `#[derive(uniffi::Enum)]` is less intrusive than writing separate bridge modules. The FFI types live close to the real types.

5. **Future optionality.** If stead ever targets Linux or Windows with native UIs, UniFFI can generate Kotlin/Python bindings from the same Rust code. swift-bridge is Swift-only.

6. **We don't need swift-bridge's edge.** swift-bridge's zero-copy advantage matters for high-frequency, latency-sensitive FFI calls. stead's Control Room will make infrequent calls (list contracts, get session) where the minimal serialization overhead of UniFFI is irrelevant.

### What we give up

- Slightly more overhead per FFI call (negligible for our use case)
- Pre-1.0 API (mitigated by cargo-swift pinning specific UniFFI versions)

### Implementation Plan

For M4 (Swift FFI Bindings), the approach would be:

```rust
// stead-ffi/src/lib.rs
uniffi::setup_scaffolding!();

#[derive(uniffi::Record)]
pub struct ContractSummary {
    pub id: String,
    pub task: String,
    pub verification: String,
    pub status: ContractStatusFFI,
    pub created_at: String,       // ISO 8601 string (chrono -> String)
    pub completed_at: Option<String>,
    pub output: Option<String>,
}

#[derive(uniffi::Enum)]
pub enum ContractStatusFFI {
    Pending,
    Running,
    Passed,
    Failed,
}

#[uniffi::export]
fn list_contracts() -> Vec<ContractSummary> {
    // call stead_core, convert types
}

#[uniffi::export]
fn get_contract(id: String) -> Option<ContractSummary> {
    // call stead_core, convert type
}
```

Then: `cargo swift package` to generate the Swift Package for Xcode.

### DateTime Handling

Neither tool natively bridges chrono types. The pragmatic solution:
- Bridge `DateTime<Utc>` as ISO 8601 `String` (e.g., `"2026-02-05T12:00:00Z"`)
- Parse in Swift with `ISO8601DateFormatter`
- Alternatively, bridge as `i64` (Unix timestamp millis) and convert in Swift with `Date(timeIntervalSince1970:)`

The String approach is safer (no precision loss) and easier to debug.
