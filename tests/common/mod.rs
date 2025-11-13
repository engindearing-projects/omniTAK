//! Common test utilities and helpers for integration tests

use std::path::PathBuf;

/// Get the path to test fixtures
pub fn fixtures_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("tests").join("fixtures")
}

/// Get the path to plugin fixtures
pub fn plugins_fixtures_dir() -> PathBuf {
    fixtures_dir().join("plugins")
}

/// Load test filter metadata
pub fn load_test_filter_metadata() -> serde_json::Value {
    let path = plugins_fixtures_dir().join("test-filter-metadata.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test filter metadata from {:?}", path));
    serde_json::from_str(&content)
        .expect("Failed to parse test filter metadata")
}

/// Load test transformer metadata
pub fn load_test_transformer_metadata() -> serde_json::Value {
    let path = plugins_fixtures_dir().join("test-transformer-metadata.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test transformer metadata from {:?}", path));
    serde_json::from_str(&content)
        .expect("Failed to parse test transformer metadata")
}

/// Create a minimal mock WASM file for testing
/// This creates a valid (but minimal) WASM module
pub fn create_mock_wasm() -> Vec<u8> {
    // Minimal valid WASM module (magic number + version)
    // This won't do anything but is valid WASM
    vec![
        0x00, 0x61, 0x73, 0x6d, // Magic number: '\0asm'
        0x01, 0x00, 0x00, 0x00, // Version: 1
    ]
}

/// Write mock WASM to temporary file
pub fn write_mock_wasm_to_temp() -> PathBuf {
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let wasm_path = temp_dir.join("test-plugin-mock.wasm");

    let mut file = std::fs::File::create(&wasm_path)
        .expect("Failed to create temp WASM file");
    file.write_all(&create_mock_wasm())
        .expect("Failed to write WASM data");

    wasm_path
}

/// Clean up temporary test files
pub fn cleanup_temp_files() {
    let temp_dir = std::env::temp_dir();
    let wasm_path = temp_dir.join("test-plugin-mock.wasm");
    let _ = std::fs::remove_file(wasm_path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_dir_exists() {
        let dir = fixtures_dir();
        assert!(dir.exists(), "Fixtures directory should exist");
    }

    #[test]
    fn test_load_filter_metadata() {
        let metadata = load_test_filter_metadata();
        assert!(metadata.get("id").is_some());
        assert!(metadata.get("name").is_some());
        assert_eq!(metadata["id"], "test-filter-plugin");
    }

    #[test]
    fn test_load_transformer_metadata() {
        let metadata = load_test_transformer_metadata();
        assert!(metadata.get("id").is_some());
        assert!(metadata.get("name").is_some());
        assert_eq!(metadata["id"], "test-transformer-plugin");
    }

    #[test]
    fn test_create_mock_wasm() {
        let wasm = create_mock_wasm();
        assert!(!wasm.is_empty());
        assert_eq!(&wasm[0..4], b"\0asm"); // Check WASM magic number
    }

    #[test]
    fn test_write_mock_wasm() {
        let path = write_mock_wasm_to_temp();
        assert!(path.exists());

        let content = std::fs::read(&path).unwrap();
        assert_eq!(&content[0..4], b"\0asm");

        cleanup_temp_files();
    }
}
