use std::process::{Command, ExitStatus};
use anyhow::Result;
use crate::commands::github::GetRepoResponse;


pub fn clone_repository(repo_details: &GetRepoResponse) -> Result<ExitStatus> {
    let clone_status = Command::new("git")
        .args(&["clone", &repo_details.ssh_url])
        .status()?;

    if !clone_status.success() {
        println!("Failed to clone repository");
    }

    Ok(clone_status)
}

pub fn add_upstream(repo_name: &String, parent_url: &String) -> Result<ExitStatus> {
    let upstream_status = Command::new("git")
        .args(&["remote", "add", "upstream", &parent_url])
        .current_dir(repo_name)
        .status()?;

    if upstream_status.success() {
        println!("Upstrem remote added: {}", parent_url);
    } else {
        println!("Failed to add upstream remote");
    }

    Ok(upstream_status)
}
