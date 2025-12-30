use anyhow::{Ok, Result};

mod config;
mod auth;

fn main() -> Result<()> {
    println!("Hello, world!");
    auth::login()?;
    Ok(())
}
