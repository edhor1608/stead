# stead (Rust workspace)

Rust implementation for the daemon-backed CLI, reusable domain crates, and macOS bridge.

## Workspace Structure

```text
rust/
├── Cargo.toml
├── stead-cli/           # binary CLI, grouped family surface
├── stead-daemon/        # versioned API envelope + event contracts
├── stead-contracts/     # 10-state lifecycle engine + SQLite repository
├── stead-resources/     # generic lease/negotiation domain
├── stead-endpoints/     # named localhost endpoint broker domain
├── stead-usf/           # Claude/Codex/OpenCode adapters + listing/query contract
├── stead-module-sdk/    # session proxy + context generator modules
├── stead-ffi/           # UniFFI bridge consumed by macOS app
└── stead-test-utils/    # shared test fixtures + CI/doc guard tests
```

The CLI is daemon-backed: command families dispatch through `stead-daemon` request/response contracts, which compose the reusable domain crates.

## Build

```bash
cd rust
cargo build --workspace
```

## Test

```bash
cargo test --workspace
```

## Quality Gates

```bash
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
```

## CLI Families

```bash
stead                     # status overview
stead contract ...
stead session ...
stead resource ...
stead attention ...
stead context ...
stead module ...
stead daemon ...
```

## Notes

- Workspace data lives under `.stead/`.
- `--json` is available for machine-facing output across command families.
- Reusable crates are prepared for standalone export (metadata + packaging guards in tests).
