use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use colored::*;

mod config;
mod auth;
mod commands;
mod utils;
mod http;

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

    Ip {
        #[arg(short, long, help = "Copy IP to clipboard")]
        copy: bool,
    }
}

fn main() -> Result<()> {
    init_logger();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Clone { repo } => commands::clone_repository(repo)?,
        Commands::Fork { repo } => commands::fork_repository(repo)?,
        Commands::Ip { copy } => commands::ip(*copy)?,
    }

    Ok(())
}

fn init_logger() {
    Builder::new()
        .filter_level(LevelFilter::Info)
        .format(|buf, record| {
            let level_string = match record.level() {
                log::Level::Error => "ERROR".red().bold(),
                log::Level::Warn => "WARN".yellow().bold(),
                log::Level::Info => "INFO".green(),
                log::Level::Debug => "DEBUG".blue(),
                log::Level::Trace => "TRACE".purple(),
            };

            writeln!(
                buf,
                "{} {}",
                level_string,
                record.args()
            )
        })
        .init();
}
