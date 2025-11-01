//! Server add/edit dialog.

use crate::{OmniTakApp, ServerDialogState};
use eframe::egui;
use omnitak_core::types::Protocol;

/// Shows the server add/edit dialog.
pub fn show(ctx: &egui::Context, app: &mut OmniTakApp) {
    // Take ownership of dialog state temporarily to avoid borrow conflicts
    let Some(mut dialog_state) = app.ui_state.server_dialog.take() else {
        return;
    };
    let mut save_clicked = false;
    let mut close_dialog = false;

    let response = egui::Window::new(if dialog_state.editing_index.is_some() {
        "Edit Server"
    } else {
        "Add Server"
    })
    .resizable(false)
    .collapsible(false)
    .show(ctx, |ui| {
        egui::Grid::new("server_dialog_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                // Server name
                ui.label("Name:");
                ui.text_edit_singleline(&mut dialog_state.config.name);
                ui.end_row();

                // Host
                ui.label("Host:");
                ui.text_edit_singleline(&mut dialog_state.config.host);
                ui.end_row();

                // Port
                ui.label("Port:");
                let mut port_str = dialog_state.config.port.to_string();
                if ui.text_edit_singleline(&mut port_str).changed() {
                    if let Ok(port) = port_str.parse::<u16>() {
                        dialog_state.config.port = port;
                    }
                }
                ui.end_row();

                // Protocol
                ui.label("Protocol:");
                egui::ComboBox::from_id_source("protocol_combo")
                    .selected_text(format!("{}", dialog_state.config.protocol))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut dialog_state.config.protocol,
                            Protocol::Tcp,
                            "TCP",
                        );
                        ui.selectable_value(
                            &mut dialog_state.config.protocol,
                            Protocol::Udp,
                            "UDP",
                        );
                        ui.selectable_value(
                            &mut dialog_state.config.protocol,
                            Protocol::Tls,
                            "TLS",
                        );
                        ui.selectable_value(
                            &mut dialog_state.config.protocol,
                            Protocol::WebSocket,
                            "WebSocket",
                        );
                    });
                ui.end_row();

                // Enabled
                ui.label("Enabled:");
                ui.checkbox(&mut dialog_state.config.enabled, "");
                ui.end_row();
            });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // TLS Configuration
        ui.checkbox(&mut dialog_state.tls_enabled, "Enable TLS");

        if dialog_state.tls_enabled {
            ui.add_space(5.0);

            egui::Grid::new("tls_config_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    // CA Certificate
                    ui.label("CA Certificate:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut dialog_state.ca_cert_path);
                        if ui.button("Browse").clicked() {
                            // File picker would go here
                            // For now, just show a placeholder
                        }
                    });
                    ui.end_row();

                    // Client Certificate
                    ui.label("Client Certificate:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut dialog_state.client_cert_path);
                        if ui.button("Browse").clicked() {
                            // File picker would go here
                        }
                    });
                    ui.end_row();

                    // Client Key
                    ui.label("Client Key:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut dialog_state.client_key_path);
                        if ui.button("Browse").clicked() {
                            // File picker would go here
                        }
                    });
                    ui.end_row();

                    // Verify cert
                    ui.label("Verify Certificate:");
                    ui.checkbox(&mut dialog_state.verify_cert, "");
                    ui.end_row();

                    // Server name (SNI)
                    ui.label("Server Name (SNI):");
                    ui.text_edit_singleline(&mut dialog_state.server_name);
                    ui.end_row();
                });
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Buttons
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                save_clicked = true;
            }

            if ui.button("Cancel").clicked() {
                close_dialog = true;
            }
        });

        // Validation message
        if let Err(err) = dialog_state.build().validate() {
            ui.add_space(5.0);
            ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
        }
    });

    // Check if window was closed via X button
    if response.is_none() {
        close_dialog = true;
    }

    // Handle save
    if save_clicked {
        let config = dialog_state.build();
        if config.validate().is_ok() {
            if let Some(idx) = dialog_state.editing_index {
                app.update_server(idx, config);
            } else {
                app.add_server(config);
            }
            // Dialog closes after save
            // Don't restore dialog_state, leave it as None
            return;
        }
    }

    // Handle dialog close
    if close_dialog {
        // Don't restore dialog_state, leave it as None
        return;
    }

    // Dialog is still open, restore the state
    app.ui_state.server_dialog = Some(dialog_state);
}
