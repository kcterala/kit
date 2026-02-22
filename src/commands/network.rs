use anyhow::Result;
use clap::Args;
use log::info;

use crate::commands::Command;

#[derive(Args)]
pub struct NetworkCommand;

impl Command for NetworkCommand {
    fn execute(&self) -> Result<()> {
        info!("Network coming soon");
        Ok(())
    }
}
