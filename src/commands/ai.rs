use std::io::{self, Write};
use std::thread;

use anyhow::Result;
use inquire::Select;
use serde::{Deserialize, Serialize};
use spinoff::{spinners, Spinner, Color};
use log::error;

use crate::config;
use crate::http;

const NUM_SUGGESTIONS: usize = 3;
const TEMPERATURE: f32 = 0.8;
const MAX_REGENERATIONS: usize = 3;
const MAX_CONTEXT_LEN: usize = 500;

const OPEN_AI_BASE_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Serialize)]
struct OpenApiRequest {
    model: String,
    temperature: f32,
    max_tokens: i32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
pub struct Message {
    role: String,
    content: String
}

#[derive(Deserialize)]
struct OpenApiResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
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
    Ok(api_key)
}

fn fetch_single_suggestion(client: &reqwest::blocking::Client, api_key: &str, original_msg: &str) -> Result<String> {
    let request = OpenApiRequest {
        model: "gpt-4.1-mini".to_string(),
        temperature: TEMPERATURE,
        max_tokens: 60,
        messages: vec![get_system_prompt_message(), get_user_prompt_message(original_msg)],
    };

    let response = client.post(OPEN_AI_BASE_URL)
        .json(&request)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        error!("API error {}: {}", status, body);
        return Err(anyhow::anyhow!("Failed to get polished commit message"));
    }

    let api_response: OpenApiResponse = response.json()?;
    let content = api_response.choices.get(0)
        .ok_or_else(|| anyhow::anyhow!("No choices in response"))?
        .message.content.trim().to_string();

    Ok(content)
}

const ADD_CONTEXT_OPTION: &str = "↻ Regenerate with more context...";

fn fetch_suggestions(api_key: &str, message: &str) -> Vec<String> {
    let mut spinner = Spinner::new(spinners::BouncingBar, "Generating commit message suggestions...", Color::Blue);

    let handles: Vec<_> = (0..NUM_SUGGESTIONS)
        .map(|_| {
            let api_key = api_key.to_string();
            let msg = message.to_string();
            thread::spawn(move || {
                let client = http::get_client();
                fetch_single_suggestion(client, &api_key, &msg)
            })
        })
        .collect();

    let mut suggestions: Vec<String> = Vec::new();
    for handle in handles {
        match handle.join() {
            Ok(Ok(msg)) => {
                if !suggestions.contains(&msg) {
                    suggestions.push(msg);
                }
            }
            Ok(Err(e)) => error!("Failed to fetch suggestion: {}", e),
            Err(_) => error!("Thread panicked"),
        }
    }

    spinner.stop();
    suggestions
}

pub fn get_polished_commit_msg(original_msg: &str) -> Result<String> {
    let api_key = get_or_prompt_api_key()?;
    let mut current_msg = original_msg.to_string();
    let mut regeneration_count = 0;

    loop {
        if regeneration_count >= MAX_REGENERATIONS {
            return Err(anyhow::anyhow!(
                "Maximum regeneration attempts ({}) reached",
                MAX_REGENERATIONS
            ));
        }

        let suggestions = fetch_suggestions(&api_key, &current_msg);
        regeneration_count += 1;

        if suggestions.is_empty() {
            return Err(anyhow::anyhow!("Failed to generate any commit message suggestions"));
        }

        let options: Vec<String> = suggestions
            .into_iter()
            .chain(std::iter::once(ADD_CONTEXT_OPTION.to_string()))
            .collect();

        let selected = Select::new("Select a commit message:", options)
            .prompt()
            .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?;

        if selected == ADD_CONTEXT_OPTION {
            print!("Additional context: ");
            io::stdout().flush()?;

            let mut extra_context = String::new();
            io::stdin().read_line(&mut extra_context)?;
            let extra_context = extra_context.trim();

            if extra_context.is_empty() {
                continue;
            }

            let new_msg = format!("{}, {}", current_msg, extra_context);
            if new_msg.len() > MAX_CONTEXT_LEN {
                error!(
                    "Context too long ({} chars). Maximum allowed: {} chars",
                    new_msg.len(),
                    MAX_CONTEXT_LEN
                );
                continue;
            }

            current_msg = new_msg;
            continue;
        }

        return Ok(selected);
    }
}