use anyhow::{Context, Result};
use clap::Parser;
use omnitak_api::{ServerBuilder, ServerConfig};
use omnitak_client::{TakClient, tls::{TlsClient, TlsClientConfig}};
use serde::Deserialize;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::signal;
use tokio_stream::StreamExt;
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
    #[serde(default)]
    servers: Vec<TakServerDef>,
}

#[derive(Debug, Deserialize)]
struct ApiConfig {
    #[serde(default = "default_bind_addr")]
    bind_addr: String,
    #[serde(default = "default_enable_tls")]
    enable_tls: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct TakServerDef {
    id: String,
    address: String,
    protocol: String,
    tls: Option<TlsConfigDef>,
}

#[derive(Debug, Deserialize, Clone)]
struct TlsConfigDef {
    cert_path: String,
    key_path: String,
    ca_path: String,
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
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

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

    // Start TAK server connections
    info!("Starting TAK server connections...");
    for server_def in &config.servers {
        info!("Connecting to TAK server: {} at {}", server_def.id, server_def.address);

        if server_def.protocol.to_lowercase() == "tls" {
            if let Some(ref tls_config) = server_def.tls {
                // Create TLS client
                let cert_path = PathBuf::from(&tls_config.cert_path);
                let key_path = PathBuf::from(&tls_config.key_path);
                let ca_path = PathBuf::from(&tls_config.ca_path);

                let mut client_config = TlsClientConfig::new(cert_path, key_path)
                    .with_ca_cert(ca_path);
                client_config.base.server_addr = server_def.address.clone();

                // Clone address for the async task
                let address = server_def.address.clone();

                match TlsClient::new(client_config) {
                    Ok(mut client) => {
                        tokio::spawn(async move {
                            info!("Connecting TLS client to {}", address);
                            if let Err(e) = client.connect().await {
                                error!("Failed to connect to TAK server: {}", e);
                            } else {
                                info!("Successfully connected to TAK server!");

                                // Read messages
                                let mut rx = client.receive_cot();
                                while let Some(result) = rx.next().await {
                                    match result {
                                        Ok(msg) => {
                                            info!("Received message: {} bytes", msg.data.len());
                                        }
                                        Err(e) => {
                                            error!("Error receiving message: {}", e);
                                            break;
                                        }
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to create TLS client: {}", e);
                    }
                }
            }
        }
    }

    // Build and run API server
    info!("Starting OmniTAK API server");
    info!("Configuration loaded from {:?}", args.config);
    info!("Bind address: {}", bind_addr);
    info!("TLS enabled: {}", server_config.enable_tls);

    let server = ServerBuilder::new(server_config)
        .with_default_user(&args.admin_user, &args.admin_password)
        .build()?;

    // Run server with graceful shutdown
    tokio::select! {
        result = server.run() => {
            if let Err(e) = result {
                error!("Server error: {}", e);
                return Err(e);
            }
        }
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal, stopping server...");
        }
    }

    Ok(())
}
