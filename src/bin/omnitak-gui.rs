//! OmniTAK GUI Application
//!
//! Desktop GUI for managing OmniTAK TAK server connections.

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // Set up logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(
                // Load icon if available
                eframe::icon_data::from_png_bytes(&[])
                    .unwrap_or_else(|_| eframe::IconData {
                        rgba: vec![],
                        width: 0,
                        height: 0,
                    }),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "OmniTAK - TAK Server Aggregator",
        options,
        Box::new(|cc| Ok(Box::new(omnitak_gui::OmniTakApp::new(cc)))),
    )
}
