use std::{env, thread, time::Duration};

use anyhow::{Result};
use reqwest::blocking::{Client};
use serde::Deserialize;
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
    let client_id = get_client_id();

    let device: DeviceCodeResponse = client.post(DEVICE_CODE_URL)
    .header("Accept", "application/json")
    .form(&[
            ("client_id", &client_id),
            ("scope", &"read:user public_repo".to_string()),
        ])
        .send()?
        .json()?;

    println!("Opening browser for GitHub login...");
    println!("Enter this code: {}", device.user_code);
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
            println!("Github authentication successful!");
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

fn get_client_id() -> String {
    dotenv::dotenv().ok();
    env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set in .env")
}
