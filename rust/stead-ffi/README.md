# stead-ffi

UniFFI-based Rust <-> Swift bridge for the rewrite surface.

## Architecture

```text
stead-daemon + stead-contracts + stead-usf
    |
stead-ffi (UniFFI wrapper crate)
    |
    ├── libstead_ffi.dylib / .a
    ├── stead_ffi.swift
    ├── stead_ffiFFI.h
    └── stead_ffiFFI.modulemap
```

## Exposed API

### Types

| Rust | FFI | Swift |
| --- | --- | --- |
| `stead_contracts::ContractStatus` | `FfiContractStatus` | `FfiContractStatus` |
| `stead_usf::CliType` | `FfiCliType` | `FfiCliType` |
| `stead_contracts::Contract` (projected) | `FfiContract` | `FfiContract` |
| `stead_usf::SessionRecord` (projected) | `FfiSessionSummary` | `FfiSessionSummary` |

### Functions

| Function | Signature |
| --- | --- |
| `list_contracts` | `(cwd: String) throws -> [FfiContract]` |
| `get_contract` | `(id: String, cwd: String) throws -> FfiContract` |
| `list_sessions` | `(cliFilter: String?, project: String?, limit: UInt32) -> [FfiSessionSummary]` |

`list_contracts` and `get_contract` are daemon-backed via workspace `.stead/stead.db`.

`list_sessions` reads workspace-local fixtures from `.stead/sessions/{claude,codex,opencode}`.

## Build

```bash
cargo build -p stead-ffi
```

## Generate Swift bindings

```bash
cargo run -p stead-ffi --bin uniffi-bindgen -- \
  generate --library target/debug/libstead_ffi.dylib \
  --language swift --out-dir stead-ffi/generated
```

## macOS release build

```bash
cargo build -p stead-ffi --release --target aarch64-apple-darwin
```
