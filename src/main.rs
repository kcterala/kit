use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

use crate::commands::{
    Command, ai_commit::AiCommitCommand, clone::CloneCommand, commit::CommitCommand,
    fork::ForkCommand, ip::IpCommand, network::NetworkCommand, setup::SetupCommand,
};

mod commands;
mod config;
mod http;
mod services;
mod utils;

#[derive(Parser)]
#[command(name = "kit")]
#[command(about = "small utility tools ", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Clone(CloneCommand),
    Fork(ForkCommand),
    Commit(CommitCommand),
    AiCommit(AiCommitCommand),
    Ip(IpCommand),
    Network(NetworkCommand),
    Setup(SetupCommand),
}

fn main() -> Result<()> {
    init_logger();

    let cli = Cli::parse();

    let command: Box<dyn Command> = match cli.command {
        Commands::Clone(cmd) => Box::new(cmd),
        Commands::Fork(cmd) => Box::new(cmd),
        Commands::Ip(cmd) => Box::new(cmd),
        Commands::AiCommit(cmd) => Box::new(cmd),
        Commands::Commit(cmd) => Box::new(cmd),
        Commands::Network(cmd) => Box::new(cmd),
        Commands::Setup(cmd) => Box::new(cmd),
    };

    command.execute()?;

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

            writeln!(buf, "{} {}", level_string, record.args())
        })
        .init();
}
