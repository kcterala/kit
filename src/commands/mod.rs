use anyhow::{Ok, Result};

use crate::commands::github::GetRepoResponse;

mod github;
mod git;

pub fn clone_repository(repo: &String) -> Result<()> {
    let (owner, repo_name) = match resolve(repo) {
        Some(result) => result,
        None => {
            eprintln!("Invalid repository URL format");
            return Ok(());
        }
    };
    let repo_details: GetRepoResponse = github::get_repo_details(&owner, &repo_name)?;
    let clone_status = git::clone_repository(&repo_details)?;
    if clone_status.success() && repo_details.fork {
        println!("this repo is a fork, add parent url as remote upstream");
        if let Some(parent) = repo_details.parent {
            git::add_upstream(&repo_name, &parent.ssh_url)?;
        }
    }

    Ok(())
}

pub fn fork_repository(repo: &String) -> Result<()> {
    println!("Not implemented till now");
    Ok(())
}

// https://github.com/kcterala/kcx.git
// git@github.com:kcterala/kcx.git
fn resolve(repo_url: &String) -> Option<(String, String)> {
    if repo_url.starts_with("https://github.com/") {
        let path = repo_url.strip_prefix("https://github.com/")?;
        let path = path.strip_suffix(".git")?;

        let parts: Vec<&str> = path.split("/").collect();
        if parts.len() == 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    } else if repo_url.starts_with("git@github.com:") {
          // Parse SSH URL
          let path = repo_url.strip_prefix("git@github.com:")?;
          let path = path.strip_suffix(".git")?;

          let parts: Vec<&str> = path.split('/').collect();
          if parts.len() == 2 {
              return Some((parts[0].to_string(), parts[1].to_string()));
          }
      }

      None
}
