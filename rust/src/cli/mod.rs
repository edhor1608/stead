//! CLI argument parsing with clap

use clap::{Parser, Subcommand};

/// stead - Operating environment for agent-driven development
#[derive(Parser, Debug)]
#[command(name = "stead")]
#[command(version = "0.2.0")]
#[command(about = "Operating environment for agent-driven development")]
pub struct Cli {
    /// Output as JSON (for agent consumption)
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create and execute a contract with verification
    Run {
        /// The task description for the agent
        task: String,

        /// Shell command to verify task completion (exit 0 = pass)
        #[arg(long)]
        verify: String,
    },

    /// List contracts with optional status filter
    List {
        /// Filter by status: pending, running, passed, failed
        #[arg(long)]
        status: Option<String>,
    },

    /// Show details of a specific contract
    Show {
        /// Contract ID
        id: String,
    },

    /// Re-run verification for a contract
    Verify {
        /// Contract ID
        id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_run_command_parsing() {
        let cli = Cli::parse_from(["stead", "run", "fix the bug", "--verify", "cargo test"]);
        match cli.command {
            Commands::Run { task, verify } => {
                assert_eq!(task, "fix the bug");
                assert_eq!(verify, "cargo test");
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_list_with_status() {
        let cli = Cli::parse_from(["stead", "list", "--status", "passed"]);
        match cli.command {
            Commands::List { status } => {
                assert_eq!(status, Some("passed".to_string()));
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_list_without_status() {
        let cli = Cli::parse_from(["stead", "list"]);
        match cli.command {
            Commands::List { status } => {
                assert_eq!(status, None);
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_show_command() {
        let cli = Cli::parse_from(["stead", "show", "abc123"]);
        match cli.command {
            Commands::Show { id } => {
                assert_eq!(id, "abc123");
            }
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn test_verify_command() {
        let cli = Cli::parse_from(["stead", "verify", "def456"]);
        match cli.command {
            Commands::Verify { id } => {
                assert_eq!(id, "def456");
            }
            _ => panic!("Expected Verify command"),
        }
    }

    #[test]
    fn test_json_flag() {
        let cli = Cli::parse_from(["stead", "--json", "list"]);
        assert!(cli.json);
    }
}
