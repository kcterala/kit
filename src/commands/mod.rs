use anyhow::{Ok, Result};
use log::{info, error};
use colored::*;

use crate::commands::github::GetRepoResponse;

mod github;
mod git;

pub fn clone_repository(repo: &str) -> Result<()> {
    let (owner, repo_name) = match resolve(repo) {
        Some(result) => result,
        None => {
            error!("Invalid repository URL format");
            return Err(anyhow::anyhow!("Invalid repository URL format"));
        }
    };

    info!("Cloning repository {}/{}", owner, repo_name);
    let repo_details: GetRepoResponse = github::get_repo_details(&owner, &repo_name)?;
    let clone_status = git::clone_repository(&repo_details)?;

    if clone_status.success() && repo_details.fork {
        info!("Repository is a fork, adding parent as upstream remote");
        if let Some(parent) = repo_details.parent {
            git::add_upstream(&repo_name, &parent.ssh_url)?;
        }
    }

    Ok(())
}

pub fn fork_repository(repo: &str) -> Result<()> {
    info!("Fork command not implemented yet");
    Ok(())
}

// https://github.com/kcterala/kcx.git
// git@github.com:kcterala/kcx.git
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
