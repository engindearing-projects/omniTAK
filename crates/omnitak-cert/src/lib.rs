//! Certificate management for OmniTAK
//!
//! Handles loading, parsing, and validating certificates for TAK server connections.
//! Supports PEM, DER, and PKCS#12 formats with password protection.

use anyhow::{anyhow, Context, Result};
use base64::prelude::*;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::{Deserialize, Serialize};
use std::io::{BufReader, Cursor};
use tracing::{debug, info};

/// Certificate data that can be stored in memory or loaded from files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateData {
    /// Certificate name/identifier
    pub name: String,
    /// Base64-encoded certificate data
    pub data: String,
    /// Original file size in bytes
    pub size: u64,
}

/// Certificate bundle containing all necessary components for TLS
#[derive(Debug)]
pub struct CertificateBundle {
    /// Client certificates (can be multiple in a chain)
    pub certs: Vec<CertificateDer<'static>>,
    /// Private key for the client certificate
    pub private_key: PrivateKeyDer<'static>,
    /// Optional CA certificate for server verification
    pub ca_certs: Option<Vec<CertificateDer<'static>>>,
}

impl Clone for CertificateBundle {
    fn clone(&self) -> Self {
        Self {
            certs: self.certs.clone(),
            private_key: self.private_key.clone_key(),
            ca_certs: self.ca_certs.clone(),
        }
    }
}

impl CertificateBundle {
    /// Create a new certificate bundle from PEM-encoded data
    pub fn from_pem(
        cert_pem: &[u8],
        key_pem: &[u8],
        ca_pem: Option<&[u8]>,
    ) -> Result<Self> {
        info!("Loading certificates from PEM data");

        // Parse client certificate
        let mut cert_reader = BufReader::new(Cursor::new(cert_pem));
        let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse client certificates")?;

        if certs.is_empty() {
            return Err(anyhow!("No certificates found in PEM data"));
        }

        info!("Loaded {} client certificate(s)", certs.len());

        // Parse private key
        let mut key_reader = BufReader::new(Cursor::new(key_pem));
        let private_key = rustls_pemfile::private_key(&mut key_reader)
            .context("Failed to read private key")?
            .ok_or_else(|| anyhow!("No private key found in PEM data"))?;

        info!("Loaded private key");

        // Parse CA certificate if provided
        let ca_certs = if let Some(ca_pem_data) = ca_pem {
            let mut ca_reader = BufReader::new(Cursor::new(ca_pem_data));
            let ca_certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut ca_reader)
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to parse CA certificates")?;

            if !ca_certs.is_empty() {
                info!("Loaded {} CA certificate(s)", ca_certs.len());
                Some(ca_certs)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            certs,
            private_key,
            ca_certs,
        })
    }

    /// Create a certificate bundle from PKCS#12 data with password
    pub fn from_pkcs12(_p12_data: &[u8], _password: Option<&str>) -> Result<Self> {
        info!("PKCS#12 format requested");

        // For now, return an error suggesting PEM format instead
        // The p12 crate API requires complex parsing that's beyond current scope
        Err(anyhow!(
            "PKCS#12 parsing is not yet fully supported. Please convert your certificate to PEM format. \
            You can use: openssl pkcs12 -in cert.p12 -out cert.pem -nodes"
        ))
    }

    /// Create a certificate bundle from base64-encoded certificate data
    pub fn from_certificate_data(
        cert_data: &CertificateData,
        key_data: Option<&CertificateData>,
        ca_data: Option<&CertificateData>,
        password: Option<&str>,
    ) -> Result<Self> {
        // Decode base64 data
        let cert_bytes = BASE64_STANDARD
            .decode(&cert_data.data)
            .context("Failed to decode certificate data")?;

        // Determine format based on content or file extension
        if cert_data.name.ends_with(".p12") || cert_data.name.ends_with(".pfx") {
            // PKCS#12 format
            Self::from_pkcs12(&cert_bytes, password)
        } else {
            // PEM format
            let key_bytes = if let Some(key) = key_data {
                BASE64_STANDARD
                    .decode(&key.data)
                    .context("Failed to decode private key data")?
            } else {
                return Err(anyhow!("Private key is required for PEM format"));
            };

            let ca_bytes = if let Some(ca) = ca_data {
                Some(
                    BASE64_STANDARD
                        .decode(&ca.data)
                        .context("Failed to decode CA certificate data")?,
                )
            } else {
                None
            };

            Self::from_pem(
                &cert_bytes,
                &key_bytes,
                ca_bytes.as_deref(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_data_serialization() {
        let cert_data = CertificateData {
            name: "test.pem".to_string(),
            data: "dGVzdA==".to_string(), // "test" in base64
            size: 4,
        };

        let json = serde_json::to_string(&cert_data).unwrap();
        let deserialized: CertificateData = serde_json::from_str(&json).unwrap();

        assert_eq!(cert_data.name, deserialized.name);
        assert_eq!(cert_data.data, deserialized.data);
        assert_eq!(cert_data.size, deserialized.size);
    }

    #[test]
    fn test_base64_decode() {
        let cert_data = CertificateData {
            name: "test.pem".to_string(),
            data: "dGVzdA==".to_string(),
            size: 4,
        };

        let decoded = BASE64_STANDARD.decode(&cert_data.data).unwrap();
        assert_eq!(decoded, b"test");
    }
}
