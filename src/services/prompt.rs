use anyhow::Result;
use inquire::{Confirm, Text};

pub fn text_input(message: &str) -> Result<String> {
    Text::new(message)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Input cancelled: {}", e))
}

pub fn confirm(message: &str) -> Result<bool> {
    Confirm::new(message)
        .with_default(false)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Confirmation cancelled: {}", e))
}
