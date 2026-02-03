//! stead CLI - Operating environment for agent-driven development
//!
//! Commands:
//! - run: Create and execute a contract
//! - list: List contracts with optional filtering
//! - show: Display contract details
//! - verify: Re-run contract verification

use clap::Parser;
use stead::cli::{Cli, Commands};
use stead::commands;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { task, verify } => {
            commands::run::execute(&task, &verify, cli.json)?;
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
    }

    Ok(())
}
