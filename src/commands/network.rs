use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

use anyhow::Result;
use clap::Args;
use colored::Colorize;
use log::info;
use openssl::asn1::Asn1Time;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};

use crate::commands::Command;
use crate::services::prompt;

#[derive(Args)]
pub struct NetworkCommand;

impl Command for NetworkCommand {
    fn execute(&self) -> Result<()> {
        let host = prompt::text_input("Enter the host:")?;
        info!("Diagnosing network for: {}", host);

        diagnose_host(&host)?;
        get_ssl_info(&host)?;

        println!(
            "\n{}",
            "No issues found. Host is reachable and TLS handshake succeeded."
                .green()
                .bold()
        );

        Ok(())
    }
}

fn diagnose_host(host: &str) -> Result<()> {
    let addrs = match (host, 443).to_socket_addrs() {
        Ok(addrs) => {
            println!("{} DNS resolved successfully", "✔".green());
            addrs
        }
        Err(e) => {
            println!("{} DNS resolution failed — host not found: {e}", "✘".red());
            return Err(e.into());
        }
    };
    let mut last_err = None;

    for addr in addrs {
        match TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT) {
            Ok(_) => {
                println!("{} TCP connection established ({})", "✔".green(), addr);
                return Ok(());
            }
            Err(e) => {
                println!("{} Could not connect to {}: {}", "✘".red(), addr, e);
                last_err = Some(e);
            }
        }
    }

    match last_err {
        Some(e) => Err(e.into()),
        None => Err(anyhow::anyhow!("no addresses to connect to")),
    }
}

fn extract_cn(name: &openssl::x509::X509NameRef) -> String {
    name.entries_by_nid(openssl::nid::Nid::COMMONNAME)
        .next()
        .and_then(|e| e.data().as_utf8().ok())
        .map(|d| d.to_string())
        .unwrap_or_else(|| "Unknown".into())
}

fn get_ssl_info(host: &str) -> Result<()> {
    let addr = format!("{}:443", host);
    let sock_addrs = addr.to_socket_addrs()?.collect::<Vec<_>>();
    if sock_addrs.is_empty() {
        return Err(anyhow::anyhow!("No addresses found for {}", host));
    }

    let mut builder = SslConnector::builder(SslMethod::tls())?;
    builder.set_verify(SslVerifyMode::NONE);
    let connector = builder.build();

    let mut connect_errors = Vec::new();
    let mut ssl_stream = None;
    for sock_addr in sock_addrs {
        match TcpStream::connect_timeout(&sock_addr, CONNECT_TIMEOUT) {
            Ok(stream) => {
                stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT))?;
                stream.set_write_timeout(Some(HANDSHAKE_TIMEOUT))?;
                match connector.connect(host, stream) {
                    Ok(stream) => {
                        stream.get_ref().set_read_timeout(None)?;
                        stream.get_ref().set_write_timeout(None)?;
                        ssl_stream = Some(stream);
                        break;
                    }
                    Err(e) => {
                        connect_errors
                            .push(format!("{sock_addr}: TLS handshake/verification failed: {e}"));
                    }
                }
            }
            Err(e) => connect_errors.push(format!("{sock_addr}: TCP connect failed: {e}")),
        }
    }
    let ssl_stream = ssl_stream.ok_or_else(|| {
        anyhow::anyhow!(
            "Unable to establish TLS connection to {} across {} resolved addresses: {}",
            host,
            connect_errors.len(),
            connect_errors.join("; ")
        )
    })?;

    let cert = ssl_stream
        .ssl()
        .peer_certificate()
        .ok_or(anyhow::anyhow!("No certificate presented"))?;

    println!(
        "\n{} TLS handshake completed {}",
        "✔".green(),
        "(insecure — certificate verification is disabled)".yellow()
    );
    println!("{} {}", "Protocol:".yellow(), ssl_stream.ssl().version_str());

    let subject_cn = extract_cn(cert.subject_name());
    let issuer_cn = extract_cn(cert.issuer_name());

    println!("\n{}", "Certificate Details".bold());
    println!("{} {}", "Issued to:".yellow(), subject_cn);
    println!("{} {}", "Issued by:".yellow(), issuer_cn);

    let sans = cert
        .subject_alt_names()
        .map(|names| {
            names
                .iter()
                .filter_map(|n| n.dnsname().map(String::from))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if !sans.is_empty() {
        println!("{} {}", "SANs:".yellow(), sans.join(", "));
    }

    let serial = cert
        .serial_number()
        .to_bn()
        .map(|bn| bn.to_hex_str().map(|s| s.to_string()))
        .ok()
        .and_then(|r| r.ok())
        .unwrap_or_else(|| "Unknown".into());

    println!("{} {}", "Serial:".yellow(), serial);

    println!("{} {}", "Valid from:".yellow(), cert.not_before());
    println!("{} {}", "Valid to:".yellow(), cert.not_after());

    let now = Asn1Time::days_from_now(0)?;
    let diff = now.diff(cert.not_after())?;
    let total_seconds = i64::from(diff.days) * 86_400 + i64::from(diff.secs);

    if total_seconds > 30 * 86_400 {
        let days_remaining = (total_seconds + 86_399) / 86_400;
        println!(
            "Status:     {} Valid — {} days remaining",
            "✔".green(),
            days_remaining
        );
    } else if total_seconds > 0 {
        let days_remaining = (total_seconds + 86_399) / 86_400;
        println!(
            "Status:     {} Expiring soon — only {} days remaining",
            "⚠".yellow(),
            days_remaining
        );
    } else {
        let days_expired = ((-total_seconds) + 86_399) / 86_400;
        println!(
            "Status:     {} Expired {} days ago — needs renewal",
            "✘".red(),
            days_expired
        );
    }

    if prompt::confirm("Show full certificate (PEM)?")? {
        let pem = String::from_utf8(cert.to_pem()?)?;
        println!("\n{}\n{}", "PEM".bold(), pem);
    }

    Ok(())
}
