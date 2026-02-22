use anyhow::Ok;
use clap::Args;
use log::info;

use crate::commands::Command;

#[derive(Args)]
pub struct ForkCommand {
    pub repo: String,
}

impl Command for ForkCommand {
    fn execute(&self) -> anyhow::Result<()> {
        info!("Fork not implemented");
        Ok(())
    }
}
