use clap::Args;

use crate::commands::Command;
use crate::services::ai;
use crate::services::git;

#[derive(Args)]
pub struct AiCommitCommand {
    pub message: String,
}

impl Command for AiCommitCommand {
    fn execute(&self) -> anyhow::Result<()> {
        // Fail fast before making an API call
        if !git::is_git_repo() {
            return Err(anyhow::anyhow!("Not a git repository"));
        }

        let ai_polished_message = ai::get_polished_commit_msg(&self.message)?;

        git::add_all()?;
        git::commit_with_message(&ai_polished_message)?;

        Ok(())
    }
}
