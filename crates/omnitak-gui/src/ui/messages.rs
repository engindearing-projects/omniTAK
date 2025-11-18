//! Enhanced messages view with color-coded affiliations, filtering, and statistics.

use crate::{AffiliationFilter, AppState, MessageLog, UiState};
use eframe::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Affiliation color constants based on MIL-STD-2525D
pub struct AffiliationColors;

impl AffiliationColors {
    pub const PENDING: egui::Color32 = egui::Color32::GRAY;
    pub const UNKNOWN: egui::Color32 = egui::Color32::YELLOW;
    pub const ASSUMED_FRIEND: egui::Color32 = egui::Color32::from_rgb(255, 165, 0); // Orange/Amber
    pub const FRIEND: egui::Color32 = egui::Color32::from_rgb(0, 191, 255); // Cyan/Blue
    pub const NEUTRAL: egui::Color32 = egui::Color32::GREEN;
    pub const SUSPECT: egui::Color32 = egui::Color32::from_rgb(255, 140, 0); // Dark Orange
    pub const HOSTILE: egui::Color32 = egui::Color32::RED;
}

/// Get color and icon for an affiliation string
fn get_affiliation_display(
    affiliation: Option<&str>,
) -> (egui::Color32, &'static str, &'static str) {
    match affiliation {
        Some("Pending") | Some("pending") => (AffiliationColors::PENDING, "⚫", "Pending"),
        Some("Unknown") | Some("unknown") => (AffiliationColors::UNKNOWN, "🟡", "Unknown"),
        Some("Assumed Friend") | Some("assumed_friend") => {
            (AffiliationColors::ASSUMED_FRIEND, "🟠", "Assumed Friend")
        }
        Some("Friend") | Some("friend") => (AffiliationColors::FRIEND, "🔵", "Friend"),
        Some("Neutral") | Some("neutral") => (AffiliationColors::NEUTRAL, "🟢", "Neutral"),
        Some("Suspect") | Some("suspect") => (AffiliationColors::SUSPECT, "🟠", "Suspect"),
        Some("Hostile") | Some("hostile") => (AffiliationColors::HOSTILE, "🔴", "Hostile"),
        _ => (egui::Color32::GRAY, "⚪", "Unknown"),
    }
}

/// Shows the enhanced messages view.
pub fn show(ui: &mut egui::Ui, state: &Arc<Mutex<AppState>>, ui_state: &mut UiState) {
    ui.heading("Message Viewer");
    ui.add_space(10.0);

    let state = state.lock().unwrap();

    // Use horizontal split: main view on left, statistics on right
    egui::SidePanel::right("message_stats_panel")
        .resizable(true)
        .default_width(250.0)
        .show_inside(ui, |ui| {
            show_statistics_panel(ui, &state.message_log, ui_state);
        });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        // Filter controls
        show_filter_controls(ui, ui_state, &state.message_log);

        ui.add_space(10.0);

        // Message display
        if state.message_log.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(
                    egui::RichText::new("No messages yet")
                        .size(16.0)
                        .color(egui::Color32::GRAY),
                );
                ui.label("Messages will appear here when connections are active");
            });
        } else {
            show_message_list(ui, &state.message_log, ui_state);
        }
    });

    // Show message details dialog if requested
    if ui_state.message_details_dialog.is_some() {
        show_message_details_dialog(ui.ctx(), ui_state);
    }
}

/// Shows filter controls
fn show_filter_controls(ui: &mut egui::Ui, ui_state: &mut UiState, messages: &[MessageLog]) {
    ui.horizontal(|ui| {
        // Text search filter
        ui.label("Search:");
        ui.text_edit_singleline(&mut ui_state.message_filter);

        ui.separator();

        // Affiliation filter dropdown
        ui.label("Affiliation:");
        egui::ComboBox::from_id_salt("affiliation_filter")
            .selected_text(format!("{:?}", ui_state.affiliation_filter))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut ui_state.affiliation_filter,
                    AffiliationFilter::All,
                    "All",
                );
                ui.selectable_value(
                    &mut ui_state.affiliation_filter,
                    AffiliationFilter::Friend,
                    "Friend",
                );
                ui.selectable_value(
                    &mut ui_state.affiliation_filter,
                    AffiliationFilter::Hostile,
                    "Hostile",
                );
                ui.selectable_value(
                    &mut ui_state.affiliation_filter,
                    AffiliationFilter::Neutral,
                    "Neutral",
                );
                ui.selectable_value(
                    &mut ui_state.affiliation_filter,
                    AffiliationFilter::Unknown,
                    "Unknown",
                );
                ui.selectable_value(
                    &mut ui_state.affiliation_filter,
                    AffiliationFilter::AssumedFriend,
                    "Assumed Friend",
                );
                ui.selectable_value(
                    &mut ui_state.affiliation_filter,
                    AffiliationFilter::Suspect,
                    "Suspect",
                );
                ui.selectable_value(
                    &mut ui_state.affiliation_filter,
                    AffiliationFilter::Pending,
                    "Pending",
                );
            });

        ui.separator();

        // Server filter
        ui.label("Server:");
        ui.text_edit_singleline(&mut ui_state.server_filter);
    });

    ui.horizontal(|ui| {
        ui.checkbox(&mut ui_state.auto_scroll, "Auto-scroll");

        if ui.button("Clear Filters").clicked() {
            ui_state.message_filter.clear();
            ui_state.server_filter.clear();
            ui_state.affiliation_filter = AffiliationFilter::All;
        }

        if ui.button("Export...").clicked() {
            // TODO: Implement export dialog
            tracing::info!("Export functionality not yet implemented");
        }
    });

    // Show filter summary
    let filtered_count = messages
        .iter()
        .filter(|msg| passes_filters(msg, ui_state))
        .count();
    ui.label(format!(
        "Showing {} of {} messages",
        filtered_count,
        messages.len()
    ));
}

/// Shows the message list with enhanced card display
fn show_message_list(ui: &mut egui::Ui, messages: &[MessageLog], ui_state: &mut UiState) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Display messages in reverse chronological order (newest first)
            for msg in messages.iter().rev() {
                if !passes_filters(msg, ui_state) {
                    continue;
                }

                show_message_card(ui, msg, ui_state);
            }

            // Auto-scroll to bottom
            if ui_state.auto_scroll {
                ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
            }
        });
}

/// Shows a single message in card format
fn show_message_card(ui: &mut egui::Ui, msg: &MessageLog, ui_state: &mut UiState) {
    let msg_id = msg
        .uid
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(&msg.msg_type);
    let is_expanded = ui_state.expanded_messages.contains(msg_id);

    egui::Frame::NONE
        .fill(ui.style().visuals.faint_bg_color)
        .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
        .corner_radius(4.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            // Header row
            ui.horizontal(|ui| {
                // Affiliation badge
                let (color, icon, label) = get_affiliation_display(msg.affiliation.as_deref());
                ui.label(egui::RichText::new(icon).color(color).size(16.0));
                ui.label(egui::RichText::new(label).color(color).strong());

                ui.separator();

                // Timestamp
                ui.label(egui::RichText::new(msg.timestamp.format("%H:%M:%S").to_string()).weak());

                ui.separator();

                // Callsign
                if let Some(callsign) = &msg.callsign {
                    ui.label(egui::RichText::new(callsign).strong());
                } else {
                    ui.label(egui::RichText::new(&msg.msg_type).weak());
                }

                // Expand/collapse button
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let button_text = if is_expanded { "▼" } else { "▶" };
                    if ui.button(button_text).clicked() {
                        if is_expanded {
                            ui_state.expanded_messages.remove(msg_id);
                        } else {
                            ui_state.expanded_messages.insert(msg_id.to_string());
                        }
                    }
                });
            });

            // Body (always visible in collapsed view)
            ui.horizontal(|ui| {
                ui.add_space(24.0); // Indent for alignment

                // Coordinates
                if let (Some(lat), Some(lon)) = (msg.lat, msg.lon) {
                    ui.label(format!("📍 {:.6}, {:.6}", lat, lon));

                    // Copy coordinates button
                    if ui
                        .small_button("📋")
                        .on_hover_text("Copy coordinates")
                        .clicked()
                    {
                        let coords = format!("{:.6}, {:.6}", lat, lon);
                        ui.ctx().copy_text(coords);
                    }
                }

                if let Some(alt) = msg.altitude {
                    ui.label(format!("⛰ {:.0}m", alt));
                }

                ui.label(format!("🖥 {}", msg.server));
            });

            // Expanded details
            if is_expanded {
                ui.add_space(5.0);
                ui.separator();

                ui.horizontal(|ui| {
                    ui.add_space(24.0);

                    ui.vertical(|ui| {
                        if let Some(uid) = &msg.uid {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("UID:").weak());
                                ui.label(uid);
                            });
                        }

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Type:").weak());
                            ui.label(&msg.msg_type);
                        });

                        if !msg.content.is_empty() {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Content:").weak());
                                ui.label(&msg.content);
                            });
                        }

                        // Action buttons
                        ui.horizontal(|ui| {
                            if ui.button("View Details").clicked() {
                                ui_state.message_details_dialog = Some(msg.clone());
                            }

                            if ui.button("Jump to Location").clicked() {
                                // TODO: Implement map view integration
                                tracing::info!("Jump to location: {:?}, {:?}", msg.lat, msg.lon);
                            }
                        });
                    });
                });
            }
        });

    ui.add_space(4.0);
}

/// Shows statistics panel
fn show_statistics_panel(ui: &mut egui::Ui, messages: &[MessageLog], ui_state: &UiState) {
    ui.heading("Statistics");
    ui.add_space(10.0);

    // Total messages
    ui.label(format!("Total Messages: {}", messages.len()));

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    // Messages per affiliation
    ui.label(egui::RichText::new("By Affiliation:").strong());
    ui.add_space(5.0);

    let mut affiliation_counts: HashMap<String, usize> = HashMap::new();
    for msg in messages {
        let affiliation = msg.affiliation.as_deref().unwrap_or("Unknown").to_string();
        *affiliation_counts.entry(affiliation).or_insert(0) += 1;
    }

    let total = messages.len() as f32;
    for (affiliation, count) in affiliation_counts.iter() {
        let (color, icon, label) = get_affiliation_display(Some(affiliation));
        let _percentage = (*count as f32 / total) * 100.0;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(icon).color(color));
            ui.label(label);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{}", count));
            });
        });

        // Progress bar
        let progress = *count as f32 / total;
        let progress_bar = egui::ProgressBar::new(progress)
            .fill(color)
            .show_percentage();
        ui.add(progress_bar);

        ui.add_space(3.0);
    }

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    // Messages per server
    ui.label(egui::RichText::new("By Server:").strong());
    ui.add_space(5.0);

    let mut server_counts: HashMap<String, usize> = HashMap::new();
    for msg in messages {
        *server_counts.entry(msg.server.clone()).or_insert(0) += 1;
    }

    for (server, count) in server_counts.iter() {
        ui.horizontal(|ui| {
            ui.label(server);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{}", count));
            });
        });
    }

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    // Message rate (approximate)
    if let (Some(first), Some(last)) = (messages.first(), messages.last()) {
        let duration = last.timestamp.signed_duration_since(first.timestamp);
        if duration.num_seconds() > 0 {
            let rate = messages.len() as f64 / duration.num_seconds() as f64;
            ui.label(format!("Average Rate: {:.2} msg/sec", rate));
        }
    }

    // Filtered count
    let filtered_count = messages
        .iter()
        .filter(|msg| passes_filters(msg, ui_state))
        .count();
    if filtered_count != messages.len() {
        ui.add_space(5.0);
        ui.label(
            egui::RichText::new(format!("Filtered: {} messages", filtered_count))
                .color(egui::Color32::YELLOW),
        );
    }
}

/// Shows message details dialog
fn show_message_details_dialog(ctx: &egui::Context, ui_state: &mut UiState) {
    let mut close_dialog = false;

    egui::Window::new("Message Details")
        .collapsible(false)
        .resizable(true)
        .default_width(600.0)
        .show(ctx, |ui| {
            if let Some(msg) = &ui_state.message_details_dialog {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Affiliation
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Affiliation:").strong());
                        let (color, icon, label) =
                            get_affiliation_display(msg.affiliation.as_deref());
                        ui.label(egui::RichText::new(icon).color(color));
                        ui.label(egui::RichText::new(label).color(color));
                    });

                    ui.add_space(5.0);

                    // Structured fields
                    egui::Grid::new("message_details_grid")
                        .num_columns(2)
                        .spacing([10.0, 5.0])
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Timestamp:").strong());
                            ui.label(msg.timestamp.to_rfc3339());
                            ui.end_row();

                            ui.label(egui::RichText::new("Server:").strong());
                            ui.label(&msg.server);
                            ui.end_row();

                            ui.label(egui::RichText::new("Type:").strong());
                            ui.label(&msg.msg_type);
                            ui.end_row();

                            if let Some(uid) = &msg.uid {
                                ui.label(egui::RichText::new("UID:").strong());
                                ui.label(uid);
                                ui.end_row();
                            }

                            if let Some(callsign) = &msg.callsign {
                                ui.label(egui::RichText::new("Callsign:").strong());
                                ui.label(callsign);
                                ui.end_row();
                            }

                            if let Some(lat) = msg.lat {
                                ui.label(egui::RichText::new("Latitude:").strong());
                                ui.label(format!("{:.6}", lat));
                                ui.end_row();
                            }

                            if let Some(lon) = msg.lon {
                                ui.label(egui::RichText::new("Longitude:").strong());
                                ui.label(format!("{:.6}", lon));
                                ui.end_row();
                            }

                            if let Some(alt) = msg.altitude {
                                ui.label(egui::RichText::new("Altitude:").strong());
                                ui.label(format!("{:.2} m", alt));
                                ui.end_row();
                            }
                        });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Raw content
                    if let Some(raw) = &msg.raw_content {
                        ui.label(egui::RichText::new("Raw Content:").strong());
                        ui.add_space(5.0);

                        egui::Frame::NONE
                            .fill(egui::Color32::from_gray(30))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(60)))
                            .corner_radius(4.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                egui::ScrollArea::horizontal().show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut raw.as_str())
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(f32::INFINITY),
                                    );
                                });
                            });

                        ui.add_space(10.0);

                        if ui.button("Copy Raw Content").clicked() {
                            ui.ctx().copy_text(raw.clone());
                        }
                    }
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }
                });
            }
        });

    if close_dialog {
        ui_state.message_details_dialog = None;
    }
}

/// Check if a message passes all active filters
fn passes_filters(msg: &MessageLog, ui_state: &UiState) -> bool {
    // Text filter (search in server, content, type)
    if !ui_state.message_filter.is_empty() {
        let filter_lower = ui_state.message_filter.to_lowercase();
        let matches = msg.server.to_lowercase().contains(&filter_lower)
            || msg.content.to_lowercase().contains(&filter_lower)
            || msg.msg_type.to_lowercase().contains(&filter_lower)
            || msg
                .uid
                .as_ref()
                .map(|u| u.to_lowercase().contains(&filter_lower))
                .unwrap_or(false)
            || msg
                .callsign
                .as_ref()
                .map(|c| c.to_lowercase().contains(&filter_lower))
                .unwrap_or(false);

        if !matches {
            return false;
        }
    }

    // Server filter
    if !ui_state.server_filter.is_empty() {
        if !msg
            .server
            .to_lowercase()
            .contains(&ui_state.server_filter.to_lowercase())
        {
            return false;
        }
    }

    // Affiliation filter
    match ui_state.affiliation_filter {
        AffiliationFilter::All => true,
        AffiliationFilter::Friend => {
            msg.affiliation.as_deref() == Some("Friend")
                || msg.affiliation.as_deref() == Some("friend")
        }
        AffiliationFilter::Hostile => {
            msg.affiliation.as_deref() == Some("Hostile")
                || msg.affiliation.as_deref() == Some("hostile")
        }
        AffiliationFilter::Neutral => {
            msg.affiliation.as_deref() == Some("Neutral")
                || msg.affiliation.as_deref() == Some("neutral")
        }
        AffiliationFilter::Unknown => {
            msg.affiliation.as_deref() == Some("Unknown")
                || msg.affiliation.as_deref() == Some("unknown")
        }
        AffiliationFilter::AssumedFriend => {
            msg.affiliation.as_deref() == Some("Assumed Friend")
                || msg.affiliation.as_deref() == Some("assumed_friend")
        }
        AffiliationFilter::Suspect => {
            msg.affiliation.as_deref() == Some("Suspect")
                || msg.affiliation.as_deref() == Some("suspect")
        }
        AffiliationFilter::Pending => {
            msg.affiliation.as_deref() == Some("Pending")
                || msg.affiliation.as_deref() == Some("pending")
        }
    }
}
