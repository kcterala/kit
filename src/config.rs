use std::{fs, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub github_token: String,
    #[serde(default)]
    pub github_username: String,
    #[serde(default)]
    pub openai_api_key: String,
}

fn config_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    path.push("kit");
    fs::create_dir_all(&path)?;
    path.push("config.json");
    Ok(path)
}

pub fn save_credentials(token: &str, username: &str) -> Result<()> {
    let path = config_path()?;
    let mut config = if path.exists() {
        let contents = fs::read_to_string(&path)?;
        serde_json::from_str(&contents).unwrap_or_default()
    } else {
        Config::default()
    };

    config.github_token = token.to_string();
    config.github_username = username.to_string();

    let config_json = serde_json::to_string(&config)?;
    fs::write(path, config_json)?;
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

pub fn load_openai_api_key() -> Result<Option<String>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&contents)?;
    if config.openai_api_key.is_empty() {
        Ok(None)
    } else {
        Ok(Some(config.openai_api_key))
    }
}

pub fn save_openai_api_key(api_key: &str) -> Result<()> {
    let path = config_path()?;
    let mut config = if path.exists() {
        let contents = fs::read_to_string(&path)?;
        serde_json::from_str(&contents).unwrap_or_default()
    } else {
        Config::default()
    };

    config.openai_api_key = api_key.to_string();
    let config_json = serde_json::to_string(&config)?;
    fs::write(path, config_json)?;
    Ok(())
}
