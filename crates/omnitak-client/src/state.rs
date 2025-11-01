use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

/// Connection state for a TAK client
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Client is disconnected
    Disconnected,
    /// Client is attempting to connect
    Connecting,
    /// Client is connected and operational
    Connected,
    /// Client is reconnecting after a failure
    Reconnecting,
    /// Client encountered a fatal error
    Failed,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Disconnected => write!(f, "Disconnected"),
            ConnectionState::Connecting => write!(f, "Connecting"),
            ConnectionState::Connected => write!(f, "Connected"),
            ConnectionState::Reconnecting => write!(f, "Reconnecting"),
            ConnectionState::Failed => write!(f, "Failed"),
        }
    }
}

/// Metrics for tracking connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionMetrics {
    /// Total bytes sent
    bytes_sent: Arc<AtomicU64>,
    /// Total bytes received
    bytes_received: Arc<AtomicU64>,
    /// Total messages sent
    messages_sent: Arc<AtomicU64>,
    /// Total messages received
    messages_received: Arc<AtomicU64>,
    /// Total errors encountered
    errors: Arc<AtomicU64>,
    /// Number of reconnection attempts
    reconnect_attempts: Arc<AtomicUsize>,
    /// Last activity timestamp
    last_activity: Arc<parking_lot::RwLock<SystemTime>>,
    /// Connection established timestamp
    connected_at: Arc<parking_lot::RwLock<Option<SystemTime>>>,
}

impl Default for ConnectionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionMetrics {
    /// Create a new ConnectionMetrics instance
    pub fn new() -> Self {
        Self {
            bytes_sent: Arc::new(AtomicU64::new(0)),
            bytes_received: Arc::new(AtomicU64::new(0)),
            messages_sent: Arc::new(AtomicU64::new(0)),
            messages_received: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
            reconnect_attempts: Arc::new(AtomicUsize::new(0)),
            last_activity: Arc::new(parking_lot::RwLock::new(SystemTime::now())),
            connected_at: Arc::new(parking_lot::RwLock::new(None)),
        }
    }

    /// Record bytes sent
    pub fn record_bytes_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
        self.update_last_activity();
    }

    /// Record bytes received
    pub fn record_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
        self.update_last_activity();
    }

    /// Record a message sent
    pub fn record_message_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
        self.update_last_activity();
    }

    /// Record a message received
    pub fn record_message_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
        self.update_last_activity();
    }

    /// Record an error
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a reconnection attempt
    pub fn record_reconnect(&self) {
        self.reconnect_attempts.fetch_add(1, Ordering::Relaxed);
    }

    /// Reset reconnect attempts counter
    pub fn reset_reconnect_attempts(&self) {
        self.reconnect_attempts.store(0, Ordering::Relaxed);
    }

    /// Update last activity timestamp
    pub fn update_last_activity(&self) {
        *self.last_activity.write() = SystemTime::now();
    }

    /// Mark connection as established
    pub fn mark_connected(&self) {
        *self.connected_at.write() = Some(SystemTime::now());
        self.reset_reconnect_attempts();
    }

    /// Mark connection as disconnected
    pub fn mark_disconnected(&self) {
        *self.connected_at.write() = None;
    }

    /// Get total bytes sent
    pub fn bytes_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }

    /// Get total bytes received
    pub fn bytes_received(&self) -> u64 {
        self.bytes_received.load(Ordering::Relaxed)
    }

    /// Get total messages sent
    pub fn messages_sent(&self) -> u64 {
        self.messages_sent.load(Ordering::Relaxed)
    }

    /// Get total messages received
    pub fn messages_received(&self) -> u64 {
        self.messages_received.load(Ordering::Relaxed)
    }

    /// Get total errors
    pub fn errors(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }

    /// Get reconnect attempts
    pub fn reconnect_attempts(&self) -> usize {
        self.reconnect_attempts.load(Ordering::Relaxed)
    }

    /// Get last activity timestamp
    pub fn last_activity(&self) -> SystemTime {
        *self.last_activity.read()
    }

    /// Get connection established timestamp
    pub fn connected_at(&self) -> Option<SystemTime> {
        *self.connected_at.read()
    }

    /// Get time since last activity
    pub fn time_since_last_activity(&self) -> Duration {
        self.last_activity()
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
    }

    /// Get connection duration (if connected)
    pub fn connection_duration(&self) -> Option<Duration> {
        self.connected_at().and_then(|t| t.elapsed().ok())
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            bytes_sent: self.bytes_sent(),
            bytes_received: self.bytes_received(),
            messages_sent: self.messages_sent(),
            messages_received: self.messages_received(),
            errors: self.errors(),
            reconnect_attempts: self.reconnect_attempts(),
            last_activity: self.last_activity(),
            connected_at: self.connected_at(),
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.bytes_sent.store(0, Ordering::Relaxed);
        self.bytes_received.store(0, Ordering::Relaxed);
        self.messages_sent.store(0, Ordering::Relaxed);
        self.messages_received.store(0, Ordering::Relaxed);
        self.errors.store(0, Ordering::Relaxed);
        self.reconnect_attempts.store(0, Ordering::Relaxed);
        *self.last_activity.write() = SystemTime::now();
        *self.connected_at.write() = None;
    }
}

/// Snapshot of connection metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub errors: u64,
    pub reconnect_attempts: usize,
    pub last_activity: SystemTime,
    pub connected_at: Option<SystemTime>,
}

impl MetricsSnapshot {
    /// Calculate throughput in bytes per second
    pub fn throughput_bps(&self) -> Option<f64> {
        self.connected_at.and_then(|connected| {
            connected.elapsed().ok().map(|duration| {
                let secs = duration.as_secs_f64();
                if secs > 0.0 {
                    (self.bytes_sent + self.bytes_received) as f64 / secs
                } else {
                    0.0
                }
            })
        })
    }

    /// Calculate message rate (messages per second)
    pub fn message_rate(&self) -> Option<f64> {
        self.connected_at.and_then(|connected| {
            connected.elapsed().ok().map(|duration| {
                let secs = duration.as_secs_f64();
                if secs > 0.0 {
                    (self.messages_sent + self.messages_received) as f64 / secs
                } else {
                    0.0
                }
            })
        })
    }

    /// Calculate error rate (errors per message)
    pub fn error_rate(&self) -> f64 {
        let total_messages = self.messages_sent + self.messages_received;
        if total_messages > 0 {
            self.errors as f64 / total_messages as f64
        } else {
            0.0
        }
    }
}

/// Combined connection state and metrics
#[derive(Debug, Clone)]
pub struct ConnectionStatus {
    state: Arc<parking_lot::RwLock<ConnectionState>>,
    metrics: ConnectionMetrics,
    error_message: Arc<parking_lot::RwLock<Option<String>>>,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionStatus {
    /// Create a new ConnectionStatus
    pub fn new() -> Self {
        Self {
            state: Arc::new(parking_lot::RwLock::new(ConnectionState::Disconnected)),
            metrics: ConnectionMetrics::new(),
            error_message: Arc::new(parking_lot::RwLock::new(None)),
        }
    }

    /// Get current connection state
    pub fn state(&self) -> ConnectionState {
        *self.state.read()
    }

    /// Set connection state
    pub fn set_state(&self, state: ConnectionState) {
        *self.state.write() = state;
    }

    /// Get metrics reference
    pub fn metrics(&self) -> &ConnectionMetrics {
        &self.metrics
    }

    /// Set error message
    pub fn set_error(&self, error: String) {
        *self.error_message.write() = Some(error);
        self.set_state(ConnectionState::Failed);
    }

    /// Clear error message
    pub fn clear_error(&self) {
        *self.error_message.write() = None;
    }

    /// Get error message
    pub fn error_message(&self) -> Option<String> {
        self.error_message.read().clone()
    }

    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        matches!(self.state(), ConnectionState::Connected)
    }

    /// Check if in error state
    pub fn is_failed(&self) -> bool {
        matches!(self.state(), ConnectionState::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_display() {
        assert_eq!(ConnectionState::Connected.to_string(), "Connected");
        assert_eq!(ConnectionState::Disconnected.to_string(), "Disconnected");
    }

    #[test]
    fn test_metrics_recording() {
        let metrics = ConnectionMetrics::new();

        metrics.record_bytes_sent(100);
        metrics.record_bytes_received(200);
        metrics.record_message_sent();
        metrics.record_message_received();
        metrics.record_error();

        assert_eq!(metrics.bytes_sent(), 100);
        assert_eq!(metrics.bytes_received(), 200);
        assert_eq!(metrics.messages_sent(), 1);
        assert_eq!(metrics.messages_received(), 1);
        assert_eq!(metrics.errors(), 1);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = ConnectionMetrics::new();

        metrics.record_bytes_sent(100);
        metrics.record_message_sent();
        metrics.reset();

        assert_eq!(metrics.bytes_sent(), 0);
        assert_eq!(metrics.messages_sent(), 0);
    }

    #[test]
    fn test_connection_status() {
        let status = ConnectionStatus::new();

        assert_eq!(status.state(), ConnectionState::Disconnected);
        assert!(!status.is_connected());

        status.set_state(ConnectionState::Connected);
        assert!(status.is_connected());

        status.set_error("Test error".to_string());
        assert!(status.is_failed());
        assert_eq!(status.error_message(), Some("Test error".to_string()));
    }
}
