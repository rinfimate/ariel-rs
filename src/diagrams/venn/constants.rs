//! Layout and colour constants for the venn diagram renderer.

// ---------------------------------------------------------------------------
// Canvas geometry
// ---------------------------------------------------------------------------

/// Default SVG canvas width (px).
pub const SVG_WIDTH: f64 = 800.0;

/// Default SVG canvas height (px).
pub const SVG_HEIGHT: f64 = 450.0;

/// Reference canvas width used for scale calculations (mirrors vennRenderer.ts).
pub const REFERENCE_WIDTH: f64 = 1600.0;

// ---------------------------------------------------------------------------
// SVG identifiers
// ---------------------------------------------------------------------------

/// Fixed id attribute for the venn diagram SVG root element.
pub const SVG_ID: &str = "mermaid-venn";

// ---------------------------------------------------------------------------
// Numeric tolerances
// ---------------------------------------------------------------------------

/// Epsilon for near-zero overlap comparison in circle intersection math.
pub const SMALL: f64 = 1e-10;
