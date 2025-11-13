//! Plugin management UI panel

use crate::{AppState, StatusLevel};
use eframe::egui;
use omnitak_plugin_api::{PluginInfo, PluginCapability};
use std::sync::{Arc, Mutex};

pub struct PluginPanelState {
    /// Search/filter text
    pub filter_text: String,

    /// Show only enabled plugins
    pub show_enabled_only: bool,

    /// Plugin type filter
    pub type_filter: PluginTypeFilter,

    /// Plugin being viewed in detail
    pub selected_plugin: Option<String>,

    /// Load plugin dialog state
    pub load_dialog: Option<LoadPluginDialog>,

    /// Plugin configuration editor
    pub config_editor: Option<ConfigEditorDialog>,
}

impl Default for PluginPanelState {
    fn default() -> Self {
        Self {
            filter_text: String::new(),
            show_enabled_only: false,
            type_filter: PluginTypeFilter::All,
            selected_plugin: None,
            load_dialog: None,
            config_editor: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginTypeFilter {
    All,
    Filter,
    Transformer,
}

pub struct LoadPluginDialog {
    pub plugin_id: String,
    pub plugin_path: String,
    pub plugin_type: String,
    pub enabled: bool,
    pub error_message: Option<String>,
}

impl Default for LoadPluginDialog {
    fn default() -> Self {
        Self {
            plugin_id: String::new(),
            plugin_path: String::new(),
            plugin_type: "filter".to_string(),
            enabled: true,
            error_message: None,
        }
    }
}

pub struct ConfigEditorDialog {
    pub plugin_id: String,
    pub config_json: String,
    pub error_message: Option<String>,
}

/// Render the plugins panel
pub fn render_plugins_panel(
    ui: &mut egui::Ui,
    state: &Arc<Mutex<AppState>>,
    panel_state: &mut PluginPanelState,
) -> Option<(String, StatusLevel)> {
    let mut status_message = None;

    ui.heading("Plugin Management");
    ui.add_space(10.0);

    // Toolbar
    ui.horizontal(|ui| {
        if ui.button("Load Plugin").clicked() {
            panel_state.load_dialog = Some(LoadPluginDialog::default());
        }

        ui.add_space(10.0);

        if ui.button("Reload All").clicked() {
            // TODO: Call API to reload all plugins
            status_message = Some(("Reloading all plugins...".to_string(), StatusLevel::Info));
        }

        ui.add_space(10.0);

        ui.label("Filter:");
        ui.text_edit_singleline(&mut panel_state.filter_text);

        ui.add_space(10.0);

        ui.checkbox(&mut panel_state.show_enabled_only, "Enabled only");

        ui.add_space(10.0);

        egui::ComboBox::from_label("Type")
            .selected_text(format!("{:?}", panel_state.type_filter))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut panel_state.type_filter, PluginTypeFilter::All, "All");
                ui.selectable_value(&mut panel_state.type_filter, PluginTypeFilter::Filter, "Filter");
                ui.selectable_value(&mut panel_state.type_filter, PluginTypeFilter::Transformer, "Transformer");
            });
    });

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    // Plugin list
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // TODO: Get actual plugins from backend
            // For now, show placeholder
            let plugins = get_mock_plugins();

            if plugins.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label("No plugins loaded");
                    ui.add_space(10.0);
                    ui.label("Click 'Load Plugin' to add one");
                });
            } else {
                for plugin in plugins {
                    if let Some(msg) = render_plugin_card(ui, &plugin, panel_state) {
                        status_message = Some(msg);
                    }
                    ui.add_space(5.0);
                }
            }
        });

    // Load plugin dialog
    if let Some(ref mut dialog) = panel_state.load_dialog {
        let mut open = true;
        egui::Window::new("Load Plugin")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                if let Some(msg) = render_load_plugin_dialog(ui, dialog) {
                    status_message = Some(msg);
                    panel_state.load_dialog = None;
                }
            });

        if !open {
            panel_state.load_dialog = None;
        }
    }

    // Config editor dialog
    if let Some(ref mut dialog) = panel_state.config_editor {
        let mut open = true;
        egui::Window::new(format!("Configure Plugin: {}", dialog.plugin_id))
            .open(&mut open)
            .resizable(true)
            .default_width(600.0)
            .default_height(400.0)
            .show(ui.ctx(), |ui| {
                if let Some(msg) = render_config_editor_dialog(ui, dialog) {
                    status_message = Some(msg);
                    panel_state.config_editor = None;
                }
            });

        if !open {
            panel_state.config_editor = None;
        }
    }

    status_message
}

fn render_plugin_card(
    ui: &mut egui::Ui,
    plugin: &PluginInfo,
    panel_state: &mut PluginPanelState,
) -> Option<(String, StatusLevel)> {
    let mut status_message = None;

    egui::Frame::group(ui.style())
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Plugin icon based on type
                let icon = if plugin.capabilities.contains(&PluginCapability::Filter) {
                    "🔍"
                } else if plugin.capabilities.contains(&PluginCapability::Transform) {
                    "⚙️"
                } else {
                    "📦"
                };

                ui.label(egui::RichText::new(icon).size(24.0));

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&plugin.name);
                        ui.label(egui::RichText::new(&plugin.version).weak());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Enabled toggle
                            let mut enabled = true; // TODO: Track actual state
                            if ui.checkbox(&mut enabled, "Enabled").changed() {
                                status_message = Some((
                                    format!("Plugin {} {}", plugin.id, if enabled { "enabled" } else { "disabled" }),
                                    StatusLevel::Info,
                                ));
                            }
                        });
                    });

                    ui.label(&plugin.description);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("by {}", plugin.author)).weak());
                        ui.add_space(10.0);

                        // Capability badges
                        for cap in &plugin.capabilities {
                            let (text, color) = match cap {
                                PluginCapability::Filter => ("Filter", egui::Color32::from_rgb(100, 149, 237)),
                                PluginCapability::Transform => ("Transform", egui::Color32::from_rgb(144, 238, 144)),
                                PluginCapability::NetworkAccess => ("Network", egui::Color32::from_rgb(255, 165, 0)),
                                PluginCapability::FilesystemAccess => ("Filesystem", egui::Color32::from_rgb(255, 99, 71)),
                            };

                            ui.label(
                                egui::RichText::new(text)
                                    .small()
                                    .background_color(color)
                                    .color(egui::Color32::WHITE)
                            );
                        }
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        if ui.button("Configure").clicked() {
                            panel_state.config_editor = Some(ConfigEditorDialog {
                                plugin_id: plugin.id.clone(),
                                config_json: "{}".to_string(),
                                error_message: None,
                            });
                        }

                        if ui.button("Metrics").clicked() {
                            panel_state.selected_plugin = Some(plugin.id.clone());
                        }

                        if ui.button("Reload").clicked() {
                            status_message = Some((
                                format!("Reloading plugin: {}", plugin.id),
                                StatusLevel::Info,
                            ));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Unload").clicked() {
                                status_message = Some((
                                    format!("Unloading plugin: {}", plugin.id),
                                    StatusLevel::Warning,
                                ));
                            }
                        });
                    });
                });
            });

            // Expandable metrics section
            if panel_state.selected_plugin.as_ref() == Some(&plugin.id) {
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                render_plugin_metrics(ui, plugin);

                if ui.button("Close Metrics").clicked() {
                    panel_state.selected_plugin = None;
                }
            }
        });

    status_message
}

fn render_plugin_metrics(ui: &mut egui::Ui, plugin: &PluginInfo) {
    ui.heading("Plugin Metrics");
    ui.add_space(5.0);

    egui::Grid::new(format!("metrics_{}", plugin.id))
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Executions:");
            ui.label("0"); // TODO: Real data
            ui.end_row();

            ui.label("Errors:");
            ui.label("0");
            ui.end_row();

            ui.label("Timeouts:");
            ui.label("0");
            ui.end_row();

            ui.label("Avg Execution Time:");
            ui.label("0.5 ms");
            ui.end_row();

            ui.label("P99 Execution Time:");
            ui.label("2.1 ms");
            ui.end_row();

            ui.label("Last Execution:");
            ui.label("2 minutes ago");
            ui.end_row();
        });

    ui.add_space(10.0);

    // Health status
    ui.horizontal(|ui| {
        ui.label("Health:");
        ui.label(egui::RichText::new("Healthy").color(egui::Color32::GREEN));
    });
}

fn render_load_plugin_dialog(
    ui: &mut egui::Ui,
    dialog: &mut LoadPluginDialog,
) -> Option<(String, StatusLevel)> {
    let mut status_message = None;

    ui.add_space(10.0);

    egui::Grid::new("load_plugin_grid")
        .num_columns(2)
        .spacing([10.0, 10.0])
        .show(ui, |ui| {
            ui.label("Plugin ID:");
            ui.text_edit_singleline(&mut dialog.plugin_id);
            ui.end_row();

            ui.label("Plugin Path:");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut dialog.plugin_path);
                if ui.button("Browse...").clicked() {
                    // TODO: File picker
                }
            });
            ui.end_row();

            ui.label("Plugin Type:");
            egui::ComboBox::from_id_source("plugin_type")
                .selected_text(&dialog.plugin_type)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut dialog.plugin_type, "filter".to_string(), "Filter");
                    ui.selectable_value(&mut dialog.plugin_type, "transformer".to_string(), "Transformer");
                });
            ui.end_row();

            ui.label("Enabled:");
            ui.checkbox(&mut dialog.enabled, "");
            ui.end_row();
        });

    ui.add_space(10.0);

    if let Some(ref error) = dialog.error_message {
        ui.label(egui::RichText::new(error).color(egui::Color32::RED));
        ui.add_space(5.0);
    }

    ui.horizontal(|ui| {
        if ui.button("Load").clicked() {
            if dialog.plugin_id.is_empty() {
                dialog.error_message = Some("Plugin ID is required".to_string());
            } else if dialog.plugin_path.is_empty() {
                dialog.error_message = Some("Plugin path is required".to_string());
            } else {
                // TODO: Call API to load plugin
                status_message = Some((
                    format!("Loading plugin: {}", dialog.plugin_id),
                    StatusLevel::Success,
                ));
            }
        }

        if ui.button("Cancel").clicked() {
            status_message = Some(("Cancelled".to_string(), StatusLevel::Info));
        }
    });

    status_message
}

fn render_config_editor_dialog(
    ui: &mut egui::Ui,
    dialog: &mut ConfigEditorDialog,
) -> Option<(String, StatusLevel)> {
    let mut status_message = None;

    ui.add_space(10.0);

    ui.label("Plugin Configuration (JSON):");
    ui.add_space(5.0);

    egui::ScrollArea::vertical()
        .max_height(300.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut dialog.config_json)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(15)
            );
        });

    ui.add_space(10.0);

    if let Some(ref error) = dialog.error_message {
        ui.label(egui::RichText::new(error).color(egui::Color32::RED));
        ui.add_space(5.0);
    }

    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            // Validate JSON
            match serde_json::from_str::<serde_json::Value>(&dialog.config_json) {
                Ok(_) => {
                    // TODO: Call API to update config
                    status_message = Some((
                        format!("Configuration updated for plugin: {}", dialog.plugin_id),
                        StatusLevel::Success,
                    ));
                }
                Err(e) => {
                    dialog.error_message = Some(format!("Invalid JSON: {}", e));
                }
            }
        }

        if ui.button("Cancel").clicked() {
            status_message = Some(("Cancelled".to_string(), StatusLevel::Info));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Load Example").clicked() {
                dialog.config_json = serde_json::to_string_pretty(&serde_json::json!({
                    "center_lat": 35.0,
                    "center_lon": -79.0,
                    "radius_degrees": 2.0,
                    "enabled": true
                })).unwrap();
            }
        });
    });

    status_message
}

// Mock data for development
fn get_mock_plugins() -> Vec<PluginInfo> {
    vec![
        PluginInfo {
            id: "geofence-fort-bragg".to_string(),
            name: "Fort Bragg Geofence Filter".to_string(),
            version: "0.1.0".to_string(),
            author: "OmniTAK Team".to_string(),
            description: "Filters CoT messages to only allow positions within Fort Bragg, NC area".to_string(),
            capabilities: vec![PluginCapability::Filter],
            binary_hash: "a1b2c3d4e5f6...".to_string(),
        },
        PluginInfo {
            id: "flightradar24-source".to_string(),
            name: "FlightRadar24 Integration".to_string(),
            version: "0.1.0".to_string(),
            author: "OmniTAK Community".to_string(),
            description: "Fetches live flight data from FlightRadar24 and converts to CoT messages".to_string(),
            capabilities: vec![
                PluginCapability::Transform,
                PluginCapability::NetworkAccess,
            ],
            binary_hash: "f6e5d4c3b2a1...".to_string(),
        },
    ]
}
