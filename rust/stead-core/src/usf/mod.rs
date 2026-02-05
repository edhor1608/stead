//! Universal Session Format
//!
//! This module provides a canonical representation for AI coding CLI sessions,
//! enabling unified visibility across Claude Code, Codex CLI, and OpenCode.

pub mod adapters;
pub mod schema;

pub use schema::*;
