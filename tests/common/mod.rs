//! Common test utilities and helpers for integration tests

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Counter for generating unique UIDs
static UID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Counter for generating unique ports
static PORT_COUNTER: AtomicU64 = AtomicU64::new(50000);

/// Generate a unique UID for testing
pub fn generate_unique_uid() -> String {
    let id = UID_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("test-uid-{}", id)
}

/// Get a unique port for testing to avoid conflicts
pub fn get_unique_port() -> u16 {
    let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
    (port % 15000 + 50000) as u16
}

/// Generate a valid CoT XML message with specified UID
pub fn generate_cot_message(uid: &str) -> Vec<u8> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let start_time = format_timestamp(now);
    let stale_time = format_timestamp(now + 300); // 5 minutes stale

    format!(
        r#"<?xml version="1.0"?>
<event version="2.0" uid="{}" type="a-f-G" time="{}" start="{}" stale="{}" how="h-e">
    <point lat="37.7749" lon="-122.4194" hae="100.0" ce="10.0" le="5.0"/>
    <detail>
        <contact callsign="TEST-{}"/>
    </detail>
</event>"#,
        uid, start_time, start_time, stale_time, uid
    )
    .into_bytes()
}

/// Generate a CoT message with custom properties
pub fn generate_cot_with_properties(
    uid: &str,
    lat: f64,
    lon: f64,
    cot_type: &str,
    callsign: &str,
) -> Vec<u8> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let start_time = format_timestamp(now);
    let stale_time = format_timestamp(now + 300);

    format!(
        r#"<?xml version="1.0"?>
<event version="2.0" uid="{}" type="{}" time="{}" start="{}" stale="{}" how="h-e">
    <point lat="{}" lon="{}" hae="100.0" ce="10.0" le="5.0"/>
    <detail>
        <contact callsign="{}"/>
    </detail>
</event>"#,
        uid, cot_type, start_time, start_time, stale_time, lat, lon, callsign
    )
    .into_bytes()
}

/// Format a Unix timestamp as ISO8601
fn format_timestamp(secs: u64) -> String {
    let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    datetime.format("%Y-%m-%dT%H:%M:%S.%fZ").to_string()
}

/// Extract UID from CoT XML message
pub fn extract_uid_from_cot(data: &[u8]) -> Option<String> {
    let msg_str = String::from_utf8_lossy(data);

    if let Some(start) = msg_str.find("uid=\"") {
        let uid_start = start + 5;
        if let Some(end) = msg_str[uid_start..].find('"') {
            return Some(msg_str[uid_start..uid_start + end].to_string());
        }
    }

    None
}

/// Mock TAK client that can send and receive messages
pub struct MockTakClient {
    write_half: tokio::io::WriteHalf<TcpStream>,
    rx_messages: mpsc::UnboundedReceiver<Vec<u8>>,
    local_addr: std::net::SocketAddr,
    _rx_task: JoinHandle<()>,
}

impl MockTakClient {
    /// Connect to a TAK server
    pub async fn connect(addr: &str) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let local_addr = stream.local_addr()?;
        let (tx, rx_messages) = mpsc::unbounded_channel();

        // Split stream into read and write halves
        let (mut read_half, write_half) = tokio::io::split(stream);

        // Spawn reader task
        let rx_task = tokio::spawn(async move {
            let mut buffer = vec![0u8; 8192];
            loop {
                match read_half.read(&mut buffer).await {
                    Ok(0) => break, // Connection closed
                    Ok(n) => {
                        let data = buffer[..n].to_vec();
                        if tx.send(data).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            write_half,
            rx_messages,
            local_addr,
            _rx_task: rx_task,
        })
    }

    /// Send a CoT message
    pub async fn send_cot(&mut self, cot_xml: &[u8]) -> anyhow::Result<()> {
        self.write_half.write_all(cot_xml).await?;
        self.write_half.flush().await?;
        Ok(())
    }

    /// Try to receive a message with timeout
    pub async fn recv_with_timeout(&mut self, timeout: Duration) -> Option<Vec<u8>> {
        tokio::time::timeout(timeout, self.rx_messages.recv())
            .await
            .ok()?
    }

    /// Get local address
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        Ok(self.local_addr)
    }
}

/// Mock TAK server that accepts connections and forwards messages
pub struct MockTakServer {
    local_addr: std::net::SocketAddr,
    clients: Arc<tokio::sync::Mutex<Vec<TcpStream>>>,
    stop_signal: Arc<tokio::sync::Notify>,
    _server_task: JoinHandle<()>,
}

impl MockTakServer {
    /// Start a mock TAK server on the given address
    pub async fn start(addr: &str) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        let clients = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let stop_signal = Arc::new(tokio::sync::Notify::new());

        let clients_clone = Arc::clone(&clients);
        let stop_clone = Arc::clone(&stop_signal);

        let server_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, _addr)) => {
                                clients_clone.lock().await.push(stream);
                            }
                            Err(_) => break,
                        }
                    }
                    _ = stop_clone.notified() => {
                        break;
                    }
                }
            }
        });

        Ok(Self {
            local_addr,
            clients,
            stop_signal,
            _server_task: server_task,
        })
    }

    /// Get the local address the server is listening on
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        Ok(self.local_addr)
    }

    /// Get the number of connected clients
    pub async fn client_count(&self) -> usize {
        self.clients.lock().await.len()
    }

    /// Broadcast a message to all connected clients
    pub async fn broadcast(&self, message: &[u8]) -> usize {
        let mut clients = self.clients.lock().await;
        let mut sent_count = 0;

        for client in clients.iter_mut() {
            if client.write_all(message).await.is_ok() {
                let _ = client.flush().await;
                sent_count += 1;
            }
        }

        sent_count
    }

    /// Stop the server
    pub async fn stop(self) {
        self.stop_signal.notify_one();
        // Wait a bit for cleanup
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Test fixture for setting up a complete test environment
pub struct TestEnvironment {
    pub pool: Arc<omnitak_pool::ConnectionPool>,
    pub distributor: Arc<omnitak_pool::MessageDistributor>,
    pub aggregator: Arc<omnitak_pool::MessageAggregator>,
    pub metrics: Arc<omnitak_pool::MetricsRegistry>,
}

impl TestEnvironment {
    /// Create a new test environment with default configuration
    pub async fn new() -> Self {
        Self::with_config(
            omnitak_pool::PoolConfig::default(),
            omnitak_pool::DistributorConfig::default(),
            omnitak_pool::AggregatorConfig::default(),
        )
        .await
    }

    /// Create a test environment with custom configuration
    pub async fn with_config(
        pool_config: omnitak_pool::PoolConfig,
        dist_config: omnitak_pool::DistributorConfig,
        agg_config: omnitak_pool::AggregatorConfig,
    ) -> Self {
        let metrics = Arc::new(omnitak_pool::MetricsRegistry::new(
            omnitak_pool::MetricsConfig {
                enabled: false, // Don't start HTTP server in tests
                ..Default::default()
            },
        ));

        let pool = Arc::new(omnitak_pool::ConnectionPool::new(pool_config));
        let distributor = Arc::new(omnitak_pool::MessageDistributor::new(
            Arc::clone(&pool),
            dist_config,
        ));
        let aggregator = Arc::new(omnitak_pool::MessageAggregator::new(
            Arc::clone(&distributor),
            agg_config,
        ));

        // Start distributor and aggregator
        distributor.start().await;
        aggregator.start().await;

        Self {
            pool,
            distributor,
            aggregator,
            metrics,
        }
    }

    /// Add a test connection to the pool
    pub async fn add_connection(
        &self,
        id: &str,
        priority: u8,
    ) -> anyhow::Result<String> {
        self.pool
            .add_connection(
                id.to_string(),
                format!("Test Connection {}", id),
                format!("localhost:{}", get_unique_port()),
                priority,
            )
            .await
    }

    /// Shutdown the test environment
    pub async fn shutdown(self) {
        self.aggregator.stop().await;
        self.distributor.stop().await;
        let _ = self.pool.shutdown().await;
    }
}

/// Wait for a condition with timeout
pub async fn wait_for_condition<F>(
    condition: F,
    timeout: Duration,
    check_interval: Duration,
) -> bool
where
    F: Fn() -> bool,
{
    let start = tokio::time::Instant::now();

    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        tokio::time::sleep(check_interval).await;
    }

    false
}

/// Initialize tracing for tests (call once per test)
pub fn init_test_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_unique_uid() {
        let uid1 = generate_unique_uid();
        let uid2 = generate_unique_uid();
        assert_ne!(uid1, uid2);
    }

    #[test]
    fn test_generate_cot_message() {
        let uid = "test-123";
        let cot = generate_cot_message(uid);
        let cot_str = String::from_utf8(cot).unwrap();
        assert!(cot_str.contains("uid=\"test-123\""));
        assert!(cot_str.contains("<event"));
        assert!(cot_str.contains("</event>"));
    }

    #[test]
    fn test_extract_uid_from_cot() {
        let cot = generate_cot_message("my-uid-456");
        let uid = extract_uid_from_cot(&cot);
        assert_eq!(uid, Some("my-uid-456".to_string()));
    }

    #[tokio::test]
    async fn test_mock_server_start() {
        let port = get_unique_port();
        let server = MockTakServer::start(&format!("127.0.0.1:{}", port))
            .await
            .unwrap();

        assert!(server.local_addr().is_ok());
        server.stop().await;
    }

    #[tokio::test]
    async fn test_test_environment() {
        let env = TestEnvironment::new().await;

        assert_eq!(env.pool.connection_count(), 0);

        let conn_id = env.add_connection("test-1", 5).await.unwrap();
        assert_eq!(env.pool.connection_count(), 1);

        env.pool.remove_connection(&conn_id).await.unwrap();
        assert_eq!(env.pool.connection_count(), 0);

        env.shutdown().await;
    }
}
