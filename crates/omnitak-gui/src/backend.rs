//! Backend service for managing OmniTAK connections.
//!
//! This module provides the bridge between the GUI and the actual
//! TAK server connection logic.

use crate::{AppMetrics, ConnectionMetadata, MessageLog};
use async_channel::{Receiver, Sender, unbounded};
use omnitak_core::types::{ConnectionId, ServerConfig, ServerStatus};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

/// Commands that can be sent to the backend service.
#[derive(Debug, Clone)]
pub enum BackendCommand {
    /// Start a connection to a server
    Connect(ServerConfig),
    /// Stop a connection
    Disconnect(String), // server name
    /// Update connection configuration
    UpdateConfig(ServerConfig),
    /// Shutdown the backend
    Shutdown,
}

/// Events sent from the backend to the GUI.
#[derive(Debug, Clone)]
pub enum BackendEvent {
    /// Connection status changed
    StatusUpdate(String, ConnectionMetadata), // server name, metadata
    /// New message received
    MessageReceived(MessageLog),
    /// Error occurred
    Error(String, String), // server name, error message
    /// Metrics updated
    MetricsUpdate(AppMetrics),
}

/// Backend service that manages connections.
pub struct BackendService {
    /// Runtime for async operations
    runtime: Arc<Runtime>,
    /// Command sender
    command_tx: Sender<BackendCommand>,
    /// Event receiver
    event_rx: Receiver<BackendEvent>,
    /// Worker thread handle
    worker_handle: Option<std::thread::JoinHandle<()>>,
}

impl BackendService {
    /// Creates a new backend service.
    pub fn new() -> anyhow::Result<Self> {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()?,
        );

        let (command_tx, command_rx) = unbounded::<BackendCommand>();
        let (event_tx, event_rx) = unbounded::<BackendEvent>();

        // Spawn worker thread
        let worker_runtime = runtime.clone();
        let worker_handle = std::thread::spawn(move || {
            worker_runtime.block_on(async {
                if let Err(e) = run_worker(command_rx, event_tx).await {
                    tracing::error!("Worker error: {}", e);
                }
            });
        });

        Ok(Self {
            runtime,
            command_tx,
            event_rx,
            worker_handle: Some(worker_handle),
        })
    }

    /// Sends a command to the backend.
    pub fn send_command(&self, command: BackendCommand) -> anyhow::Result<()> {
        self.command_tx.try_send(command)?;
        Ok(())
    }

    /// Tries to receive an event from the backend (non-blocking).
    pub fn try_recv_event(&self) -> Option<BackendEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Receives an event from the backend (blocking).
    pub fn recv_event(&self) -> Option<BackendEvent> {
        self.event_rx.recv_blocking().ok()
    }

    /// Shuts down the backend service.
    pub fn shutdown(mut self) -> anyhow::Result<()> {
        let _ = self.command_tx.try_send(BackendCommand::Shutdown);

        if let Some(handle) = self.worker_handle.take() {
            handle.join().map_err(|_| anyhow::anyhow!("Failed to join worker thread"))?;
        }

        Ok(())
    }
}

impl Drop for BackendService {
    fn drop(&mut self) {
        let _ = self.command_tx.try_send(BackendCommand::Shutdown);
    }
}

/// Worker state
struct WorkerState {
    /// Active connections
    connections: HashMap<String, ConnectionHandle>,
    /// Event sender
    event_tx: Sender<BackendEvent>,
}

/// Handle to an active connection
struct ConnectionHandle {
    /// Connection ID
    id: ConnectionId,
    /// Server configuration
    config: ServerConfig,
    /// Metadata
    metadata: ConnectionMetadata,
    /// Cancellation token
    cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

/// Main worker loop
async fn run_worker(
    command_rx: Receiver<BackendCommand>,
    event_tx: Sender<BackendEvent>,
) -> anyhow::Result<()> {
    let mut state = WorkerState {
        connections: HashMap::new(),
        event_tx: event_tx.clone(),
    };

    tracing::info!("Backend worker started");

    loop {
        match command_rx.recv().await {
            Ok(BackendCommand::Connect(config)) => {
                tracing::info!("Connecting to server: {}", config.name);
                handle_connect(&mut state, config).await;
            }
            Ok(BackendCommand::Disconnect(server_name)) => {
                tracing::info!("Disconnecting from server: {}", server_name);
                handle_disconnect(&mut state, &server_name).await;
            }
            Ok(BackendCommand::UpdateConfig(config)) => {
                tracing::info!("Updating config for server: {}", config.name);
                handle_update_config(&mut state, config).await;
            }
            Ok(BackendCommand::Shutdown) => {
                tracing::info!("Shutting down backend worker");
                break;
            }
            Err(_) => {
                tracing::info!("Command channel closed, shutting down");
                break;
            }
        }

        // Update metrics
        update_metrics(&state).await;
    }

    // Clean up all connections
    for (name, _) in state.connections.drain() {
        tracing::info!("Cleaning up connection: {}", name);
    }

    Ok(())
}

/// Handles connection command
async fn handle_connect(state: &mut WorkerState, config: ServerConfig) {
    let id = ConnectionId::new();
    let mut metadata = ConnectionMetadata::new(id, config.name.clone());

    // Simulate connection attempt for now
    // TODO: Integrate with actual omnitak-client
    metadata.mark_connected();

    let _ = state.event_tx.send(BackendEvent::StatusUpdate(
        config.name.clone(),
        metadata.clone(),
    )).await;

    state.connections.insert(
        config.name.clone(),
        ConnectionHandle {
            id,
            config,
            metadata,
            cancel_tx: None,
        },
    );
}

/// Handles disconnect command
async fn handle_disconnect(state: &mut WorkerState, server_name: &str) {
    if let Some(mut handle) = state.connections.remove(server_name) {
        // Send cancellation signal
        if let Some(cancel_tx) = handle.cancel_tx.take() {
            let _ = cancel_tx.send(());
        }

        handle.metadata.mark_disconnected(None);
        let _ = state.event_tx.send(BackendEvent::StatusUpdate(
            server_name.to_string(),
            handle.metadata,
        )).await;
    }
}

/// Handles config update command
async fn handle_update_config(state: &mut WorkerState, config: ServerConfig) {
    // Disconnect if connected
    handle_disconnect(state, &config.name).await;

    // Reconnect with new config if enabled
    if config.enabled {
        handle_connect(state, config).await;
    }
}

/// Updates and sends metrics
async fn update_metrics(state: &WorkerState) {
    let mut metrics = AppMetrics::default();

    for handle in state.connections.values() {
        metrics.total_messages_received += handle.metadata.messages_received;
        metrics.total_messages_sent += handle.metadata.messages_sent;
        metrics.total_bytes_received += handle.metadata.bytes_received;
        metrics.total_bytes_sent += handle.metadata.bytes_sent;

        if handle.metadata.status == ServerStatus::Connected {
            metrics.active_connections += 1;
        } else if handle.metadata.status == ServerStatus::Failed {
            metrics.failed_connections += 1;
        }
    }

    let _ = state.event_tx.send(BackendEvent::MetricsUpdate(metrics)).await;
}
