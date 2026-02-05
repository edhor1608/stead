# stead-ffi

UniFFI-based FFI bindings for stead-core, targeting Swift (macOS/iOS).

## Architecture

```
stead-core (Rust library)
    |
stead-ffi (UniFFI wrapper crate)
    |
    ├── libstead_ffi.dylib / .a  (compiled library)
    ├── stead_ffi.swift           (generated Swift bindings)
    ├── stead_ffiFFI.h            (C header)
    └── stead_ffiFFI.modulemap    (Clang module map)
```

## Exposed API

### Types

| Rust (stead-core)   | FFI Type             | Swift Type           |
|---------------------|----------------------|----------------------|
| `ContractStatus`    | `FfiContractStatus`  | `FfiContractStatus`  |
| `CliType`           | `FfiCliType`         | `FfiCliType`         |
| `Contract`          | `FfiContract`        | `FfiContract`        |
| `SessionSummary`    | `FfiSessionSummary`  | `FfiSessionSummary`  |

DateTime fields are exposed as ISO 8601 strings (`String` / `String?`).

### Functions

| Function | Signature |
|----------|-----------|
| `list_contracts` | `(cwd: String) throws -> [FfiContract]` |
| `get_contract` | `(id: String, cwd: String) throws -> FfiContract` |
| `list_sessions` | `(cliFilter: String?, project: String?, limit: UInt32) -> [FfiSessionSummary]` |

## Building

```bash
# Build the library
cargo build -p stead-ffi

# Generate Swift bindings
cargo run -p stead-ffi --bin uniffi-bindgen -- \
  generate --library target/debug/libstead_ffi.dylib \
  --language swift --out-dir stead-ffi/generated
```

## Using in Xcode

1. Add `libstead_ffi.a` (static lib) to your Xcode project
2. Add `stead_ffi.swift` to your Swift sources
3. Add `stead_ffiFFI.h` and `stead_ffiFFI.modulemap` to your project
4. Configure the modulemap in Build Settings > Swift Compiler > Import Paths

For release builds, build with:
```bash
cargo build -p stead-ffi --release --target aarch64-apple-darwin
```
