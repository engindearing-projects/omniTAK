//! GUI application for OmniTAK TAK server aggregator.
//!
//! This crate provides a native desktop GUI built with egui/eframe for managing
//! OmniTAK server connections, viewing status, and monitoring message flow.

use eframe::egui;
use omnitak_core::types::{
    ConnectionId, ConnectionMetadata, Protocol, ServerConfig, ServerStatus, TlsConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod ui;
use ui::*;

pub mod backend;
use backend::{BackendCommand, BackendEvent, BackendService};

pub mod config_io;
pub use config_io::{export_config, import_config, ConfigFile};

/// Main application state for the OmniTAK GUI.
pub struct OmniTakApp {
    /// Configuration state
    pub state: Arc<Mutex<AppState>>,

    /// UI state
    pub ui_state: UiState,

    /// Backend service
    pub backend: Option<BackendService>,

    /// Status message for user notifications
    pub status_message: Option<(String, StatusLevel)>,

    /// Timestamp for status message expiry
    pub status_message_expiry: Option<std::time::Instant>,
}

/// Status message level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl Default for OmniTakApp {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(AppState::default())),
            ui_state: UiState::default(),
            backend: None,
            status_message: None,
            status_message_expiry: None,
        }
    }
}

/// Application state (shared between UI and backend).
#[derive(Default, Serialize, Deserialize)]
pub struct AppState {
    /// Server configurations
    pub servers: Vec<ServerConfig>,

    /// Connection metadata indexed by server name
    pub connections: HashMap<String, ConnectionMetadata>,

    /// Message log (recent messages)
    pub message_log: Vec<MessageLog>,

    /// Application metrics
    pub metrics: AppMetrics,
}

/// Application metrics.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AppMetrics {
    /// Total messages received across all connections
    pub total_messages_received: u64,

    /// Total messages sent across all connections
    pub total_messages_sent: u64,

    /// Total bytes received
    pub total_bytes_received: u64,

    /// Total bytes sent
    pub total_bytes_sent: u64,

    /// Active connections count
    pub active_connections: usize,

    /// Failed connections count
    pub failed_connections: usize,
}

/// Message log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageLog {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Server name
    pub server: String,

    /// Message content (truncated)
    pub content: String,

    /// Message type
    pub msg_type: String,
}

/// UI state (not serialized).
#[derive(Default)]
pub struct UiState {
    /// Currently selected tab
    pub selected_tab: Tab,

    /// Add/Edit server dialog state
    pub server_dialog: Option<ServerDialogState>,

    /// Filter for message log
    pub message_filter: String,

    /// Auto-scroll message log
    pub auto_scroll: bool,
}

/// Application tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Connections,
    Messages,
    Settings,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Dashboard
    }
}

/// Server add/edit dialog state.
pub struct ServerDialogState {
    /// Server being edited (None for new server)
    pub editing_index: Option<usize>,

    /// Server configuration being edited
    pub config: ServerConfig,

    /// TLS configuration dialog state
    pub tls_enabled: bool,
    pub ca_cert_path: String,
    pub client_cert_path: String,
    pub client_key_path: String,
    pub verify_cert: bool,
    pub server_name: String,
}

impl ServerDialogState {
    /// Creates a new dialog for adding a server.
    pub fn new() -> Self {
        Self {
            editing_index: None,
            config: ServerConfig::builder()
                .name("New Server")
                .host("localhost")
                .port(8089)
                .protocol(Protocol::Tls)
                .enabled(true)
                .build(),
            tls_enabled: true,
            ca_cert_path: String::new(),
            client_cert_path: String::new(),
            client_key_path: String::new(),
            verify_cert: true,
            server_name: String::new(),
        }
    }

    /// Creates a dialog for editing an existing server.
    pub fn edit(index: usize, config: ServerConfig) -> Self {
        let (
            tls_enabled,
            ca_cert_path,
            client_cert_path,
            client_key_path,
            verify_cert,
            server_name,
        ) = if let Some(tls) = &config.tls {
            (
                true,
                tls.ca_cert_path.to_string_lossy().to_string(),
                tls.client_cert_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                tls.client_key_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                tls.verify_cert,
                tls.server_name.clone().unwrap_or_default(),
            )
        } else {
            (
                false,
                String::new(),
                String::new(),
                String::new(),
                true,
                String::new(),
            )
        };

        Self {
            editing_index: Some(index),
            config,
            tls_enabled,
            ca_cert_path,
            client_cert_path,
            client_key_path,
            verify_cert,
            server_name,
        }
    }

    /// Builds the final server config from the dialog state.
    pub fn build(&self) -> ServerConfig {
        let mut config = self.config.clone();

        if self.tls_enabled && !self.ca_cert_path.is_empty() {
            let mut tls = TlsConfig::new(PathBuf::from(&self.ca_cert_path));

            if !self.client_cert_path.is_empty() && !self.client_key_path.is_empty() {
                tls = tls.with_client_cert(
                    PathBuf::from(&self.client_cert_path),
                    PathBuf::from(&self.client_key_path),
                );
            }

            tls = tls.with_verify_cert(self.verify_cert);

            if !self.server_name.is_empty() {
                tls = tls.with_server_name(self.server_name.clone());
            }

            config.tls = Some(tls);
        } else {
            config.tls = None;
        }

        config
    }
}

impl OmniTakApp {
    /// Creates a new OmniTAK GUI application.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous state if available
        let state = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        // Initialize backend service
        let backend = match BackendService::new() {
            Ok(service) => {
                tracing::info!("Backend service initialized");
                Some(service)
            }
            Err(e) => {
                tracing::error!("Failed to initialize backend service: {}", e);
                None
            }
        };

        Self {
            state: Arc::new(Mutex::new(state)),
            ui_state: UiState {
                auto_scroll: true,
                ..Default::default()
            },
            backend,
            status_message: None,
            status_message_expiry: None,
        }
    }

    /// Shows a status message to the user
    pub fn show_status(&mut self, message: String, level: StatusLevel, duration_secs: u64) {
        self.status_message = Some((message, level));
        self.status_message_expiry =
            Some(std::time::Instant::now() + Duration::from_secs(duration_secs));
    }

    /// Clears the status message if expired
    fn check_status_expiry(&mut self) {
        if let Some(expiry) = self.status_message_expiry {
            if std::time::Instant::now() > expiry {
                self.status_message = None;
                self.status_message_expiry = None;
            }
        }
    }

    /// Processes backend events
    fn process_backend_events(&mut self) {
        // Collect all events first to avoid borrowing conflicts
        let events: Vec<BackendEvent> = if let Some(backend) = &self.backend {
            let mut collected = Vec::new();
            while let Some(event) = backend.try_recv_event() {
                collected.push(event);
            }
            collected
        } else {
            Vec::new()
        };

        // Now process events without holding a reference to backend
        for event in events {
            match event {
                BackendEvent::StatusUpdate(server_name, metadata) => {
                    self.update_connection_metadata(server_name, metadata);
                }
                BackendEvent::MessageReceived(log) => {
                    self.add_message_log(log);
                }
                BackendEvent::Error(server_name, error) => {
                    tracing::error!("Backend error for {}: {}", server_name, error);
                    self.show_status(
                        format!("Error for {}: {}", server_name, error),
                        StatusLevel::Error,
                        10,
                    );
                }
                BackendEvent::MetricsUpdate(metrics) => {
                    let mut state = self.state.lock().unwrap();
                    state.metrics = metrics;
                }
            }
        }
    }

    /// Connects to a server
    pub fn connect_server(&mut self, config: ServerConfig) {
        if let Some(backend) = &self.backend {
            if let Err(e) = backend.send_command(BackendCommand::Connect(config.clone())) {
                tracing::error!("Failed to send connect command: {}", e);
                self.show_status(
                    format!("Failed to connect to {}", config.name),
                    StatusLevel::Error,
                    5,
                );
            } else {
                self.show_status(
                    format!("Connecting to {}...", config.name),
                    StatusLevel::Info,
                    3,
                );
            }
        }
    }

    /// Disconnects from a server
    pub fn disconnect_server(&mut self, server_name: String) {
        if let Some(backend) = &self.backend {
            if let Err(e) = backend.send_command(BackendCommand::Disconnect(server_name.clone())) {
                tracing::error!("Failed to send disconnect command: {}", e);
                self.show_status(
                    format!("Failed to disconnect from {}", server_name),
                    StatusLevel::Error,
                    5,
                );
            } else {
                self.show_status(
                    format!("Disconnecting from {}...", server_name),
                    StatusLevel::Info,
                    3,
                );
            }
        }
    }

    /// Adds a new server configuration.
    pub fn add_server(&mut self, config: ServerConfig) {
        let mut state = self.state.lock().unwrap();
        state.servers.push(config);
    }

    /// Updates an existing server configuration.
    pub fn update_server(&mut self, index: usize, config: ServerConfig) {
        let mut state = self.state.lock().unwrap();
        if index < state.servers.len() {
            state.servers[index] = config;
        }
    }

    /// Removes a server configuration.
    pub fn remove_server(&mut self, index: usize) {
        let mut state = self.state.lock().unwrap();
        if index < state.servers.len() {
            state.servers.remove(index);
        }
    }

    /// Updates connection metadata.
    pub fn update_connection_metadata(
        &mut self,
        server_name: String,
        metadata: ConnectionMetadata,
    ) {
        let mut state = self.state.lock().unwrap();
        state.connections.insert(server_name, metadata);

        // Update metrics
        state.metrics.active_connections = state
            .connections
            .values()
            .filter(|m| m.status == ServerStatus::Connected)
            .count();
        state.metrics.failed_connections = state
            .connections
            .values()
            .filter(|m| m.status == ServerStatus::Failed)
            .count();

        state.metrics.total_messages_received = state
            .connections
            .values()
            .map(|m| m.messages_received)
            .sum();
        state.metrics.total_messages_sent =
            state.connections.values().map(|m| m.messages_sent).sum();
        state.metrics.total_bytes_received =
            state.connections.values().map(|m| m.bytes_received).sum();
        state.metrics.total_bytes_sent = state.connections.values().map(|m| m.bytes_sent).sum();
    }

    /// Adds a message to the log.
    pub fn add_message_log(&mut self, log: MessageLog) {
        let mut state = self.state.lock().unwrap();
        state.message_log.push(log);

        // Keep only last 1000 messages
        let len = state.message_log.len();
        if len > 1000 {
            state.message_log.drain(0..len - 1000);
        }
    }

    /// Exports configuration to a file
    pub fn export_config(&self, path: &str) -> anyhow::Result<()> {
        let state = self.state.lock().unwrap();
        let config = ConfigFile::new(state.servers.clone());

        // Validate before exporting
        if let Err(errors) = config.validate() {
            return Err(anyhow::anyhow!(
                "Configuration validation failed: {:?}",
                errors
            ));
        }

        export_config(&config, path)?;
        Ok(())
    }

    /// Imports configuration from a file
    pub fn import_config(&mut self, path: &str) -> anyhow::Result<usize> {
        let config = import_config(path)?;

        // Validate before importing
        if let Err(errors) = config.validate() {
            return Err(anyhow::anyhow!(
                "Configuration validation failed: {:?}",
                errors
            ));
        }

        let mut state = self.state.lock().unwrap();
        let imported_count = config.servers.len();
        state.servers.extend(config.servers);

        Ok(imported_count)
    }

    /// Replaces all server configurations with imported ones
    pub fn import_config_replace(&mut self, path: &str) -> anyhow::Result<usize> {
        let config = import_config(path)?;

        // Validate before importing
        if let Err(errors) = config.validate() {
            return Err(anyhow::anyhow!(
                "Configuration validation failed: {:?}",
                errors
            ));
        }

        let mut state = self.state.lock().unwrap();
        let imported_count = config.servers.len();
        state.servers = config.servers;

        Ok(imported_count)
    }
}

impl eframe::App for OmniTakApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process backend events
        self.process_backend_events();

        // Check status message expiry
        self.check_status_expiry();

        // Top panel with tabs
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.heading("🎯 OmniTAK");

                ui.separator();

                if ui
                    .selectable_label(self.ui_state.selected_tab == Tab::Dashboard, "📊 Dashboard")
                    .clicked()
                {
                    self.ui_state.selected_tab = Tab::Dashboard;
                }

                if ui
                    .selectable_label(
                        self.ui_state.selected_tab == Tab::Connections,
                        "🔌 Connections",
                    )
                    .clicked()
                {
                    self.ui_state.selected_tab = Tab::Connections;
                }

                if ui
                    .selectable_label(self.ui_state.selected_tab == Tab::Messages, "💬 Messages")
                    .clicked()
                {
                    self.ui_state.selected_tab = Tab::Messages;
                }

                if ui
                    .selectable_label(self.ui_state.selected_tab == Tab::Settings, "⚙ Settings")
                    .clicked()
                {
                    self.ui_state.selected_tab = Tab::Settings;
                }
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| match self.ui_state.selected_tab {
            Tab::Dashboard => ui::dashboard::show(ui, &self.state),
            Tab::Connections => ui::connections::show(ui, self),
            Tab::Messages => ui::messages::show(ui, &self.state, &mut self.ui_state),
            Tab::Settings => ui::settings::show(ui, self),
        });

        // Bottom status bar
        if let Some((message, level)) = &self.status_message {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (icon, color) = match level {
                        StatusLevel::Info => ("ℹ", egui::Color32::LIGHT_BLUE),
                        StatusLevel::Success => ("✓", egui::Color32::GREEN),
                        StatusLevel::Warning => ("⚠", egui::Color32::YELLOW),
                        StatusLevel::Error => ("✗", egui::Color32::RED),
                    };
                    ui.colored_label(color, icon);
                    ui.label(message);
                });
            });
        }

        // Server dialog (modal)
        if self.ui_state.server_dialog.is_some() {
            ui::server_dialog::show(ctx, self);
        }

        // Request repaint for real-time updates
        ctx.request_repaint_after(Duration::from_secs(1));
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let state = self.state.lock().unwrap();
        eframe::set_value(storage, eframe::APP_KEY, &*state);
    }
}

/// Formats bytes into a human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}

/// Formats a duration into a human-readable string.
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();

    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}
