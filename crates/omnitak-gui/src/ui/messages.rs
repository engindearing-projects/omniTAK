//! Messages view for viewing CoT message logs.

use crate::{AppState, UiState};
use eframe::egui;
use std::sync::{Arc, Mutex};

/// Shows the messages view.
pub fn show(ui: &mut egui::Ui, state: &Arc<Mutex<AppState>>, ui_state: &mut UiState) {
    ui.heading("Message Log");
    ui.add_space(10.0);

    // Controls
    ui.horizontal(|ui| {
        ui.label("Filter:");
        ui.text_edit_singleline(&mut ui_state.message_filter);

        ui.checkbox(&mut ui_state.auto_scroll, "Auto-scroll");

        if ui.button("🗑 Clear Log").clicked() {
            let mut state = state.lock().unwrap();
            state.message_log.clear();
        }
    });

    ui.add_space(10.0);

    // Message log
    let state = state.lock().unwrap();

    if state.message_log.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(egui::RichText::new("No messages yet").size(16.0).color(egui::Color32::GRAY));
            ui.label("Messages will appear here when connections are active");
        });
        return;
    }

    let filtered_messages: Vec<_> = state.message_log.iter()
        .filter(|msg| {
            if ui_state.message_filter.is_empty() {
                true
            } else {
                msg.server.to_lowercase().contains(&ui_state.message_filter.to_lowercase()) ||
                msg.content.to_lowercase().contains(&ui_state.message_filter.to_lowercase()) ||
                msg.msg_type.to_lowercase().contains(&ui_state.message_filter.to_lowercase())
            }
        })
        .collect();

    ui.label(format!("Showing {} of {} messages", filtered_messages.len(), state.message_log.len()));

    ui.add_space(5.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("message_log_grid")
                .num_columns(4)
                .spacing([10.0, 5.0])
                .striped(true)
                .show(ui, |ui| {
                    // Header
                    ui.label(egui::RichText::new("Timestamp").strong());
                    ui.label(egui::RichText::new("Server").strong());
                    ui.label(egui::RichText::new("Type").strong());
                    ui.label(egui::RichText::new("Content").strong());
                    ui.end_row();

                    // Messages
                    for msg in filtered_messages.iter().rev() {
                        ui.label(msg.timestamp.format("%H:%M:%S").to_string());
                        ui.label(&msg.server);
                        ui.label(
                            egui::RichText::new(&msg.msg_type)
                                .background_color(egui::Color32::from_gray(60))
                        );
                        ui.label(&msg.content);
                        ui.end_row();
                    }
                });

            // Auto-scroll to bottom
            if ui_state.auto_scroll {
                ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
            }
        });
}
