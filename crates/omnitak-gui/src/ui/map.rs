//! Map panel for visualizing TAK positions with altitude.

use crate::{AppState, MessageLog};
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use walkers::{HttpTiles, Map, MapMemory, Position};
use walkers::sources::OpenStreetMap;

/// Map panel state (persisted)
#[derive(Serialize, Deserialize)]
pub struct MapPanelState {
    /// Follow latest position
    pub follow_mode: bool,

    /// Show altitude labels
    pub show_altitude: bool,

    /// Altitude color coding enabled
    pub altitude_coloring: bool,

    /// Minimum altitude for color scale (meters)
    pub min_altitude: f64,

    /// Maximum altitude for color scale (meters)
    pub max_altitude: f64,

    /// HTTP tile downloader (not serialized)
    #[serde(skip)]
    pub tiles: Option<HttpTiles>,

    /// Map memory/state (not serialized)
    #[serde(skip)]
    pub map_memory: Option<MapMemory>,
}

impl Default for MapPanelState {
    fn default() -> Self {
        Self {
            follow_mode: true,
            show_altitude: true,
            altitude_coloring: true,
            min_altitude: 0.0,
            max_altitude: 10000.0,
            tiles: None,
            map_memory: None,
        }
    }
}

impl MapPanelState {
    pub fn new() -> Self {
        Self {
            follow_mode: true,
            show_altitude: true,
            altitude_coloring: true,
            min_altitude: 0.0,
            max_altitude: 10000.0,
            tiles: None,
            map_memory: None,
        }
    }
}


/// Converts altitude to color (low = blue, mid = green, high = red)
fn altitude_to_color(altitude: f64, min_alt: f64, max_alt: f64) -> egui::Color32 {
    let normalized = ((altitude - min_alt) / (max_alt - min_alt)).clamp(0.0, 1.0);

    if normalized < 0.5 {
        // Blue to green (0.0 - 0.5)
        let t = normalized * 2.0;
        egui::Color32::from_rgb(
            0,
            (255.0 * t) as u8,
            (255.0 * (1.0 - t)) as u8,
        )
    } else {
        // Green to red (0.5 - 1.0)
        let t = (normalized - 0.5) * 2.0;
        egui::Color32::from_rgb(
            (255.0 * t) as u8,
            (255.0 * (1.0 - t)) as u8,
            0,
        )
    }
}

/// Renders a position marker on the map
fn draw_marker(
    painter: &egui::Painter,
    screen_pos: egui::Pos2,
    msg: &MessageLog,
    state: &MapPanelState,
) {
    let radius = 8.0;

    // Determine marker color
    let color = if state.altitude_coloring {
        if let Some(alt) = msg.altitude {
            altitude_to_color(alt, state.min_altitude, state.max_altitude)
        } else {
            egui::Color32::GRAY
        }
    } else {
        // Color by affiliation if available
        match msg.affiliation.as_deref() {
            Some("f") | Some("friend") => egui::Color32::BLUE,
            Some("h") | Some("hostile") => egui::Color32::RED,
            Some("n") | Some("neutral") => egui::Color32::GREEN,
            Some("u") | Some("unknown") => egui::Color32::YELLOW,
            _ => egui::Color32::GRAY,
        }
    };

    // Draw marker circle
    painter.circle_filled(screen_pos, radius, color);
    painter.circle_stroke(screen_pos, radius, egui::Stroke::new(2.0, egui::Color32::WHITE));

    // Draw callsign label
    if let Some(callsign) = &msg.callsign {
        let label_pos = screen_pos + egui::vec2(10.0, -10.0);
        painter.text(
            label_pos,
            egui::Align2::LEFT_CENTER,
            callsign,
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );
    }

    // Draw altitude label if enabled
    if state.show_altitude {
        if let Some(alt) = msg.altitude {
            let alt_text = format!("{:.0}m", alt);
            let alt_pos = screen_pos + egui::vec2(10.0, 5.0);
            painter.text(
                alt_pos,
                egui::Align2::LEFT_CENTER,
                &alt_text,
                egui::FontId::proportional(10.0),
                egui::Color32::LIGHT_GRAY,
            );
        }
    }
}

/// Shows the map panel
pub fn show(ui: &mut egui::Ui, app_state: &Arc<Mutex<AppState>>, map_state: &mut MapPanelState) {
    ui.heading("Tactical Map");

    // Controls panel
    ui.horizontal(|ui| {
        ui.checkbox(&mut map_state.follow_mode, "Follow Latest");
        ui.separator();
        ui.checkbox(&mut map_state.show_altitude, "Show Altitude");
        ui.separator();
        ui.checkbox(&mut map_state.altitude_coloring, "Altitude Colors");

        if map_state.altitude_coloring {
            ui.separator();
            ui.label("Alt Range:");
            ui.add(egui::DragValue::new(&mut map_state.min_altitude).suffix("m").speed(10.0));
            ui.label("-");
            ui.add(egui::DragValue::new(&mut map_state.max_altitude).suffix("m").speed(10.0));
        }
    });

    ui.separator();

    // Get messages with positions
    let state = app_state.lock().unwrap();
    let positions: Vec<MessageLog> = state
        .message_log
        .iter()
        .filter(|msg| msg.lat.is_some() && msg.lon.is_some())
        .cloned()
        .collect();
    drop(state);

    // Determine map center
    let center_pos = if map_state.follow_mode {
        // Follow latest position
        positions.last()
            .and_then(|msg| {
                if let (Some(lat), Some(lon)) = (msg.lat, msg.lon) {
                    Some(walkers::lat_lon(lat, lon))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| walkers::lat_lon(37.7749, -122.4194)) // Default to San Francisco
    } else {
        // Default center (will be overridden by map memory)
        walkers::lat_lon(37.7749, -122.4194)
    };

    // Initialize tiles if not already initialized
    if map_state.tiles.is_none() {
        map_state.tiles = Some(HttpTiles::new(OpenStreetMap, ui.ctx().to_owned()));
    }

    // Initialize map memory if not already initialized
    if map_state.map_memory.is_none() {
        map_state.map_memory = Some(MapMemory::default());
    }

    // Get mutable references
    let tiles = map_state.tiles.as_mut().unwrap();
    let memory = map_state.map_memory.as_mut().unwrap();

    // Map widget
    ui.add(Map::new(
        Some(tiles),
        memory,
        center_pos,
    ));

    // TODO: Draw markers using walkers Plugin trait
    // For now, show position count
    ui.label(format!("Tracking {} positions", positions.len()));

    // Position list (temporary debug view)
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            for msg in positions.iter().take(10) {
                if let (Some(lat), Some(lon)) = (msg.lat, msg.lon) {
                    let alt_str = msg.altitude
                        .map(|a| format!(" @ {:.0}m", a))
                        .unwrap_or_default();

                    let callsign = msg.callsign.as_deref().unwrap_or("Unknown");

                    ui.label(format!(
                        "{}: {:.4}, {:.4}{}",
                        callsign, lat, lon, alt_str
                    ));
                }
            }
        });
}
