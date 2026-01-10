use std::process::{Command, ExitStatus};
use anyhow::Result;
use log::{info, error};
use colored::*;
use crate::commands::github::GetRepoResponse;

pub fn clone_repository(repo_details: &GetRepoResponse) -> Result<ExitStatus> {
    info!("Cloning from {}", repo_details.ssh_url.bright_black());

    let clone_status = Command::new("git")
        .args(&["clone", &repo_details.ssh_url])
        .status()?;

    if !clone_status.success() {
        error!("Failed to clone repository");
    } else {
        info!("{} Repository cloned successfully", "✓".green());
    }

    Ok(clone_status)
}

pub fn add_upstream(repo_name: &str, parent_url: &str) -> Result<ExitStatus> {
    info!("Adding upstream remote...");

    let upstream_status = Command::new("git")
        .args(&["remote", "add", "upstream", &parent_url])
        .current_dir(repo_name)
        .status()?;

    if upstream_status.success() {
        info!("{} Upstream remote added: {}", "✓".green(), parent_url.bright_black());
    } else {
        error!("Failed to add upstream remote");
    }

    Ok(upstream_status)
}

pub fn is_git_repo() -> bool {
    Command::new("git")
        .args(&["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn add_all() -> Result<ExitStatus> {
    let status = Command::new("git")
        .args(&["add", "."])
        .status()?;

    if !status.success() {
        error!("Failed to stage changes");
    }

    Ok(status)
}

pub fn commit_with_message(message: &str) -> Result<ExitStatus> {
    let status = Command::new("git")
        .args(&["commit", "-m", message])
        .status()?;

    if status.success() {
        info!("{} Committed: {}", "✓".green(), message);
    } else {
        error!("Failed to commit");
    }

    Ok(status)
}
