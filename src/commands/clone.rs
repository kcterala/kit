use crate::auth;
use crate::commands::Command;
use crate::config;
use crate::services::git;
use crate::services::github;
use crate::services::github::GetRepoResponse;
use anyhow::Result;
use clap::Args;
use log::{error, info};

#[derive(Args)]
pub struct CloneCommand {
    pub repo: String,
}

impl Command for CloneCommand {
    fn execute(&self) -> anyhow::Result<()> {
        let (owner, repo_name) = match resolve(&self.repo) {
            Some(result) => result,
            None => {
                error!("Invalid repository URL format");
                return Err(anyhow::anyhow!("Invalid repository URL format"));
            }
        };

        // Ensure we have credentials (will trigger login if needed)
        auth::get_github_token()?;

        info!("Cloning repository {}/{}", owner, repo_name);
        let repo_details: GetRepoResponse = github::get_repo_details(&owner, &repo_name)?;
        let clone_status = git::clone_repository(&repo_details)?;

        if !clone_status.success() {
            return Err(anyhow::anyhow!("Could not clone repository"));
        }

        // Only add upstream if it's a fork AND owner matches logged-in user
        if should_add_upstream(&owner, &repo_details)? {
            info!("Repository is a fork, adding parent as upstream remote");
            let parent = repo_details
                .parent
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Forked repository has no parent"))?;

            git::add_upstream(&repo_name, &parent.ssh_url)?;
        }

        Ok(())
    }
}

/// https://github.com/kcterala/kcx.git
/// git@github.com:kcterala/kcx.git
fn resolve(repo_url: &str) -> Option<(String, String)> {
    if repo_url.starts_with("https://github.com/") {
        let path = repo_url.strip_prefix("https://github.com/")?;
        let path = path.strip_suffix(".git").unwrap_or(path);

        let parts: Vec<&str> = path.split("/").collect();
        if parts.len() == 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    } else if repo_url.starts_with("git@github.com:") {
        // Parse SSH URL
        let path = repo_url.strip_prefix("git@github.com:")?;
        let path = path.strip_suffix(".git").unwrap_or(path);

        let parts: Vec<&str> = path.split("/").collect();
        if parts.len() == 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }

    None
}

fn should_add_upstream(owner: &str, repo_details: &GetRepoResponse) -> Result<bool> {
    let github_username = config::load_username()?;
    Ok(repo_details.fork && owner.eq_ignore_ascii_case(&github_username))
}
