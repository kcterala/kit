use anyhow::{Result};
use log::{info, error, warn};
use colored::*;

use crate::commands::github::GetRepoResponse;
use crate::utils;
use crate::http;
use crate::config;
use crate::auth;

pub mod github;
mod git;
mod ai;

const BASE_URL_FOR_IP: &str = "https://1.1.1.1/cdn-cgi/trace";

pub fn commit(message: &str) -> Result<()> {
    if !git::is_git_repo() {
        return Err(anyhow::anyhow!("Not a git repository"));
    }

    let polished_message = ai::get_polished_commit_msg(message)?;

    git::add_all()?;
    git::commit_with_message(&polished_message)?;

    Ok(())
}

pub fn clone_repository(repo: &str) -> Result<()> {
    let (owner, repo_name) = match resolve(repo) {
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
        let parent = repo_details.parent
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Forked repository has no parent"))?;

        git::add_upstream(&repo_name, &parent.ssh_url)?;
    }

    Ok(())
}


fn should_add_upstream(owner: &str, repo_details: &GetRepoResponse) -> Result<bool> {
    let github_username = config::load_username()?;
    Ok(repo_details.fork && owner.eq_ignore_ascii_case(&github_username))
}

pub fn fork_repository(repo: &str) -> Result<()> {
    info!("Fork command not implemented yet");
    Ok(())
}


pub fn ip(copy_to_clipboard: bool) -> Result<()> {
    let client = http::get_client();

    let response = client.get(BASE_URL_FOR_IP)
        .header("User-Agent", "kit-cli")
        .send()?;

    if !response.status().is_success() {
        error!("Failed to fetch IP address from Cloudflare");
        return Err(anyhow::anyhow!("Failed to fetch IP address"));
    }

    let body = response.text()?;

    // Parse the response to extract information
    let mut ip_address = None;
    let mut location = None;

    for line in body.lines() {
        if let Some(value) = line.strip_prefix("ip=") {
            ip_address = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("loc=") {
            location = Some(value.to_string());
        }
    }

    if let Some(ip) = ip_address {
        info!("Your IP address: {}", ip.cyan().bold());

        if let Some(loc) = location {
            info!("Location: {}", loc.yellow());
        }

        // Copy to clipboard only if flag is set
        if copy_to_clipboard {
            match utils::copy_to_clipboard(&ip) {
                std::result::Result::Ok(_) => info!("IP address copied to clipboard!"),
                Err(e) => warn!("Failed to copy to clipboard: {}", e),
            }
        }

        return Ok(());
    }

    error!("Could not find IP address in response");
    Err(anyhow::anyhow!("Could not parse IP address from response"))
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
