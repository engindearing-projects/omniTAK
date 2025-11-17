//! Error types for TAK Data Package operations

use thiserror::Error;

/// Result type for data package operations
pub type Result<T> = std::result::Result<T, DataPackageError>;

/// Errors that can occur during data package operations
#[derive(Error, Debug)]
pub enum DataPackageError {
    /// I/O error during file operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// ZIP archive error
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// XML parsing error
    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    /// Invalid manifest structure
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    /// Missing required file in package
    #[error("Missing required file: {0}")]
    MissingFile(String),

    /// Package validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Unsupported package version
    #[error("Unsupported package version: {0}")]
    UnsupportedVersion(String),

    /// Content type mismatch
    #[error("Content type mismatch: expected {expected}, got {actual}")]
    ContentTypeMismatch { expected: String, actual: String },

    /// Path traversal attack detected
    #[error("Security error: path traversal detected in {0}")]
    PathTraversal(String),

    /// Package too large
    #[error("Package size {size} exceeds maximum {max_size}")]
    PackageTooLarge { size: u64, max_size: u64 },

    /// Invalid UTF-8 in manifest
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}
