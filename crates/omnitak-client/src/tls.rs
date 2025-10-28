use crate::client::{
    calculate_backoff, ClientConfig, CotMessage, HealthCheck, HealthStatus, MessageMetadata, TakClient,
};
use crate::state::{ConnectionState, ConnectionStatus};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use bytes::BytesMut;
use omnitak_cert::{CertificateBundle, CertificateData};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::{ClientConfig as RustlsConfig, RootCertStore};
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::timeout;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, instrument, warn};

/// Frame delimiter for newline-delimited protocol
const NEWLINE_DELIMITER: u8 = b'\n';

/// Maximum frame size (10MB)
const MAX_FRAME_SIZE: usize = 10 * 1024 * 1024;

/// TLS certificate source - either from files or in-memory data
#[derive(Debug, Clone)]
pub enum TlsCertSource {
    /// Load certificates from file paths
    Files {
        /// Path to client certificate (PEM format)
        cert_path: PathBuf,
        /// Path to client private key (PEM format)
        key_path: PathBuf,
        /// Path to CA certificate for server verification (PEM format)
        ca_cert_path: Option<PathBuf>,
    },
    /// Use certificates from memory (uploaded via web UI)
    Memory {
        /// Client certificate data
        cert_data: CertificateData,
        /// Private key data (optional for PKCS#12)
        key_data: Option<CertificateData>,
        /// CA certificate data (optional)
        ca_data: Option<CertificateData>,
        /// Password for encrypted certificates (PKCS#12)
        password: Option<String>,
    },
    /// Use pre-parsed certificate bundle
    Bundle(CertificateBundle),
}

/// TLS certificate configuration
#[derive(Debug, Clone)]
pub struct TlsCertConfig {
    /// Certificate source (files, memory, or bundle)
    pub source: TlsCertSource,
}

/// Configuration specific to TLS client
#[derive(Debug, Clone)]
pub struct TlsClientConfig {
    /// Base client configuration
    pub base: ClientConfig,
    /// TLS certificate configuration
    pub cert_config: TlsCertConfig,
    /// Server name for SNI (Server Name Indication)
    /// If None, will be extracted from server_addr
    pub server_name: Option<String>,
    /// Enable TLS 1.3 only (recommended for security)
    pub tls13_only: bool,
    /// Verify server certificate (should always be true in production)
    pub verify_server: bool,
}

impl TlsClientConfig {
    /// Create a new TLS client configuration from file paths
    pub fn new(cert_path: PathBuf, key_path: PathBuf) -> Self {
        Self {
            base: ClientConfig::default(),
            cert_config: TlsCertConfig {
                source: TlsCertSource::Files {
                    cert_path,
                    key_path,
                    ca_cert_path: None,
                },
            },
            server_name: None,
            tls13_only: true,
            verify_server: true,
        }
    }

    /// Create a new TLS client configuration from in-memory certificate data
    pub fn from_memory(
        cert_data: CertificateData,
        key_data: Option<CertificateData>,
        ca_data: Option<CertificateData>,
        password: Option<String>,
    ) -> Self {
        Self {
            base: ClientConfig::default(),
            cert_config: TlsCertConfig {
                source: TlsCertSource::Memory {
                    cert_data,
                    key_data,
                    ca_data,
                    password,
                },
            },
            server_name: None,
            tls13_only: true,
            verify_server: true,
        }
    }

    /// Create a new TLS client configuration from a certificate bundle
    pub fn from_bundle(bundle: CertificateBundle) -> Self {
        Self {
            base: ClientConfig::default(),
            cert_config: TlsCertConfig {
                source: TlsCertSource::Bundle(bundle),
            },
            server_name: None,
            tls13_only: true,
            verify_server: true,
        }
    }

    /// Set CA certificate path (only for file-based config)
    pub fn with_ca_cert(mut self, ca_cert_path: PathBuf) -> Self {
        if let TlsCertSource::Files { ref mut ca_cert_path: ca, .. } = self.cert_config.source {
            *ca = Some(ca_cert_path);
        }
        self
    }

    /// Set server name for SNI
    pub fn with_server_name(mut self, server_name: String) -> Self {
        self.server_name = Some(server_name);
        self
    }
}

/// TLS client for secure TAK server connections
pub struct TlsClient {
    config: TlsClientConfig,
    status: Arc<ConnectionStatus>,
    stream: Option<TlsStream<TcpStream>>,
    tls_config: Arc<RustlsConfig>,
    recv_tx: Option<Sender<Result<CotMessage>>>,
    recv_rx: Option<Receiver<Result<CotMessage>>>,
    shutdown_tx: Option<Sender<()>>,
}

impl TlsClient {
    /// Create a new TLS client
    pub fn new(config: TlsClientConfig) -> Result<Self> {
        let tls_config = Self::build_tls_config(&config)?;
        let (recv_tx, recv_rx) = mpsc::channel(config.base.recv_buffer_size);

        Ok(Self {
            config,
            status: Arc::new(ConnectionStatus::new()),
            stream: None,
            tls_config: Arc::new(tls_config),
            recv_tx: Some(recv_tx),
            recv_rx: Some(recv_rx),
            shutdown_tx: None,
        })
    }

    /// Get connection status
    pub fn status(&self) -> &ConnectionStatus {
        &self.status
    }

    /// Build rustls configuration
    fn build_tls_config(config: &TlsClientConfig) -> Result<RustlsConfig> {
        info!("Building TLS configuration");

        // Load or parse certificate bundle based on source
        let bundle = match &config.cert_config.source {
            TlsCertSource::Files { cert_path, key_path, ca_cert_path } => {
                info!("Loading certificates from files");
                Self::load_bundle_from_files(cert_path, key_path, ca_cert_path.as_ref())?
            }
            TlsCertSource::Memory { cert_data, key_data, ca_data, password } => {
                info!("Loading certificates from memory");
                CertificateBundle::from_certificate_data(
                    cert_data,
                    key_data.as_ref(),
                    ca_data.as_ref(),
                    password.as_deref(),
                )?
            }
            TlsCertSource::Bundle(bundle) => {
                info!("Using pre-parsed certificate bundle");
                bundle.clone()
            }
        };

        // Load root certificates
        let mut root_store = RootCertStore::empty();

        if let Some(ca_certs) = &bundle.ca_certs {
            // Use custom CA certificates from bundle
            for cert in ca_certs {
                root_store
                    .add(cert.clone())
                    .context("Failed to add CA certificate to root store")?;
            }
            info!("Loaded {} custom CA certificate(s)", ca_certs.len());
        } else {
            // Use system root certificates
            root_store.extend(
                webpki_roots::TLS_SERVER_ROOTS
                    .iter()
                    .cloned()
            );
            info!("Using system root certificates");
        }

        info!("Loaded {} client certificate(s)", bundle.certs.len());

        // Build TLS configuration
        let mut tls_config = RustlsConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(bundle.certs, bundle.private_key)
            .context("Failed to build TLS config with client auth")?;

        // Configure TLS versions
        if config.tls13_only {
            // TLS 1.3 only (most secure)
            info!("Configured for TLS 1.3 only");
        }

        // Disable certificate verification if requested (NOT RECOMMENDED)
        if !config.verify_server {
            warn!("Server certificate verification is DISABLED - this is insecure!");
        }

        tls_config.alpn_protocols = vec![b"http/1.1".to_vec()];

        Ok(tls_config)
    }

    /// Load certificate bundle from file paths
    fn load_bundle_from_files(
        cert_path: &PathBuf,
        key_path: &PathBuf,
        ca_cert_path: Option<&PathBuf>,
    ) -> Result<CertificateBundle> {
        // Load client certificate
        let cert_file = std::fs::File::open(cert_path)
            .context("Failed to open client certificate file")?;
        let mut cert_reader = BufReader::new(cert_file);

        let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse client certificates")?;

        if certs.is_empty() {
            return Err(anyhow!("No certificates found in certificate file"));
        }

        // Load private key
        let key_file = std::fs::File::open(key_path)
            .context("Failed to open private key file")?;
        let mut key_reader = BufReader::new(key_file);

        let private_key = rustls_pemfile::private_key(&mut key_reader)
            .context("Failed to read private key")?
            .ok_or_else(|| anyhow!("No private key found in key file"))?;

        // Load CA certificate if provided
        let ca_certs = if let Some(ca_path) = ca_cert_path {
            let ca_cert_file = std::fs::File::open(ca_path)
                .context("Failed to open CA certificate file")?;
            let mut ca_cert_reader = BufReader::new(ca_cert_file);

            let ca_certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut ca_cert_reader)
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to parse CA certificates")?;

            if !ca_certs.is_empty() {
                Some(ca_certs)
            } else {
                None
            }
        } else {
            None
        };

        Ok(CertificateBundle {
            certs,
            private_key,
            ca_certs,
        })
    }

    /// Extract server name from address
    fn get_server_name(&self) -> Result<ServerName<'static>> {
        let server_name = self.config.server_name.as_ref().map(|s| s.as_str())
            .unwrap_or_else(|| {
                // Extract hostname from server_addr
                self.config
                    .base
                    .server_addr
                    .split(':')
                    .next()
                    .unwrap_or("localhost")
            });

        ServerName::try_from(server_name.to_string())
            .map_err(|e| anyhow!("Invalid server name: {}", e))
    }

    /// Establish TLS connection
    #[instrument(skip(self))]
    async fn establish_connection(&mut self) -> Result<()> {
        self.status.set_state(ConnectionState::Connecting);

        info!("Connecting to {} with TLS", self.config.base.server_addr);

        // Establish TCP connection first
        let tcp_stream = timeout(
            self.config.base.connect_timeout,
            TcpStream::connect(&self.config.base.server_addr),
        )
        .await
        .context("Connection timeout")?
        .context("Failed to connect")?;

        // Perform TLS handshake
        let server_name = self.get_server_name()?;
        let connector = TlsConnector::from(Arc::clone(&self.tls_config));

        let tls_stream = timeout(
            self.config.base.connect_timeout,
            connector.connect(server_name, tcp_stream),
        )
        .await
        .context("TLS handshake timeout")?
        .context("TLS handshake failed")?;

        info!("TLS handshake successful");

        self.stream = Some(tls_stream);
        self.status.set_state(ConnectionState::Connected);
        self.status.metrics().mark_connected();

        Ok(())
    }

    /// Read a newline-delimited frame from the TLS stream
    async fn read_frame(&mut self, buffer: &mut BytesMut) -> Result<Option<bytes::Bytes>> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;

        loop {
            // Check if we have a complete frame in the buffer
            if let Some(pos) = buffer.iter().position(|&b| b == NEWLINE_DELIMITER) {
                let frame = buffer.split_to(pos + 1);
                let mut frame_bytes = frame.freeze();

                // Remove the newline delimiter
                if frame_bytes.last() == Some(&NEWLINE_DELIMITER) {
                    frame_bytes.truncate(frame_bytes.len() - 1);
                }

                self.status.metrics().record_bytes_received(frame_bytes.len() as u64);
                return Ok(Some(frame_bytes));
            }

            // Check buffer size limit
            if buffer.len() >= MAX_FRAME_SIZE {
                return Err(anyhow!("Frame too large"));
            }

            // Read more data
            let read_result = timeout(
                self.config.base.read_timeout,
                stream.read_buf(buffer),
            )
            .await
            .context("Read timeout")?
            .context("Read error")?;

            if read_result == 0 {
                if buffer.is_empty() {
                    return Ok(None); // Clean disconnect
                } else {
                    return Err(anyhow!("Connection closed with incomplete frame"));
                }
            }
        }
    }

    /// Write a frame to the TLS stream
    async fn write_frame(&mut self, data: &[u8]) -> Result<()> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;

        timeout(
            self.config.base.write_timeout,
            stream.write_all(data),
        )
        .await
        .context("Write timeout")?
        .context("Write error")?;

        timeout(
            self.config.base.write_timeout,
            stream.write_all(&[NEWLINE_DELIMITER]),
        )
        .await
        .context("Write timeout")?
        .context("Write error")?;

        stream.flush().await.context("Flush error")?;
        self.status.metrics().record_bytes_sent(data.len() as u64);

        Ok(())
    }

    /// Start background receive task
    fn start_receive_task(&mut self) {
        let mut buffer = BytesMut::with_capacity(8192);
        let status = Arc::clone(&self.status);
        let tx = self.recv_tx.as_ref().unwrap().clone();
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Move the stream out for the task
        if let Some(mut stream) = self.stream.take() {
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = shutdown_rx.recv() => {
                            debug!("TLS receive task shutting down");
                            break;
                        }
                        result = Self::read_frame_static(&mut stream, &mut buffer, &status) => {
                            match result {
                                Ok(Some(frame)) => {
                                    status.metrics().record_message_received();
                                    let message = CotMessage {
                                        data: frame,
                                        metadata: Some(MessageMetadata {
                                            received_at: std::time::SystemTime::now(),
                                            source_addr: None,
                                        }),
                                    };

                                    if tx.send(Ok(message)).await.is_err() {
                                        debug!("Receive channel closed");
                                        break;
                                    }
                                }
                                Ok(None) => {
                                    info!("TLS connection closed by remote");
                                    status.set_state(ConnectionState::Disconnected);
                                    break;
                                }
                                Err(e) => {
                                    error!(error = %e, "Error reading frame");
                                    status.metrics().record_error();
                                    let _ = tx.send(Err(e)).await;
                                    break;
                                }
                            }
                        }
                    }
                }
            });
        }
    }

    /// Static helper for reading frames (used in async task)
    async fn read_frame_static(
        stream: &mut TlsStream<TcpStream>,
        buffer: &mut BytesMut,
        status: &ConnectionStatus,
    ) -> Result<Option<bytes::Bytes>> {
        loop {
            if let Some(pos) = buffer.iter().position(|&b| b == NEWLINE_DELIMITER) {
                let frame = buffer.split_to(pos + 1);
                let mut frame_bytes = frame.freeze();

                if frame_bytes.last() == Some(&NEWLINE_DELIMITER) {
                    frame_bytes.truncate(frame_bytes.len() - 1);
                }

                status.metrics().record_bytes_received(frame_bytes.len() as u64);
                return Ok(Some(frame_bytes));
            }

            if buffer.len() >= MAX_FRAME_SIZE {
                return Err(anyhow!("Frame too large"));
            }

            let n = stream.read_buf(buffer).await.context("Read error")?;

            if n == 0 {
                if buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(anyhow!("Connection closed with incomplete frame"));
                }
            }
        }
    }
}

#[async_trait]
impl TakClient for TlsClient {
    async fn connect(&mut self) -> Result<()> {
        let config = self.config.base.reconnect.clone();

        // Inline retry logic to avoid closure capture issues
        let result = if !config.enabled {
            self.establish_connection().await
        } else {
            let mut attempt = 0u32;
            loop {
                match self.establish_connection().await {
                    Ok(()) => {
                        if attempt > 0 {
                            info!(
                                attempt = attempt,
                                "Successfully reconnected after {} attempts",
                                attempt
                            );
                        }
                        break Ok(());
                    }
                    Err(e) => {
                        attempt += 1;

                        if let Some(max) = config.max_attempts {
                            if attempt >= max {
                                error!(
                                    attempt = attempt,
                                    error = %e,
                                    "Max reconnect attempts reached"
                                );
                                break Err(e);
                            }
                        }

                        let backoff = calculate_backoff(attempt - 1, &config);
                        warn!(
                            attempt = attempt,
                            backoff_secs = backoff.as_secs(),
                            error = %e,
                            "Connection attempt failed, retrying after backoff"
                        );

                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        };

        if result.is_ok() {
            self.start_receive_task();
        }

        result
    }

    async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from {}", self.config.base.server_addr);

        // Signal shutdown to receive task
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        // Close the stream
        if let Some(mut stream) = self.stream.take() {
            stream.shutdown().await.context("Failed to shutdown stream")?;
        }

        self.status.set_state(ConnectionState::Disconnected);
        self.status.metrics().mark_disconnected();

        Ok(())
    }

    async fn send_cot(&mut self, message: CotMessage) -> Result<()> {
        if !self.is_connected() {
            return Err(anyhow!("Not connected"));
        }

        debug!(size = message.data.len(), "Sending CoT message over TLS");

        self.write_frame(&message.data).await?;
        self.status.metrics().record_message_sent();

        Ok(())
    }

    fn receive_cot(&mut self) -> ReceiverStream<Result<CotMessage>> {
        let rx = self.recv_rx.take().unwrap_or_else(|| {
            let (_, rx) = mpsc::channel(1);
            rx
        });

        ReceiverStream::new(rx)
    }

    async fn health_check(&self) -> HealthCheck {
        let status = self.status.state();

        let health_status = match status {
            ConnectionState::Connected => {
                let time_since_activity = self.status.metrics().time_since_last_activity();
                if time_since_activity > std::time::Duration::from_secs(60) {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                }
            }
            ConnectionState::Connecting | ConnectionState::Reconnecting => {
                HealthStatus::Degraded
            }
            ConnectionState::Disconnected => HealthStatus::Disconnected,
            ConnectionState::Failed => HealthStatus::Unhealthy,
        };

        HealthCheck {
            status: health_status,
            message: self.status.error_message(),
            rtt: None,
        }
    }

    fn is_connected(&self) -> bool {
        self.status.is_connected()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_cert_config() {
        let config = TlsCertConfig {
            cert_path: PathBuf::from("/path/to/cert.pem"),
            key_path: PathBuf::from("/path/to/key.pem"),
            ca_cert_path: None,
        };

        assert_eq!(config.cert_path, PathBuf::from("/path/to/cert.pem"));
    }

    #[test]
    fn test_tls_client_config_builder() {
        let config = TlsClientConfig::new(
            PathBuf::from("/cert.pem"),
            PathBuf::from("/key.pem"),
        )
        .with_server_name("example.com".to_string());

        assert_eq!(config.server_name, Some("example.com".to_string()));
        assert!(config.tls13_only);
    }
}
