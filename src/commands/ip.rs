use clap::{Args, arg};
use colored::*;
use log::{error, info, warn};

use crate::commands::Command;
use crate::http;
use crate::utils;

const BASE_URL_FOR_IP: &str = "https://1.1.1.1/cdn-cgi/trace";

#[derive(Args)]
pub struct IpCommand {
    #[arg(short, long, help = "Copy IP to clipboard")]
    pub copy: bool,
}

impl Command for IpCommand {
    fn execute(&self) -> anyhow::Result<()> {
        let client = http::get_client();

        let response = client
            .get(BASE_URL_FOR_IP)
            .header("User-Agent", "kit-cli")
            .send()?;

        if !response.status().is_success() {
            error!("Failed to fetch IP address from Cloudflare");
            return Err(anyhow::anyhow!("Failed to fetch IP address"));
        }

        let body = response.text()?;

        // Parse the response to extract information
        let mut ip_address = None;
        let mut location = None;

        for line in body.lines() {
            if let Some(value) = line.strip_prefix("ip=") {
                ip_address = Some(value.to_string());
            } else if let Some(value) = line.strip_prefix("loc=") {
                location = Some(value.to_string());
            }
        }

        if let Some(ip) = ip_address {
            info!("Your IP address: {}", ip.cyan().bold());

            if let Some(loc) = location {
                info!("Location: {}", loc.yellow());
            }

            // Copy to clipboard only if flag is set
            if self.copy {
                match utils::copy_to_clipboard(&ip) {
                    std::result::Result::Ok(_) => info!("IP address copied to clipboard!"),
                    Err(e) => warn!("Failed to copy to clipboard: {}", e),
                }
            }

            return Ok(());
        }

        error!("Could not find IP address in response");
        Err(anyhow::anyhow!("Could not parse IP address from response"))
    }
}
