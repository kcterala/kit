use anyhow::Result;
use reqwest::blocking::Client;
use serde::Deserialize;
use log::{debug, error};
use crate::config;

const GET_REPO_DETAILS: &str = "https://api.github.com/repos/{owner}/{repo}";

#[derive(Deserialize, Debug)]
pub struct GetRepoResponse {
    pub fork: bool,
    pub ssh_url: String,
    pub parent: Option<ParentRepoInfo>
}

#[derive(Deserialize, Debug)]
pub struct ParentRepoInfo {
    pub ssh_url: String,
}


pub fn get_repo_details(owner: &str, repo_name: &str) -> Result<GetRepoResponse> {
    debug!("Fetching repo details for {}/{}", owner, repo_name);

    let token = config::load_token()?;
    let client = Client::new();
    let url = GET_REPO_DETAILS.replace("{owner}", owner).replace("{repo}", repo_name);

    let response = client.get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Github-Api-Version", "2022-11-28")
        .header("User-Agent", "kit-cli")
        .send()?;

    debug!("Status: {}", response.status());
    if !response.status().is_success() {
        error!("Failed to fetch repo details from GitHub");
        return Err(anyhow::anyhow!("failed to fetch repo details from github"));
    }
    let response_text = response.text()?;
    debug!("Response body: {}", response_text);
    let get_repo_response: GetRepoResponse = serde_json::from_str(&response_text)?;

    Ok(get_repo_response)

}
