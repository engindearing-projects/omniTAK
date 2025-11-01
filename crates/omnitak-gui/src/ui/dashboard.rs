//! Dashboard view showing system overview and metrics.

use crate::{format_bytes, format_duration, AppState};
use eframe::egui;
use omnitak_core::types::ServerStatus;
use std::sync::{Arc, Mutex};

/// Shows the dashboard view.
pub fn show(ui: &mut egui::Ui, state: &Arc<Mutex<AppState>>) {
    let state = state.lock().unwrap();

    ui.heading("System Dashboard");
    ui.add_space(10.0);

    // Metrics cards
    ui.horizontal(|ui| {
        metric_card(
            ui,
            "Active Connections",
            &state.metrics.active_connections.to_string(),
            egui::Color32::GREEN,
        );
        metric_card(
            ui,
            "Failed Connections",
            &state.metrics.failed_connections.to_string(),
            egui::Color32::RED,
        );
        metric_card(
            ui,
            "Total Servers",
            &state.servers.len().to_string(),
            egui::Color32::BLUE,
        );
    });

    ui.add_space(10.0);

    ui.horizontal(|ui| {
        metric_card(
            ui,
            "Messages Received",
            &state.metrics.total_messages_received.to_string(),
            egui::Color32::LIGHT_BLUE,
        );
        metric_card(
            ui,
            "Messages Sent",
            &state.metrics.total_messages_sent.to_string(),
            egui::Color32::LIGHT_GREEN,
        );
    });

    ui.add_space(10.0);

    ui.horizontal(|ui| {
        metric_card(
            ui,
            "Bytes Received",
            &format_bytes(state.metrics.total_bytes_received),
            egui::Color32::from_rgb(100, 150, 255),
        );
        metric_card(
            ui,
            "Bytes Sent",
            &format_bytes(state.metrics.total_bytes_sent),
            egui::Color32::from_rgb(100, 255, 150),
        );
    });

    ui.add_space(20.0);

    // Connection status overview
    ui.heading("Connection Status");
    ui.add_space(10.0);

    egui::Grid::new("connection_status_grid")
        .num_columns(4)
        .spacing([20.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Server").strong());
            ui.label(egui::RichText::new("Status").strong());
            ui.label(egui::RichText::new("Messages").strong());
            ui.label(egui::RichText::new("Uptime").strong());
            ui.end_row();

            for server in &state.servers {
                if let Some(metadata) = state.connections.get(&server.name) {
                    ui.label(&server.name);

                    // Status with color
                    let (status_text, status_color) = match metadata.status {
                        ServerStatus::Connected => ("Connected", egui::Color32::GREEN),
                        ServerStatus::Disconnected => ("Disconnected", egui::Color32::GRAY),
                        ServerStatus::Reconnecting => ("Reconnecting", egui::Color32::YELLOW),
                        ServerStatus::Failed => ("Failed", egui::Color32::RED),
                    };
                    ui.colored_label(status_color, status_text);

                    ui.label(format!(
                        "{} ↓ / {} ↑",
                        metadata.messages_received, metadata.messages_sent
                    ));

                    if let Some(uptime) = metadata.uptime() {
                        ui.label(format_duration(uptime));
                    } else {
                        ui.label("-");
                    }

                    ui.end_row();
                } else {
                    ui.label(&server.name);
                    ui.colored_label(egui::Color32::GRAY, "No connection");
                    ui.label("-");
                    ui.label("-");
                    ui.end_row();
                }
            }
        });

    if state.servers.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(
                egui::RichText::new("No servers configured")
                    .size(16.0)
                    .color(egui::Color32::GRAY),
            );
            ui.label("Go to the Connections tab to add servers");
        });
    }
}

/// Shows a metric card.
fn metric_card(ui: &mut egui::Ui, title: &str, value: &str, color: egui::Color32) {
    egui::Frame::none()
        .fill(egui::Color32::from_gray(40))
        .rounding(5.0)
        .inner_margin(15.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(title)
                        .size(12.0)
                        .color(egui::Color32::GRAY),
                );
                ui.label(egui::RichText::new(value).size(24.0).strong().color(color));
            });
        });
}
