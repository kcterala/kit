use std::{fs, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct Config {
    pub github_token: String,
    pub github_username: String,
}

fn config_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    path.push("kit");
    fs::create_dir_all(&path)?;
    path.push("config.json");
    Ok(path)
}

pub fn save_credentials(token: &str, username: &str) -> Result<()> {
    let config = Config {
        github_token: token.to_string(),
        github_username: username.to_string(),
    };

    let config_json = serde_json::to_string(&config)?;
    fs::write(config_path()?, config_json)?;
    Ok(())
}

pub fn load_token() -> Result<String> {
    let path = config_path()?;
    let contents = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&contents)?;
    Ok(config.github_token)
}

pub fn load_username() -> Result<String> {
    let path = config_path()?;
    let contents = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&contents)?;
    Ok(config.github_username)
}
