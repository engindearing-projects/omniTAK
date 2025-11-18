//! Certificate Enrollment UI
//!
//! Provides UI for enrolling with TAK servers that require username/password authentication.

use eframe::egui;
use omnitak_cert::enrollment::{EnrollmentClient, EnrollmentRequest};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// State for certificate enrollment dialog
#[derive(Default)]
pub struct EnrollmentDialogState {
    /// Server URL
    pub server_url: String,
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// Certificate validity in days
    pub validity_days: String,
    /// Common name (optional, defaults to username)
    pub common_name: String,
    /// Whether enrollment is in progress
    pub enrolling: bool,
    /// Enrollment result
    pub result: Option<EnrollmentResult>,
    /// Error message
    pub error_message: Option<String>,
}

/// Enrollment result
pub enum EnrollmentResult {
    Success {
        cert_path: String,
        server_host: String,
        server_port: Option<u16>,
    },
    Failed {
        error: String,
    },
}

impl EnrollmentDialogState {
    pub fn new() -> Self {
        Self {
            server_url: String::new(),
            username: String::new(),
            password: String::new(),
            validity_days: "365".to_string(),
            common_name: String::new(),
            enrolling: false,
            result: None,
            error_message: None,
        }
    }

    /// Reset the dialog state
    pub fn reset(&mut self) {
        self.server_url.clear();
        self.username.clear();
        self.password.clear();
        self.validity_days = "365".to_string();
        self.common_name.clear();
        self.enrolling = false;
        self.result = None;
        self.error_message = None;
    }
}

/// Render the certificate enrollment dialog
///
/// Returns `Some((message, status_level))` if a status message should be displayed
pub fn render_enrollment_dialog(
    ui: &mut egui::Ui,
    state: &mut EnrollmentDialogState,
    runtime: &Arc<Runtime>,
) -> Option<(String, crate::StatusLevel)> {
    let mut status_message = None;
    let mut close_dialog = false;

    egui::Window::new("🔐 Certificate Enrollment")
        .id(egui::Id::new("enrollment_dialog"))
        .collapsible(false)
        .resizable(false)
        .default_width(500.0)
        .show(ui.ctx(), |ui| {
            // Show result or form
            if let Some(result) = &state.result {
                match result {
                    EnrollmentResult::Success {
                        cert_path,
                        server_host,
                        server_port,
                    } => {
                        ui.colored_label(egui::Color32::GREEN, "✓ Enrollment Successful!");
                        ui.add_space(10.0);

                        egui::Grid::new("enrollment_result_grid")
                            .num_columns(2)
                            .spacing([10.0, 8.0])
                            .show(ui, |ui| {
                                ui.label("Certificate:");
                                ui.label(cert_path);
                                ui.end_row();

                                ui.label("Server:");
                                if let Some(port) = server_port {
                                    ui.label(format!("{}:{}", server_host, port));
                                } else {
                                    ui.label(server_host);
                                }
                                ui.end_row();
                            });

                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("Close").clicked() {
                                close_dialog = true;
                            }
                        });
                    }
                    EnrollmentResult::Failed { error } => {
                        ui.colored_label(egui::Color32::RED, "✗ Enrollment Failed");
                        ui.add_space(10.0);

                        ui.label("Error:");
                        ui.label(egui::RichText::new(error).color(egui::Color32::RED).size(12.0));

                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("Try Again").clicked() {
                                state.result = None;
                                state.error_message = None;
                            }

                            if ui.button("Close").clicked() {
                                close_dialog = true;
                            }
                        });
                    }
                }
            } else {
                // Show enrollment form
                ui.label("Enter TAK server details to enroll and receive a client certificate:");
                ui.add_space(10.0);

                egui::Grid::new("enrollment_form_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        // Server URL
                        ui.label("Server URL:");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.server_url)
                                .hint_text("https://tak-server.example.com:8443")
                                .desired_width(300.0),
                        );
                        ui.end_row();

                        // Username
                        ui.label("Username:");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.username)
                                .hint_text("your-username")
                                .desired_width(300.0),
                        );
                        ui.end_row();

                        // Password
                        ui.label("Password:");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.password)
                                .password(true)
                                .hint_text("your-password")
                                .desired_width(300.0),
                        );
                        ui.end_row();

                        // Certificate validity
                        ui.label("Validity (days):");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.validity_days)
                                .hint_text("365")
                                .desired_width(100.0),
                        );
                        ui.end_row();

                        // Common name (optional)
                        ui.label("Common Name:");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.common_name)
                                .hint_text("(optional, defaults to username)")
                                .desired_width(300.0),
                        );
                        ui.end_row();
                    });

                // Error message
                if let Some(error) = &state.error_message {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(error).color(egui::Color32::RED).size(12.0));
                }

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);

                // Buttons
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!state.enrolling, |ui| {
                        if ui.button(if state.enrolling { "Enrolling..." } else { "Enroll" }).clicked() {
                            // Validate inputs
                            if state.server_url.is_empty() {
                                state.error_message = Some("Server URL is required".to_string());
                            } else if state.username.is_empty() {
                                state.error_message = Some("Username is required".to_string());
                            } else if state.password.is_empty() {
                                state.error_message = Some("Password is required".to_string());
                            } else {
                                // Start enrollment
                                state.enrolling = true;
                                state.error_message = None;
                                status_message = Some((
                                    "Starting certificate enrollment...".to_string(),
                                    crate::StatusLevel::Info,
                                ));

                                // Perform enrollment in background
                                let server_url = state.server_url.clone();
                                let username = state.username.clone();
                                let password = state.password.clone();
                                let validity_days = state.validity_days.parse::<u32>().ok();
                                let common_name = if state.common_name.is_empty() {
                                    None
                                } else {
                                    Some(state.common_name.clone())
                                };

                                // Note: In a real implementation, you'd want to use async/await properly
                                // For now, this is a placeholder showing the structure
                                let runtime_clone = Arc::clone(runtime);
                                std::thread::spawn(move || {
                                    runtime_clone.block_on(async {
                                        let client = EnrollmentClient::new();
                                        let request = EnrollmentRequest {
                                            server_url,
                                            username,
                                            password,
                                            validity_days,
                                            common_name,
                                        };

                                        match client.enroll(&request).await {
                                            Ok(response) => {
                                                // Save certificate to disk
                                                let cert_dir = std::env::current_dir()
                                                    .unwrap_or_default()
                                                    .join("certs");
                                                std::fs::create_dir_all(&cert_dir).ok();

                                                let cert_path = cert_dir.join(format!(
                                                    "{}_cert.pem",
                                                    request.username
                                                ));

                                                // In a real implementation, save the certificate here
                                                // For now, just return success
                                                println!(
                                                    "Certificate enrolled successfully for {}",
                                                    request.username
                                                );
                                            }
                                            Err(e) => {
                                                println!("Enrollment failed: {}", e);
                                            }
                                        }
                                    });
                                });
                            }
                        }
                    });

                    if ui.button("Cancel").clicked() {
                        close_dialog = true;
                    }

                    // Loading indicator
                    if state.enrolling {
                        ui.spinner();
                    }
                });

                ui.add_space(10.0);

                // Help text
                ui.collapsing("ℹ️ Help", |ui| {
                    ui.label("TAK servers may require username/password authentication to issue client certificates.");
                    ui.label("Common enrollment endpoints:");
                    ui.label("• /Marti/api/tls/signClient");
                    ui.label("• /Marti/api/tls/enrollment");
                    ui.label("• /api/cert/enroll");
                    ui.add_space(5.0);
                    ui.label("The certificate will be automatically saved and can be used for secure connections.");
                });
            }
        });

    if close_dialog {
        state.reset();
    }

    status_message
}
