use std::io::{self, Write};

use anyhow::Result;
use inquire::Select;
use serde::{Deserialize, Serialize};
use log::{info, error};

use crate::config;
use crate::http;

const NUM_SUGGESTIONS: usize = 3;
const TEMPERATURE: f32 = 0.8;

const OPEN_AI_BASE_URL: &str = "https://api.openai.com/v1/responses";

#[derive(Serialize)]
struct OpenApiRequest {
    model: String, 
    parallel_tool_calls: bool,
    temperature: f32,
    max_output_tokens: i32,
    input: Vec<Message>,
}

#[derive(Serialize)]
pub struct Message {
    role: String,
    content: String
}

#[derive(Deserialize)]
struct OpenApiResonse {
    output: Vec<Output>
}

#[derive(Deserialize)]
struct Output {
    content: Vec<Content>

}

#[derive(Deserialize)]
struct Content {
    text: String
}

fn get_system_prompt_message() -> Message {
    Message { 
        role: "system".to_string(),
        content:  "You rewrite git commit messages to be professional and follow Conventional Commits. Its ok if you skip scope but try to figure out. Output only the commit message. No explanations.".to_string()
    }
}

fn get_user_prompt_message(message: &str) -> Message {
    Message { 
        role: "user".to_string(),
        content:  message.to_string()
    }
}


fn get_or_prompt_api_key() -> Result<String> {
    if let Some(key) = config::load_openai_api_key()? {
        return Ok(key);
    }

    print!("Enter your OpenAI API key: ");
    io::stdout().flush()?;

    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    if api_key.is_empty() {
        return Err(anyhow::anyhow!("API key cannot be empty"));
    }

    config::save_openai_api_key(&api_key)?;
    info!("API key saved to config");
    Ok(api_key)
}

fn fetch_single_suggestion(client: &reqwest::blocking::Client, api_key: &str, original_msg: &str) -> Result<String> {
    let open_api_request = OpenApiRequest {
        model: "gpt-4.1-mini".to_string(),
        parallel_tool_calls: false,
        temperature: TEMPERATURE,
        max_output_tokens: 40,
        input: vec![get_system_prompt_message(), get_user_prompt_message(original_msg)]
    };

    let response = client.post(OPEN_AI_BASE_URL)
        .body(serde_json::to_string(&open_api_request)?)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()?;

    if !response.status().is_success() {
        error!("Failed to polish given message: {}", response.status());
        return Err(anyhow::anyhow!("Failed to get polished commit message"));
    }

    let response_text = response.text()?;
    let open_api_response: OpenApiResonse = serde_json::from_str(&response_text)?;
    let content = open_api_response.output.get(0)
        .ok_or_else(|| anyhow::anyhow!("No output in response"))?
        .content.get(0)
        .ok_or_else(|| anyhow::anyhow!("No content in response"))?;
    Ok(content.text.trim().to_string())
}

pub fn get_polished_commit_msg(original_msg: &str) -> Result<String> {
    let api_key = get_or_prompt_api_key()?;
    let client = http::get_client();

    info!("Generating commit message suggestions...");

    let mut suggestions: Vec<String> = Vec::new();
    for _ in 0..NUM_SUGGESTIONS {
        match fetch_single_suggestion(&client, &api_key, original_msg) {
            Ok(msg) => {
                if !suggestions.contains(&msg) {
                    suggestions.push(msg);
                }
            }
            Err(e) => error!("Failed to fetch suggestion: {}", e),
        }
    }

    if suggestions.is_empty() {
        return Err(anyhow::anyhow!("Failed to generate any commit message suggestions"));
    }

    let selected = Select::new("Select a commit message:", suggestions)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?;

    Ok(selected)
}