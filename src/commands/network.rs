use std::fmt;
use std::net::{TcpStream, ToSocketAddrs};
use std::process;
use std::time::Duration;

use anyhow::Result;
use clap::Args;
use log::info;

use crate::commands::Command;
use crate::services::prompt;

#[derive(Args)]
pub struct NetworkCommand;

#[derive(Clone)]
pub enum NetworkOption {
    Host,
    Ssl,
}

const ALL_OPTIONS: [NetworkOption; 2] = [NetworkOption::Host, NetworkOption::Ssl];

impl fmt::Display for NetworkOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkOption::Host => write!(f, "Check host availability"),
            NetworkOption::Ssl => write!(f, "Check SSL certificate"),
        }
    }
}

impl Command for NetworkCommand {
    fn execute(&self) -> Result<()> {
        let selection = prompt::select("What do you want my help with?", ALL_OPTIONS.to_vec())?;

        match selection {
            NetworkOption::Host => {
                let host = prompt::text_input("Enter the host:")?;
                info!("Checking host: {}", host);
                diagnose_url(&host)?;
            }
            NetworkOption::Ssl => {
                let url = prompt::text_input("Enter the URL:")?;
                info!("Checking SSL: {}", url);
            }
        };

        Ok(())
    }
}

fn diagnose_url(host: &str) -> Result<()> {
    let addrs = match (host, 443).to_socket_addrs() {
        Ok(addrs) => {
            println!("✅ DNS resolution successful!");
            addrs
        }
        Err(e) => {
            println!("❌ Cannot do dns resolution {e}");
            return Err(e.into());
        }
    };

    ping(host);

    let timeout = Duration::from_secs(3);
    let mut last_err = None;

    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(_) => {
                println!("✅ Connected to {}", addr);
                return Ok(());
            }
            Err(e) => {
                println!("❌ Failed {}: {}", addr, e);
                last_err = Some(e);
            }
        }
    }

    match last_err {
        Some(e) => Err(e.into()),
        None => Err(anyhow::anyhow!("no addresses to connect to")),
    }
}

fn ping(host: &str) {
    let status = process::Command::new("ping")
        .args(&["-c", "3", "-W", "3", host])
        .stdout(process::Stdio::null())
        .status();

    match status {
        Ok(s) if s.success() => println!("✅ Ping successful!"),
        Ok(_) => println!("❌ Ping failed: host unreachable"),
        Err(e) => println!("❌ Ping error: {}", e),
    }
}
