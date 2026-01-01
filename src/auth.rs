use std::{thread, time::Duration};

use anyhow::{Result};
use reqwest::blocking::{Client};
use serde::Deserialize;
use log::{info, error};
use colored::*;
use crate::config;

const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    interval: u64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
}

pub fn get_github_token() -> Result<String> {
    if let Ok(token) = config::load_token() {
        return Ok(token);
    }
    
    login()?;
    config::load_token()

}

pub fn login() -> Result<()> {
    let client = Client::new();
    let client_id = "Ov23liC1zydB6XvkXoCl".to_string();

    let device: DeviceCodeResponse = client.post(DEVICE_CODE_URL)
    .header("Accept", "application/json")
    .form(&[
            ("client_id", &client_id),
            ("scope", &"read:user public_repo".to_string()),
        ])
        .send()?
        .json()?;

    info!("Opening browser for GitHub login...");
    info!("Enter this code: {}", device.user_code.bright_cyan().bold());
    open::that(&device.verification_uri)?;

    loop {
        thread::sleep(Duration::from_secs(device.interval));
        let token: TokenResponse = client
            .post(TOKEN_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", &client_id),
                ("device_code", &device.device_code),
                (
                    "grant_type",
                    &"urn:ietf:params:oauth:grant-type:device_code".to_string(),
                ),
            ])
            .send()?
            .json()?;

        if let Some(access_token) = token.access_token {
            config::save_token(&access_token)?;
            info!("{} GitHub authentication successful!", "✓".green());
            return Ok(());
        }

        match token.error.as_deref() {
            Some("authorization_pending") => continue,
            Some("access_denied") => return Err(anyhow::anyhow!("Authorization denied")),
            Some(err) => return Err(anyhow::anyhow!("OAuth error: {}", err)),
            None => {}
        }

    }

}
