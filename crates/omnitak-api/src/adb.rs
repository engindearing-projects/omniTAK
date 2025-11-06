//! ADB integration endpoints for pulling certificates from connected Android devices

use crate::auth::{AuthUser, RequireOperator};
use crate::types::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use omnitak_adb::{AdbClient, AtakPackage, TakCertificateBundle};
use omnitak_cert::{CertificateSource, parse_certificate_bundle, parse_pkcs12};
use omnitak_client::tls::{TlsClient, TlsClientConfig};
use omnitak_core::ConnectionId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;
use validator::Validate;

use super::rest::{ApiState, ConnectionInfo};

// ============================================================================
// Types
// ============================================================================

/// Request to pull certificates from device
#[derive(Debug, Deserialize, Validate)]
pub struct PullCertsRequest {
    /// Device serial number (optional, auto-detect if single device)
    pub device_serial: Option<String>,
    /// ATAK package name (defaults to civilian ATAK)
    #[serde(default = "default_package")]
    pub package: String,
    /// Output directory for certificates (defaults to "certs/from-device")
    #[serde(default = "default_cert_dir")]
    pub cert_dir: String,
    /// Automatically create TAK connection after pulling certs
    #[serde(default = "default_auto_connect")]
    pub auto_connect: bool,
}

fn default_package() -> String {
    "com.atakmap.app.civ".to_string()
}

fn default_cert_dir() -> String {
    "certs/from-device".to_string()
}

fn default_auto_connect() -> bool {
    true
}

/// Response after pulling certificates
#[derive(Debug, Serialize)]
pub struct PullCertsResponse {
    /// Success flag
    pub success: bool,
    /// Message
    pub message: String,
    /// Certificate bundle information
    pub bundle: Option<CertificateBundleInfo>,
    /// Connection ID if auto-connect was enabled
    pub connection_id: Option<String>,
}

/// Certificate bundle information
#[derive(Debug, Serialize)]
pub struct CertificateBundleInfo {
    /// Server address
    pub server_address: Option<String>,
    /// Server name
    pub server_name: String,
    /// Number of certificate files pulled
    pub cert_count: usize,
    /// Certificate file paths
    pub cert_files: Vec<String>,
}

/// List of connected ADB devices
#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    /// Connected devices
    pub devices: Vec<DeviceInfoResponse>,
}

/// Device information
#[derive(Debug, Serialize)]
pub struct DeviceInfoResponse {
    /// Device serial number
    pub serial: String,
    /// Device state
    pub state: String,
    /// Device model
    pub model: Option<String>,
    /// Device product
    pub product: Option<String>,
}

// ============================================================================
// Endpoints
// ============================================================================

/// GET /api/v1/adb/devices - List connected Android devices
pub async fn list_devices(
    _user: AuthUser,
) -> Result<Json<DeviceListResponse>, ApiError> {
    let adb = AdbClient::new();

    // Check if ADB is available
    if !adb.is_available() {
        return Err(ApiError::BadRequest {
            message: "ADB is not available. Make sure Android SDK Platform-Tools is installed.".to_string(),
        });
    }

    let devices = adb.list_devices().map_err(|e| ApiError::Internal {
        message: format!("Failed to list devices: {}", e),
    })?;

    let devices = devices
        .into_iter()
        .map(|d| DeviceInfoResponse {
            serial: d.serial,
            state: d.state,
            model: d.model,
            product: d.product,
        })
        .collect();

    Ok(Json(DeviceListResponse { devices }))
}

/// POST /api/v1/adb/pull-certs - Pull certificates from connected device
pub async fn pull_certificates(
    State(state): State<ApiState>,
    _user: AuthUser,
    Json(req): Json<PullCertsRequest>,
) -> Result<Json<PullCertsResponse>, ApiError> {
    req.validate().map_err(|e| ApiError::BadRequest {
        message: format!("Invalid request: {}", e),
    })?;

    let adb = AdbClient::new();

    // Check if ADB is available
    if !adb.is_available() {
        return Err(ApiError::BadRequest {
            message: "ADB is not available. Make sure Android SDK Platform-Tools is installed.".to_string(),
        });
    }

    // Get device
    let device = if let Some(serial) = req.device_serial {
        adb.get_device(&serial).map_err(|e| ApiError::NotFound {
            message: format!("Device not found: {}", e),
        })?
    } else {
        adb.auto_detect_device().map_err(|e| ApiError::BadRequest {
            message: format!("Failed to detect device: {}", e),
        })?
    };

    info!("Pulling certificates from device: {}", device.serial);

    // Determine ATAK package
    let package = match req.package.as_str() {
        "com.atakmap.app.civ" => AtakPackage::Civilian,
        "com.atakmap.app.mil" => AtakPackage::Military,
        _ => AtakPackage::Custom(Box::leak(req.package.clone().into_boxed_str())),
    };

    // Create output directory
    let cert_dir = PathBuf::from(&req.cert_dir);
    tokio::fs::create_dir_all(&cert_dir)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to create certificate directory: {}", e),
        })?;

    // Pull certificates
    let bundle = device
        .pull_tak_certificates(&cert_dir, package)
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to pull certificates: {}", e),
        })?;

    info!(
        "Successfully pulled {} certificate files",
        bundle.certificates.len()
    );

    // Build response
    let bundle_info = CertificateBundleInfo {
        server_address: bundle.server_address.clone(),
        server_name: bundle.server_name.clone(),
        cert_count: bundle.certificates.len(),
        cert_files: bundle
            .certificates
            .iter()
            .map(|c| c.local_path.display().to_string())
            .collect(),
    };

    // Auto-connect if requested
    let connection_id = if req.auto_connect {
        match create_connection_from_bundle(state, bundle).await {
            Ok(id) => {
                info!("Successfully created connection: {}", id);
                Some(id)
            }
            Err(e) => {
                warn!("Failed to auto-create connection: {}", e);
                None
            }
        }
    } else {
        None
    };

    Ok(Json(PullCertsResponse {
        success: true,
        message: format!(
            "Successfully pulled {} certificate files from device",
            bundle_info.cert_count
        ),
        bundle: Some(bundle_info),
        connection_id,
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a TAK connection from certificate bundle
async fn create_connection_from_bundle(
    state: ApiState,
    bundle: TakCertificateBundle,
) -> anyhow::Result<String> {
    info!("Creating TAK connection from certificate bundle...");

    // Find P12 file or PEM files
    let p12_file = bundle
        .certificates
        .iter()
        .find(|c| c.cert_type == omnitak_adb::CertificateType::P12);

    let client_cert = bundle
        .certificates
        .iter()
        .find(|c| c.cert_type == omnitak_adb::CertificateType::Client);

    let client_key = bundle
        .certificates
        .iter()
        .find(|c| c.cert_type == omnitak_adb::CertificateType::Key);

    let ca_cert = bundle
        .certificates
        .iter()
        .find(|c| c.cert_type == omnitak_adb::CertificateType::CA);

    // Determine certificate source
    let cert_source = if let Some(p12) = p12_file {
        info!("Using PKCS#12 certificate: {}", p12.local_path.display());

        // Note: PKCS#12 requires password, which we don't have
        // User will need to convert to PEM or provide password
        warn!("PKCS#12 files require password. Please convert to PEM format:");
        warn!("  openssl pkcs12 -in {} -out client.pem -clcerts -nokeys", p12.local_path.display());
        warn!("  openssl pkcs12 -in {} -out client.key -nocerts -nodes", p12.local_path.display());
        warn!("  openssl pkcs12 -in {} -out ca.pem -cacerts -nokeys", p12.local_path.display());

        return Err(anyhow::anyhow!(
            "PKCS#12 certificates require password. Please convert to PEM format first."
        ));
    } else if let (Some(cert), Some(key), Some(ca)) = (client_cert, client_key, ca_cert) {
        info!("Using PEM certificates");
        CertificateSource::Files {
            cert_path: cert.local_path.to_string_lossy().to_string(),
            key_path: key.local_path.to_string_lossy().to_string(),
            ca_path: Some(ca.local_path.to_string_lossy().to_string()),
        }
    } else {
        return Err(anyhow::anyhow!("No valid certificate files found"));
    };

    // Create connection ID
    let connection_id = ConnectionId::new(Uuid::new_v4().to_string());

    // Determine server address
    let server_address = bundle
        .server_address
        .or_else(|| {
            bundle
                .server_host
                .map(|h| format!("{}:{}", h, bundle.server_port))
        })
        .unwrap_or_else(|| format!("takserver.example.com:{}", bundle.server_port));

    info!("Connecting to TAK server: {}", server_address);

    // Create TLS client configuration
    let tls_config = TlsClientConfig {
        address: server_address.clone(),
        certificate_source: cert_source,
        validate_certs: true,
        reconnect: omnitak_client::ReconnectConfig {
            enabled: true,
            initial_delay: std::time::Duration::from_secs(1),
            max_delay: std::time::Duration::from_secs(60),
            backoff_multiplier: 2.0,
        },
    };

    // Create TLS client
    let mut client = TlsClient::new(connection_id.clone(), tls_config);

    // Spawn connection task
    let pool = state.pool.clone();
    tokio::spawn(async move {
        if let Err(e) = client.start(pool).await {
            error!("TLS client error: {}", e);
        }
    });

    // Add to connections list
    let conn_info = ConnectionInfo {
        id: connection_id.to_string(),
        address: server_address,
        protocol: "tls".to_string(),
        status: "connecting".to_string(),
        connected_at: Some(chrono::Utc::now()),
        last_activity: Some(chrono::Utc::now()),
        messages_sent: 0,
        messages_received: 0,
        bytes_sent: 0,
        bytes_received: 0,
    };

    state.connections.write().await.push(conn_info);

    Ok(connection_id.to_string())
}
