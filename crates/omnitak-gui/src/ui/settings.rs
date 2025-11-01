//! Settings view for application configuration.

use crate::{AppState, OmniTakApp};
use eframe::egui;
use std::sync::{Arc, Mutex};

/// Shows the settings view.
pub fn show(ui: &mut egui::Ui, app: &mut OmniTakApp) {
    let state = &app.state;
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

    // Import/Export Configuration
    egui::Frame::none()
        .fill(egui::Color32::from_gray(35))
        .rounding(5.0)
        .inner_margin(15.0)
        .show(ui, |ui| {
            ui.heading("Import/Export Configuration");
            ui.add_space(5.0);

            ui.label("Backup or restore your server configurations");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("📤 Export to YAML").clicked() {
                    match app.export_config("omnitak_config.yaml") {
                        Ok(()) => {
                            app.show_status(
                                "Configuration exported to omnitak_config.yaml".to_string(),
                                crate::StatusLevel::Success,
                                5,
                            );
                        }
                        Err(e) => {
                            app.show_status(
                                format!("Export failed: {}", e),
                                crate::StatusLevel::Error,
                                10,
                            );
                        }
                    }
                }

                if ui.button("📤 Export to JSON").clicked() {
                    match app.export_config("omnitak_config.json") {
                        Ok(()) => {
                            app.show_status(
                                "Configuration exported to omnitak_config.json".to_string(),
                                crate::StatusLevel::Success,
                                5,
                            );
                        }
                        Err(e) => {
                            app.show_status(
                                format!("Export failed: {}", e),
                                crate::StatusLevel::Error,
                                10,
                            );
                        }
                    }
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("📥 Import from File").clicked() {
                    // TODO: Add file picker dialog
                    app.show_status(
                        "File picker not yet implemented. Place config file as 'import_config.yaml' in current directory".to_string(),
                        crate::StatusLevel::Info,
                        5,
                    );
                }

                if ui.button("🔄 Import from import_config.yaml").clicked() {
                    match app.import_config("import_config.yaml") {
                        Ok(count) => {
                            app.show_status(
                                format!("Imported {} server(s) from import_config.yaml", count),
                                crate::StatusLevel::Success,
                                5,
                            );
                        }
                        Err(e) => {
                            app.show_status(
                                format!("Import failed: {}", e),
                                crate::StatusLevel::Error,
                                10,
                            );
                        }
                    }
                }
            });

            ui.add_space(5.0);
            ui.label(egui::RichText::new("⚠ Importing will add to existing servers").color(egui::Color32::YELLOW));
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
        ui.label("• File picker for import/export");
        ui.label(egui::RichText::new("(Coming soon)").color(egui::Color32::YELLOW));
    });
}
