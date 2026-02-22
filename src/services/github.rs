use crate::http;
use anyhow::Result;
use log::{debug, error};
use serde::Deserialize;

const GET_REPO_DETAILS: &str = "https://api.github.com/repos/{owner}/{repo}";

#[derive(Deserialize, Debug)]
pub struct GetRepoResponse {
    pub fork: bool,
    pub ssh_url: String,
    pub parent: Option<ParentRepoInfo>,
}

#[derive(Deserialize, Debug)]
pub struct ParentRepoInfo {
    pub ssh_url: String,
}

pub fn get_repo_details(token: &str, owner: &str, repo_name: &str) -> Result<GetRepoResponse> {
    debug!("Fetching repo details for {}/{}", owner, repo_name);

    let client = http::get_client();
    let url = GET_REPO_DETAILS
        .replace("{owner}", owner)
        .replace("{repo}", repo_name);

    let response = client
        .get(&url)
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

pub fn fetch_github_username(client: &reqwest::blocking::Client, token: &str) -> Result<String> {
    let response = client
        .get("https://api.github.com/user")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Github-Api-Version", "2022-11-28")
        .header("User-Agent", "kit-cli")
        .send()?;

    if !response.status().is_success() {
        error!("Failed to fetch user info from GitHub");
        return Err(anyhow::anyhow!("Failed to fetch user info"));
    }

    #[derive(Deserialize)]
    struct UserInfo {
        login: String,
    }

    let user_info: UserInfo = response.json()?;
    Ok(user_info.login)
}
