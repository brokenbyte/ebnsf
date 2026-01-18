//! WebAssembly bindings for ebnsf
//!
//! This module provides WASM-compatible functions to generate railroad diagrams
//! from EBNF specifications.

use wasm_bindgen::prelude::*;

use crate::winnow::v4::parse_ebnf;

/// Returns version information about the WASM module
#[wasm_bindgen]
pub fn version_info() -> String {
    format!("ebnsf v{}", env!("CARGO_PKG_VERSION"))
}

/// Parses an EBNF grammar and returns an SVG railroad diagram
///
/// # Arguments
/// * `src` - The EBNF grammar source text
///
/// # Returns
/// An HTML string containing either the SVG diagram or an error message
#[wasm_bindgen]
pub fn ebnf_to_svg(src: &str) -> String {
    match parse_ebnf(src) {
        Ok(diagram) => {
            let svg = diagram.to_string();
            format!(
                r#"<div style="width: auto; height: auto; max-height: 100%; max-width: 100%; overflow: auto;">
{}
</div>"#,
                svg
            )
        }
        Err(e) => {
            format!(
                r#"<div class="error" style="color: red; font-family: monospace; white-space: pre-wrap;">
Parse error:
{}
</div>"#,
                e
            )
        }
    }
}
