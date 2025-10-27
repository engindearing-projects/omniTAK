//! Example server demonstrating the OmniTAK API

use omnitak_api::{ServerBuilder, ServerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure server
    let config = ServerConfig {
        bind_addr: "0.0.0.0:8443".parse().unwrap(),
        enable_tls: false, // Disable TLS for development
        rate_limit_rps: 100,
        enable_swagger: true,
        enable_static_files: true,
        ..Default::default()
    };

    // Build and start server
    let server = ServerBuilder::new(config)
        .with_default_user("admin", "admin_password_123")
        .build()?;

    println!("Starting OmniTAK API server...");
    println!("API Documentation: http://localhost:8443/swagger-ui");
    println!("Web UI: http://localhost:8443/");
    println!("Health Check: http://localhost:8443/health");
    println!("\nDefault credentials:");
    println!("  Username: admin");
    println!("  Password: admin_password_123");
    println!("\nPress Ctrl+C to stop the server");

    server.run().await?;

    Ok(())
}
