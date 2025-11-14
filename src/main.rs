use anyhow::{Context, Result};
use clap::Parser;
use omnitak_api::{ServerBuilder, ServerConfig};
use serde::Deserialize;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{error, info};

/// OmniTAK - High-performance TAK aggregator and message broker
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config/config.yaml")]
    config: PathBuf,

    /// Override bind address
    #[arg(short, long)]
    bind: Option<SocketAddr>,

    /// Admin username for initial setup
    #[arg(long, default_value = "admin")]
    admin_user: String,

    /// Admin password for initial setup
    #[arg(long, env = "OMNITAK_ADMIN_PASSWORD", default_value = "changeme")]
    admin_password: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    api: ApiConfig,
}

#[derive(Debug, Deserialize)]
struct ApiConfig {
    #[serde(default = "default_bind_addr")]
    bind_addr: String,
    #[serde(default = "default_enable_tls")]
    enable_tls: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            bind_addr: default_bind_addr(),
            enable_tls: default_enable_tls(),
        }
    }
}

fn default_bind_addr() -> String {
    "0.0.0.0:8443".to_string()
}

fn default_enable_tls() -> bool {
    false
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install default crypto provider for rustls
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Parse command line arguments
    let args = Args::parse();

    // Load configuration file
    let config_content = fs::read_to_string(&args.config)
        .with_context(|| format!("Failed to read config file: {:?}", args.config))?;

    let config: Config = serde_yaml::from_str(&config_content)
        .context("Failed to parse config file")?;

    // Build server configuration
    let bind_addr: SocketAddr = args
        .bind
        .unwrap_or_else(|| {
            config
                .api
                .bind_addr
                .parse()
                .expect("Invalid bind address in config")
        });

    let server_config = ServerConfig {
        bind_addr,
        enable_tls: config.api.enable_tls,
        tls_cert_path: None,
        tls_key_path: None,
        auth_config: Default::default(),
        rate_limit_rps: 100,
        enable_swagger: true,
        enable_static_files: true,
    };

    // Build and run server
    info!("Starting OmniTAK server");
    info!("Configuration loaded from {:?}", args.config);
    info!("Bind address: {}", bind_addr);
    info!("TLS enabled: {}", server_config.enable_tls);

    let server = ServerBuilder::new(server_config)
        .with_default_user(&args.admin_user, &args.admin_password)
        .build()?;

    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        return Err(e);
    }

    Ok(())
}
