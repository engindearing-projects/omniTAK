use super::bindings::omnitak::plugins::host;

/// Convenience wrappers for calling host functions
///
/// These functions provide ergonomic access to host-provided functionality
/// by converting types to and from what the host expects.

/// Log a simple message to the host's logging system
pub fn log(message: &str) {
    host::log(message);
}

/// Log a structured message with key-value properties
///
/// This is useful for adding context to log messages that can be parsed
/// and filtered by the host's logging system.
///
/// # Example
/// ```no_run
/// log_structured(
///     "Message filtered",
///     &[("plugin", "example-filter"), ("action", "tagged")]
/// );
/// ```
pub fn log_structured(message: &str, properties: &[(&str, &str)]) {
    let properties: Vec<(String, String)> = properties
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    host::log_structured(message, &properties);
}
