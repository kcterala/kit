use anyhow::Result;
use clap::Args;
use inquire::Select;
use log::info;

use crate::commands::Command;

#[derive(Args)]
pub struct NetworkCommand;

impl Command for NetworkCommand {
    fn execute(&self) -> Result<()> {
        let options = vec![
            "Is my internet working?",
            "Is this url working?",
            "Is this host reachable?",
        ];

        let selected = Select::new("What do you want to know, cheif", options)
            .prompt()
            .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?;

        info!("User selected ${selected}");

        Ok(())
    }
}
