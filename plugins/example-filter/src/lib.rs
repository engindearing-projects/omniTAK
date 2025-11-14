mod bindings;
mod host;

use bindings::{export, exports::omnitak::plugins::message_filter::Guest};

/// Example Message Filter Plugin
///
/// This plugin demonstrates how to create a simple message filter for CoT XML.
/// It checks for hostile keywords and can modify or reject messages based on content.
pub struct ExampleFilterPlugin;

// Export the plugin
export!(ExampleFilterPlugin);

impl Guest for ExampleFilterPlugin {
    /// Filter a CoT XML message
    ///
    /// This implementation:
    /// - Checks for hostile keywords in the message
    /// - Tags hostile messages with a warning attribute
    /// - Logs all filtering actions
    fn filter_message(cot_xml: String) -> Result<String, String> {
        host::log("ExampleFilterPlugin: Processing message...");

        // Check if the message is empty
        if cot_xml.trim().is_empty() {
            host::log("ExampleFilterPlugin: Empty message received");
            return Err("Message is empty".to_string());
        }

        // Define hostile keywords to filter
        let hostile_keywords = vec![
            "hostile",
            "enemy",
            "threat",
            "attack",
            "danger",
        ];

        // Check for hostile keywords (case-insensitive)
        let lower_cot = cot_xml.to_lowercase();
        let mut found_keywords = Vec::new();

        for keyword in &hostile_keywords {
            if lower_cot.contains(keyword) {
                found_keywords.push(*keyword);
            }
        }

        if !found_keywords.is_empty() {
            // Log the detection
            let keywords_str = found_keywords.join(", ");
            host::log_structured(
                "ExampleFilterPlugin: Hostile keywords detected",
                &[
                    ("keywords", &keywords_str),
                    ("action", "tagged"),
                ],
            );

            // Modify the message to add a warning tag
            // In a real implementation, you would use proper XML parsing
            let modified = if cot_xml.contains("<event") {
                cot_xml.replace(
                    "<event",
                    &format!("<event hostile_detected=\"true\" keywords=\"{}\"", keywords_str)
                )
            } else {
                // If no <event> tag found, wrap the message
                format!(
                    "<!-- HOSTILE CONTENT DETECTED: {} -->\n{}",
                    keywords_str, cot_xml
                )
            };

            Ok(modified)
        } else {
            // Message is clean, pass through unchanged
            host::log("ExampleFilterPlugin: Message passed filter");
            Ok(cot_xml)
        }
    }

    /// Get the plugin name
    fn get_name() -> String {
        "Example Filter Plugin".to_string()
    }

    /// Get the plugin version
    fn get_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Get the plugin description
    fn get_description() -> String {
        "A simple example filter that detects hostile keywords in CoT messages".to_string()
    }
}

// Note: Unit tests are not included because they require a WASM runtime
// with host function implementations. Testing should be done through
// integration tests with the actual host runtime.
