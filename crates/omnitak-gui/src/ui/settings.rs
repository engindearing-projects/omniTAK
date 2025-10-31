//! Settings view for application configuration.

use crate::AppState;
use eframe::egui;
use std::sync::{Arc, Mutex};

/// Shows the settings view.
pub fn show(ui: &mut egui::Ui, state: &Arc<Mutex<AppState>>) {
    ui.heading("Settings");
    ui.add_space(10.0);

    ui.label("Application configuration and preferences");
    ui.add_space(20.0);

    // About section
    egui::Frame::none()
        .fill(egui::Color32::from_gray(35))
        .rounding(5.0)
        .inner_margin(15.0)
        .show(ui, |ui| {
            ui.heading("About OmniTAK");
            ui.add_space(5.0);
            ui.label("Version: 0.2.0");
            ui.label("License: MIT OR Apache-2.0");
            ui.add_space(5.0);
            ui.label("OmniTAK is a military-grade TAK server aggregator that connects to");
            ui.label("multiple TAK servers simultaneously and intelligently routes CoT messages.");
        });

    ui.add_space(20.0);

    // Configuration info
    egui::Frame::none()
        .fill(egui::Color32::from_gray(35))
        .rounding(5.0)
        .inner_margin(15.0)
        .show(ui, |ui| {
            ui.heading("Configuration");
            ui.add_space(5.0);

            let state = state.lock().unwrap();

            ui.label(format!("Total Servers: {}", state.servers.len()));
            ui.label(format!("Active Connections: {}", state.metrics.active_connections));
            ui.label(format!("Message Log Size: {} entries", state.message_log.len()));
        });

    ui.add_space(20.0);

    // Future settings
    ui.label(egui::RichText::new("Additional settings will be added in future releases").color(egui::Color32::GRAY));
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.label("• Auto-start connections on launch");
        ui.label(egui::RichText::new("(Coming soon)").color(egui::Color32::YELLOW));
    });

    ui.horizontal(|ui| {
        ui.label("• Message retention policy");
        ui.label(egui::RichText::new("(Coming soon)").color(egui::Color32::YELLOW));
    });

    ui.horizontal(|ui| {
        ui.label("• Export configuration");
        ui.label(egui::RichText::new("(Coming soon)").color(egui::Color32::YELLOW));
    });

    ui.horizontal(|ui| {
        ui.label("• Import configuration from YAML");
        ui.label(egui::RichText::new("(Coming soon)").color(egui::Color32::YELLOW));
    });
}
