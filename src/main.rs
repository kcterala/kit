use anyhow::Result;
use clap::{Parser, Subcommand};

mod config;
mod auth;
mod commands;

#[derive(Parser)]
#[command(name = "kit")]
#[command(about = "A GitHub CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Clone {
        repo: String,
    },
    
    Fork {
        repo: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Clone { repo } => commands::clone_repository(repo)?,
        Commands::Fork { repo } => commands::fork_repository(repo)?,
    }

    Ok(())
}
