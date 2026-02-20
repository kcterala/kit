use anyhow::Result;
use clap::Args;

use crate::commands::Command;
use crate::services::{ai, git};

#[derive(Args)]
pub struct AiCommitCommand {
    pub message: String,
}

impl Command for AiCommitCommand {
    fn execute(&self) -> Result<()> {
        if !git::is_git_repo() {
            return Err(anyhow::anyhow!("Not a git repository"));
        }

        let polished_message = ai::get_polished_commit_msg(&self.message)?;

        git::add_all()?;
        git::commit_with_message(&polished_message)?;

        Ok(())
    }
}
