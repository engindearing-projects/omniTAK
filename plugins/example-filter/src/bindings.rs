// WIT bindings generation
//
// This module generates the Rust bindings from the WIT interface definition.
// The bindings allow the plugin to interact with the host and implement the
// required guest interface.

wit_bindgen::generate!({
    path: "wit/message-filter.wit",
    default_bindings_module: "crate::bindings",
    pub_export_macro: true,
});
