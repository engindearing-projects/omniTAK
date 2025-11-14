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
            ui.label(format!(
                "Active Connections: {}",
                state.metrics.active_connections
            ));
            ui.label(format!(
                "Message Log Size: {} entries",
                state.message_log.len()
            ));
        });

    ui.add_space(20.0);

    // Future settings
    ui.label(
        egui::RichText::new("Additional settings will be added in future releases")
            .color(egui::Color32::GRAY),
    );
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
                // Handle ongoing export promise
                if let Some(promise) = &app.ui_state.export_promise {
                    if let Some(result) = promise.ready() {
                        if let Some(path) = result {
                            let state = app.state.lock().unwrap();
                            let config = crate::ConfigFile::new(state.servers.clone());
                            drop(state); // Release lock before I/O

                            match crate::export_config(&config, path.as_path()) {
                                Ok(()) => {
                                    app.show_status(
                                        format!("Configuration exported to {}", path.display()),
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
                        app.ui_state.export_promise = None;
                    }
                }

                if ui.button("📤 Export Configuration").clicked() && app.ui_state.export_promise.is_none() {
                    let promise = poll_promise::Promise::spawn_thread("export_dialog", || {
                        rfd::FileDialog::new()
                            .add_filter("YAML", &["yaml", "yml"])
                            .add_filter("JSON", &["json"])
                            .set_file_name("omnitak_config.yaml")
                            .save_file()
                    });
                    app.ui_state.export_promise = Some(promise);
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                // Handle ongoing import promise
                if let Some(promise) = &app.ui_state.import_promise {
                    if let Some(result) = promise.ready() {
                        if let Some(path) = result {
                            match crate::import_config(&path) {
                                Ok(config) => {
                                    let mut state = app.state.lock().unwrap();
                                    let count = config.servers.len();
                                    state.servers.extend(config.servers);
                                    drop(state); // Release lock

                                    app.show_status(
                                        format!("Imported {} server(s) from {}", count, path.display()),
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
                        app.ui_state.import_promise = None;
                    }
                }

                if ui.button("📥 Import Configuration").clicked() && app.ui_state.import_promise.is_none() {
                    let promise = poll_promise::Promise::spawn_thread("import_dialog", || {
                        rfd::FileDialog::new()
                            .add_filter("YAML", &["yaml", "yml"])
                            .add_filter("JSON", &["json"])
                            .pick_file()
                    });
                    app.ui_state.import_promise = Some(promise);
                }
            });

            ui.add_space(5.0);
            ui.label(egui::RichText::new("⚠ Importing will add to existing servers").color(egui::Color32::YELLOW));
        });

    ui.add_space(20.0);

    // Application Settings
    egui::Frame::none()
        .fill(egui::Color32::from_gray(35))
        .rounding(5.0)
        .inner_margin(15.0)
        .show(ui, |ui| {
            ui.heading("Application Settings");
            ui.add_space(10.0);

            let mut settings_changed = false;

            // Auto-start connections
            ui.horizontal(|ui| {
                let mut auto_start = {
                    let state = app.state.lock().unwrap();
                    state.settings.auto_start_connections
                };

                if ui.checkbox(&mut auto_start, "Auto-start connections on launch").changed() {
                    let mut state = app.state.lock().unwrap();
                    state.settings.auto_start_connections = auto_start;
                    settings_changed = true;
                }

                ui.label(egui::RichText::new("ℹ").color(egui::Color32::LIGHT_BLUE))
                    .on_hover_text("Automatically connect to all enabled servers when the application starts");
            });

            ui.add_space(10.0);

            // Message retention policy
            ui.horizontal(|ui| {
                ui.label("Message log retention:");

                let mut max_messages = {
                    let state = app.state.lock().unwrap();
                    state.settings.max_message_log_size
                };

                let mut temp_value = max_messages.to_string();
                if ui.add(egui::TextEdit::singleline(&mut temp_value).desired_width(100.0)).changed() {
                    if let Ok(value) = temp_value.parse::<usize>() {
                        if value >= 100 && value <= 100000 {
                            let mut state = app.state.lock().unwrap();
                            state.settings.max_message_log_size = value;
                            settings_changed = true;
                        }
                    }
                }

                ui.label("messages");
                ui.label(egui::RichText::new("ℹ").color(egui::Color32::LIGHT_BLUE))
                    .on_hover_text("Maximum number of messages to keep in the log (100-100000)");
            });

            if settings_changed {
                app.show_status(
                    "Settings updated".to_string(),
                    crate::StatusLevel::Success,
                    3,
                );
            }
        });

    ui.add_space(20.0);
}
