//! stead CLI - Operating environment for agent-driven development
//!
//! Commands:
//! - run: Create and execute a contract
//! - list: List contracts with optional filtering
//! - show: Display contract details
//! - verify: Re-run contract verification
//! - session: Browse AI CLI sessions

use clap::Parser;
use stead_core::cli::{Cli, Commands, SessionCommands};
use stead_core::commands;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { task, verify } => {
            commands::run::execute(&task, &verify, cli.json)?;
        }
        Commands::Create { task, verify } => {
            commands::create::execute(&task, &verify, cli.json)?;
        }
        Commands::List { status } => {
            commands::list::execute(status.as_deref(), cli.json)?;
        }
        Commands::Show { id } => {
            commands::show::execute(&id, cli.json)?;
        }
        Commands::Verify { id } => {
            commands::verify::execute(&id, cli.json)?;
        }
        Commands::Claim { id, owner } => {
            commands::claim::execute(&id, &owner, cli.json)?;
        }
        Commands::Cancel { id } => {
            commands::cancel::execute(&id, cli.json)?;
        }
        Commands::Session { command } => match command {
            SessionCommands::List { cli: cli_filter, project, limit } => {
                commands::session::list_sessions(
                    cli_filter.as_deref(),
                    project.as_deref(),
                    limit,
                    cli.json,
                )?;
            }
            SessionCommands::Show { id, full } => {
                commands::session::show_session(&id, full, cli.json)?;
            }
        },
    }

    Ok(())
}
