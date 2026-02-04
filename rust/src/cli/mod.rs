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

    /// Browse AI CLI sessions (Claude Code, Codex CLI, OpenCode)
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum SessionCommands {
    /// List sessions from all installed AI CLIs
    List {
        /// Filter by CLI: claude, codex, opencode
        #[arg(long)]
        cli: Option<String>,

        /// Filter by project path (substring match)
        #[arg(long)]
        project: Option<String>,

        /// Maximum number of sessions to show
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Show details of a specific session
    Show {
        /// Session ID (e.g., claude-abc123, codex-def456)
        id: String,

        /// Show full timeline (default: summary only)
        #[arg(long)]
        full: bool,
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

    #[test]
    fn test_session_list_command() {
        let cli = Cli::parse_from(["stead", "session", "list"]);
        match cli.command {
            Commands::Session { command } => match command {
                SessionCommands::List { cli, project, limit } => {
                    assert_eq!(cli, None);
                    assert_eq!(project, None);
                    assert_eq!(limit, 20);
                }
                _ => panic!("Expected List subcommand"),
            },
            _ => panic!("Expected Session command"),
        }
    }

    #[test]
    fn test_session_list_with_filters() {
        let cli = Cli::parse_from([
            "stead", "session", "list", "--cli", "claude", "--project", "stead", "--limit", "10",
        ]);
        match cli.command {
            Commands::Session { command } => match command {
                SessionCommands::List { cli, project, limit } => {
                    assert_eq!(cli, Some("claude".to_string()));
                    assert_eq!(project, Some("stead".to_string()));
                    assert_eq!(limit, 10);
                }
                _ => panic!("Expected List subcommand"),
            },
            _ => panic!("Expected Session command"),
        }
    }

    #[test]
    fn test_session_show_command() {
        let cli = Cli::parse_from(["stead", "session", "show", "claude-abc123"]);
        match cli.command {
            Commands::Session { command } => match command {
                SessionCommands::Show { id, full } => {
                    assert_eq!(id, "claude-abc123");
                    assert!(!full);
                }
                _ => panic!("Expected Show subcommand"),
            },
            _ => panic!("Expected Session command"),
        }
    }

    #[test]
    fn test_session_show_full() {
        let cli = Cli::parse_from(["stead", "session", "show", "--full", "codex-def456"]);
        match cli.command {
            Commands::Session { command } => match command {
                SessionCommands::Show { id, full } => {
                    assert_eq!(id, "codex-def456");
                    assert!(full);
                }
                _ => panic!("Expected Show subcommand"),
            },
            _ => panic!("Expected Session command"),
        }
    }
}
