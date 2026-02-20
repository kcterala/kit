use anyhow::Result;
use clap::Args;

use crate::commands::Command;
use crate::services::git;

#[derive(Args)]
pub struct CommitCommand {
    pub message: String,
}

impl Command for CommitCommand {
    fn execute(&self) -> Result<()> {
        if !git::is_git_repo() {
            return Err(anyhow::anyhow!("Not a git repository"));
        }

        git::add_all()?;
        git::commit_with_message(&self.message)?;

        Ok(())
    }
}
