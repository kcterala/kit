use std::fmt::Display;

use anyhow::Result;
use inquire::{Select, Text};

pub fn select<T: Display + Clone>(message: &str, options: Vec<T>) -> Result<T> {
    Select::new(message, options)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))
}

pub fn text_input(message: &str) -> Result<String> {
    Text::new(message)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Input cancelled: {}", e))
}
