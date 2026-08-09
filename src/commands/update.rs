use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::Args;
use reqwest::header::USER_AGENT;
use serde::Deserialize;

use crate::commands::Command;

const LATEST_RELEASE_API_URL: &str = "https://api.github.com/repos/kcterala/kit/releases/latest";

#[derive(Args)]
pub struct UpdateCommand;

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

impl Command for UpdateCommand {
    fn execute(&self) -> Result<()> {
        update_kit()
    }
}

fn update_kit() -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .context("Could not create the update client")?;
    let user_agent = format!("kit/{}", env!("CARGO_PKG_VERSION"));
    let release = client
        .get(LATEST_RELEASE_API_URL)
        .header(USER_AGENT, &user_agent)
        .send()
        .context("Could not check for the latest Kit release")?
        .error_for_status()
        .context("GitHub rejected the release check")?
        .json::<GitHubRelease>()
        .context("Could not read the latest Kit release")?;
    let latest_version = release.tag_name.trim_start_matches('v');
    let current_version = env!("CARGO_PKG_VERSION");

    if latest_version == current_version {
        println!("kit {current_version} is already up to date");
        return Ok(());
    }

    let asset_name = release_asset_name()?;
    let download_url = format!(
        "https://github.com/kcterala/kit/releases/download/{}/{}",
        release.tag_name, asset_name
    );
    println!("Updating kit from {current_version} to {latest_version}...");

    let binary = client
        .get(download_url)
        .header(USER_AGENT, user_agent)
        .send()
        .context("Could not download the latest Kit release")?
        .error_for_status()
        .context("GitHub rejected the Kit download")?
        .bytes()
        .context("Could not read the downloaded Kit binary")?;

    replace_current_executable(&binary)?;
    println!("kit updated successfully to {latest_version}");
    Ok(())
}

fn release_asset_name() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => Ok("kit-macos-amd64"),
        ("macos", "aarch64") => Ok("kit-macos-arm64"),
        ("linux", "x86_64") => Ok("kit-linux-amd64"),
        ("linux", "aarch64") => Ok("kit-linux-arm64"),
        (os, architecture) => bail!("kit update does not support {os}/{architecture}"),
    }
}

fn replace_current_executable(binary: &[u8]) -> Result<()> {
    let executable = std::env::current_exe().context("Could not locate the current Kit binary")?;
    let directory = executable
        .parent()
        .context("The current Kit binary has no parent directory")?;
    let temporary_path = update_path(directory);

    let result = (|| -> Result<()> {
        let mut temporary_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
            .with_context(|| {
                format!(
                    "Could not write to {}; rerun with the permissions used to install kit",
                    directory.display()
                )
            })?;
        temporary_file
            .write_all(binary)
            .context("Could not write the updated Kit binary")?;
        temporary_file
            .set_permissions(fs::Permissions::from_mode(0o755))
            .context("Could not make the updated Kit binary executable")?;
        temporary_file
            .sync_all()
            .context("Could not finish writing the updated Kit binary")?;
        fs::rename(&temporary_path, &executable).with_context(|| {
            format!(
                "Could not replace {}; rerun with the permissions used to install kit",
                executable.display()
            )
        })?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}

fn update_path(directory: &Path) -> std::path::PathBuf {
    directory.join(format!(".kit-update-{}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_an_asset_for_the_build_platform() {
        let asset = release_asset_name().unwrap();

        assert!(asset.starts_with("kit-"));
        assert!(asset.ends_with("amd64") || asset.ends_with("arm64"));
    }
}
