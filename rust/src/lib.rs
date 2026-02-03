//! stead - Operating environment for agent-driven development
//!
//! This library provides the core functionality for the stead CLI:
//! - Contract schema and lifecycle management
//! - JSONL-based persistent storage
//! - CLI argument parsing
//! - Command implementations

pub mod cli;
pub mod commands;
pub mod schema;
pub mod storage;
