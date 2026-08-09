use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use log::info;

use crate::commands::Command;
use crate::http;

const AGENT_INSTRUCTIONS_URL: &str = "https://agents.kcterala.dev/agents.md";

#[derive(Args)]
pub struct SetupCommand {
    #[command(subcommand)]
    command: SetupCommands,
}

#[derive(Subcommand)]
enum SetupCommands {
    #[command(about = "Install global coding conventions for detected coding agents")]
    Agents,
}

impl Command for SetupCommand {
    fn execute(&self) -> Result<()> {
        match &self.command {
            SetupCommands::Agents => setup_agents(),
        }
    }
}

struct Agent {
    name: &'static str,
    executable: &'static str,
    config_directory: &'static str,
    instructions_file: &'static str,
}

const SUPPORTED_AGENTS: [Agent; 2] = [
    Agent {
        name: "Pi",
        executable: "pi",
        config_directory: ".pi/agent",
        instructions_file: ".pi/agent/AGENTS.md",
    },
    Agent {
        name: "Claude Code",
        executable: "claude",
        config_directory: ".claude",
        instructions_file: ".claude/CLAUDE.md",
    },
];

fn setup_agents() -> Result<()> {
    let home_directory = dirs::home_dir().context("Could not find home directory")?;
    let detected_agents = detect_agents(&home_directory, std::env::var_os("PATH").as_deref());

    if detected_agents.is_empty() {
        bail!("No supported coding agents detected (supported: Pi and Claude Code)");
    }

    let instructions = download_agent_instructions()?;

    for (agent_name, instructions_file) in detected_agents {
        overwrite_agent_instructions(&instructions_file, &instructions)?;
        info!(
            "Installed conventions for {} at {}",
            agent_name,
            instructions_file.display()
        );
    }

    Ok(())
}

fn detect_agents(home_directory: &Path, path: Option<&OsStr>) -> Vec<(&'static str, PathBuf)> {
    SUPPORTED_AGENTS
        .iter()
        .filter(|agent| {
            home_directory.join(agent.config_directory).is_dir()
                || executable_exists_in_path(agent.executable, path)
        })
        .map(|agent| (agent.name, home_directory.join(agent.instructions_file)))
        .collect()
}

fn executable_exists_in_path(executable: &str, path: Option<&OsStr>) -> bool {
    let Some(path) = path else {
        return false;
    };

    std::env::split_paths(path).any(|directory| {
        let executable_path = directory.join(executable);
        executable_path.is_file() || executable_path.with_extension("exe").is_file()
    })
}

fn download_agent_instructions() -> Result<String> {
    http::get_client()
        .get(AGENT_INSTRUCTIONS_URL)
        .header("User-Agent", "kit-cli")
        .send()
        .context("Failed to download coding conventions")?
        .error_for_status()
        .context("Failed to download coding conventions")?
        .text()
        .context("Failed to read coding conventions")
}

fn overwrite_agent_instructions(instructions_file: &Path, instructions: &str) -> Result<()> {
    let parent_directory = instructions_file
        .parent()
        .context("Agent instructions file has no parent directory")?;

    fs::create_dir_all(parent_directory).with_context(|| {
        format!(
            "Failed to create agent config directory {}",
            parent_directory.display()
        )
    })?;
    fs::write(instructions_file, instructions).with_context(|| {
        format!(
            "Failed to overwrite agent instructions at {}",
            instructions_file.display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn temporary_directory() -> PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("kit-setup-agents-{unique_suffix}"))
    }

    #[test]
    fn detects_agents_from_existing_config_directories() {
        let home_directory = temporary_directory();
        fs::create_dir_all(home_directory.join(".pi/agent")).unwrap();
        fs::create_dir_all(home_directory.join(".claude")).unwrap();

        let detected_agents = detect_agents(&home_directory, None);

        assert_eq!(detected_agents.len(), 2);
        assert_eq!(
            detected_agents[0].1,
            home_directory.join(".pi/agent/AGENTS.md")
        );
        assert_eq!(
            detected_agents[1].1,
            home_directory.join(".claude/CLAUDE.md")
        );

        fs::remove_dir_all(home_directory).unwrap();
    }

    #[test]
    fn detects_agents_from_executables_in_path() {
        let home_directory = temporary_directory();
        let binary_directory = home_directory.join("bin");
        fs::create_dir_all(&binary_directory).unwrap();
        fs::write(binary_directory.join("pi"), "").unwrap();

        let detected_agents = detect_agents(&home_directory, Some(binary_directory.as_os_str()));

        assert_eq!(detected_agents.len(), 1);
        assert_eq!(detected_agents[0].0, "Pi");

        fs::remove_dir_all(home_directory).unwrap();
    }

    #[test]
    fn overwrites_existing_agent_instructions() {
        let home_directory = temporary_directory();
        let instructions_file = home_directory.join(".claude/CLAUDE.md");
        fs::create_dir_all(instructions_file.parent().unwrap()).unwrap();
        fs::write(&instructions_file, "old instructions").unwrap();

        overwrite_agent_instructions(&instructions_file, "new instructions").unwrap();

        assert_eq!(
            fs::read_to_string(&instructions_file).unwrap(),
            "new instructions"
        );

        fs::remove_dir_all(home_directory).unwrap();
    }
}
