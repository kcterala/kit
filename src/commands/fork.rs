use anyhow::Result;
use clap::Args;
use log::info;

use crate::commands::Command;

#[derive(Args)]
pub struct ForkCommand {
    pub repo: String,
}

impl Command for ForkCommand {
    fn execute(&self) -> Result<()> {
        info!("Fork command not implemented yet");
        Ok(())
    }
}
