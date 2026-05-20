//! Constants for the info diagram renderer.
//! Source: src/diagrams/info/infoRenderer.ts

/// SVG width (px) — configureSvgSize(svg, 100, 400, true)
pub const SVG_WIDTH: f64 = 400.0;

/// SVG height (px)
pub const SVG_HEIGHT: f64 = 100.0;

/// Text x position
pub const TEXT_X: f64 = 100.0;

/// Text y position
pub const TEXT_Y: f64 = 40.0;

/// Font size for version label
pub const FONT_SIZE: u32 = 32;

/// The Mermaid version this renderer targets.
pub const MERMAID_VERSION: &str = "11.15.0";

/// ariel-rs version (from Cargo.toml, injected at compile time).
pub const ARIEL_VERSION: &str = env!("CARGO_PKG_VERSION");
