#![allow(missing_docs)]
#![allow(clippy::all)]

// Generate Wasmtime bindings from WIT interface definition
// The bindings provide type-safe access to the plugin interface
wasmtime::component::bindgen!({
    path: "wit/omnitak-plugins.wit",
    imports: { default: trappable },
});
