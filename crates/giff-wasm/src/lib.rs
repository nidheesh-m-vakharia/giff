// crates/giff-wasm/src/lib.rs
use giff_core::StackStore;
use wasm_bindgen::prelude::*;

/// Parse a stacked.toml string and return the stack names as a JSON array.
/// This is the WASM entry point for the future web UI.
#[wasm_bindgen]
pub fn parse_stack_store(toml: &str) -> Result<String, JsValue> {
    let store = StackStore::from_toml(toml).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let names: Vec<&str> = store.stacks.iter().map(|s| s.name.as_str()).collect();
    Ok(serde_json::to_string(&names).unwrap())
}
