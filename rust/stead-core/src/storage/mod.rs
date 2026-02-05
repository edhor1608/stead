//! JSONL-based contract storage
//!
//! Contracts are stored in a single append-only JSONL file:
//! .stead/contracts.jsonl

mod jsonl;

pub use jsonl::*;
