//! Connections view for managing server connections.

use crate::{format_bytes, OmniTakApp, ServerDialogState};
use eframe::egui;
use omnitak_core::types::ServerStatus;

/// Shows the connections view.
pub fn show(ui: &mut egui::Ui, app: &mut OmniTakApp) {
    ui.heading("Server Connections");
    ui.add_space(10.0);

    // Add server button
    if ui.button("➕ Add Server").clicked() {
        app.ui_state.server_dialog = Some(ServerDialogState::new());
    }

    ui.add_space(10.0);

    // Server list
    let state = app.state.lock().unwrap();
    let servers_clone = state.servers.clone();
    let connections_clone = state.connections.clone();
    drop(state);

    if servers_clone.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(
                egui::RichText::new("No servers configured")
                    .size(16.0)
                    .color(egui::Color32::GRAY),
            );
            ui.label("Click 'Add Server' to configure your first TAK server connection");
        });
        return;
    }

    let mut server_to_remove: Option<usize> = None;
    let mut server_to_edit: Option<usize> = None;
    let mut server_to_connect: Option<usize> = None;
    let mut server_to_disconnect: Option<String> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (idx, server) in servers_clone.iter().enumerate() {
            egui::Frame::none()
                .fill(egui::Color32::from_gray(35))
                .rounding(5.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            // Server name and host
                            ui.label(egui::RichText::new(&server.name).size(16.0).strong());
                            ui.label(format!(
                                "{}:{} ({})",
                                server.host, server.port, server.protocol
                            ));

                            // Connection status
                            if let Some(metadata) = connections_clone.get(&server.name) {
                                ui.horizontal(|ui| {
                                    let (status_text, status_color) = match metadata.status {
                                        ServerStatus::Connected => {
                                            ("● Connected", egui::Color32::GREEN)
                                        }
                                        ServerStatus::Disconnected => {
                                            ("● Disconnected", egui::Color32::GRAY)
                                        }
                                        ServerStatus::Reconnecting => {
                                            ("● Reconnecting", egui::Color32::YELLOW)
                                        }
                                        ServerStatus::Failed => ("● Failed", egui::Color32::RED),
                                    };
                                    ui.colored_label(status_color, status_text);

                                    if let Some(error) = &metadata.last_error {
                                        ui.label(
                                            egui::RichText::new(format!("({})", error))
                                                .color(egui::Color32::LIGHT_RED),
                                        );
                                    }
                                });

                                // Metrics
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "↓ {} msgs ({}) | ↑ {} msgs ({})",
                                        metadata.messages_received,
                                        format_bytes(metadata.bytes_received),
                                        metadata.messages_sent,
                                        format_bytes(metadata.bytes_sent)
                                    ));
                                });

                                if metadata.reconnect_attempts > 0 {
                                    ui.label(format!(
                                        "Reconnect attempts: {}",
                                        metadata.reconnect_attempts
                                    ));
                                }
                            } else {
                                ui.colored_label(egui::Color32::GRAY, "● Not connected");
                            }

                            // TLS indicator
                            if server.tls.is_some() {
                                ui.label(
                                    egui::RichText::new("🔒 TLS Enabled")
                                        .color(egui::Color32::LIGHT_GREEN),
                                );
                            }

                            // Tags
                            if !server.tags.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label("Tags:");
                                    for tag in &server.tags {
                                        ui.label(
                                            egui::RichText::new(tag)
                                                .background_color(egui::Color32::from_gray(60)),
                                        );
                                    }
                                });
                            }
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑 Delete").clicked() {
                                server_to_remove = Some(idx);
                            }

                            if ui.button("✏ Edit").clicked() {
                                server_to_edit = Some(idx);
                            }

                            // Connect/Disconnect button
                            if let Some(metadata) = connections_clone.get(&server.name) {
                                if metadata.status == ServerStatus::Connected {
                                    if ui.button("⏸ Disconnect").clicked() {
                                        server_to_disconnect = Some(server.name.clone());
                                    }
                                } else if metadata.status != ServerStatus::Reconnecting {
                                    if ui.button("▶ Connect").clicked() {
                                        server_to_connect = Some(idx);
                                    }
                                }
                            } else if server.enabled {
                                if ui.button("▶ Connect").clicked() {
                                    server_to_connect = Some(idx);
                                }
                            }

                            // Enable/disable toggle
                            let enabled_text = if server.enabled {
                                "Enabled"
                            } else {
                                "Disabled"
                            };
                            let enabled_color = if server.enabled {
                                egui::Color32::GREEN
                            } else {
                                egui::Color32::GRAY
                            };
                            ui.colored_label(enabled_color, enabled_text);
                        });
                    });
                });

            ui.add_space(10.0);
        }
    });

    // Handle actions
    if let Some(idx) = server_to_remove {
        // Disconnect before removing
        let server_name = {
            let state = app.state.lock().unwrap();
            state.servers.get(idx).map(|s| s.name.clone())
        };
        if let Some(name) = server_name {
            app.disconnect_server(name);
        }
        app.remove_server(idx);
    }

    if let Some(idx) = server_to_edit {
        let server_opt = {
            let state = app.state.lock().unwrap();
            state.servers.get(idx).cloned()
        };
        if let Some(server) = server_opt {
            app.ui_state.server_dialog = Some(ServerDialogState::edit(idx, server));
        }
    }

    if let Some(idx) = server_to_connect {
        let server_opt = {
            let state = app.state.lock().unwrap();
            state.servers.get(idx).cloned()
        };
        if let Some(server) = server_opt {
            app.connect_server(server);
        }
    }

    if let Some(server_name) = server_to_disconnect {
        app.disconnect_server(server_name);
    }
}
