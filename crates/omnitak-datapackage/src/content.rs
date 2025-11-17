//! Package content types and metadata

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::TakFileType;

/// Type of content in a data package
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// CoT event data
    CotEvent,
    /// Map overlay (KML/KMZ)
    MapOverlay,
    /// Map tiles (MBTiles)
    MapTiles,
    /// Configuration file
    Configuration,
    /// Certificate/key material
    Certificate,
    /// User preferences
    Preferences,
    /// Image or other attachment
    Attachment,
    /// Route/waypoint data
    Route,
    /// Unknown content type
    Unknown,
}

impl From<TakFileType> for ContentType {
    fn from(file_type: TakFileType) -> Self {
        match file_type {
            TakFileType::CotEvent => ContentType::CotEvent,
            TakFileType::KmlOverlay => ContentType::MapOverlay,
            TakFileType::MapTiles => ContentType::MapTiles,
            TakFileType::Config => ContentType::Configuration,
            TakFileType::Certificate => ContentType::Certificate,
            TakFileType::Preferences => ContentType::Preferences,
            TakFileType::Attachment => ContentType::Attachment,
            TakFileType::Unknown => ContentType::Unknown,
        }
    }
}

/// Metadata for a single piece of content in a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageContent {
    /// Path within the package (zip entry)
    pub path: String,
    /// Content type
    pub content_type: ContentType,
    /// File size in bytes
    pub size: u64,
    /// Whether this content should be ignored
    pub ignore: bool,
    /// Optional description
    pub description: Option<String>,
    /// Original file path (if created from local file)
    pub source_path: Option<PathBuf>,
}

impl PackageContent {
    /// Create content metadata from a file path
    pub fn from_path<P: AsRef<Path>>(path: P, zip_entry: &str) -> Self {
        let path_ref = path.as_ref();
        let file_type = path_ref
            .extension()
            .and_then(|e| e.to_str())
            .map(TakFileType::from_extension)
            .unwrap_or(TakFileType::Unknown);

        let size = path_ref.metadata().map(|m| m.len()).unwrap_or(0);

        Self {
            path: zip_entry.to_string(),
            content_type: ContentType::from(file_type),
            size,
            ignore: false,
            description: None,
            source_path: Some(path_ref.to_path_buf()),
        }
    }

    /// Create content metadata for in-memory data
    pub fn from_bytes(zip_entry: &str, data: &[u8], content_type: ContentType) -> Self {
        Self {
            path: zip_entry.to_string(),
            content_type,
            size: data.len() as u64,
            ignore: false,
            description: None,
            source_path: None,
        }
    }
}

/// Summary of package contents
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageSummary {
    /// Total number of files
    pub total_files: usize,
    /// Total size in bytes
    pub total_size: u64,
    /// Number of CoT events
    pub cot_events: usize,
    /// Number of map overlays
    pub map_overlays: usize,
    /// Number of map tile files
    pub map_tiles: usize,
    /// Number of config files
    pub configs: usize,
    /// Number of certificates
    pub certificates: usize,
    /// Number of attachments
    pub attachments: usize,
    /// Number of unknown files
    pub unknown: usize,
}

impl PackageSummary {
    /// Add content to the summary
    pub fn add_content(&mut self, content: &PackageContent) {
        self.total_files += 1;
        self.total_size += content.size;

        match content.content_type {
            ContentType::CotEvent => self.cot_events += 1,
            ContentType::MapOverlay => self.map_overlays += 1,
            ContentType::MapTiles => self.map_tiles += 1,
            ContentType::Configuration | ContentType::Preferences => self.configs += 1,
            ContentType::Certificate => self.certificates += 1,
            ContentType::Attachment | ContentType::Route => self.attachments += 1,
            ContentType::Unknown => self.unknown += 1,
        }
    }

    /// Get human-readable size
    pub fn human_readable_size(&self) -> String {
        let size = self.total_size;
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}
