//! Layout and colour constants for the venn diagram renderer.

// ---------------------------------------------------------------------------
// Canvas geometry
// ---------------------------------------------------------------------------

/// Default SVG canvas width (px).
pub const SVG_WIDTH: f64 = 800.0;

/// Default SVG canvas height (px).
pub const SVG_HEIGHT: f64 = 450.0;

/// Height reserved at the top of the canvas for the diagram title (px).
pub const TITLE_HEIGHT: f64 = 48.0;

/// Scale factor for font sizes and stroke widths, derived from the ratio
/// of SVG_WIDTH to a 1600px reference width (mirrors vennRenderer.ts).
pub const SCALE: f64 = SVG_WIDTH / 1600.0;

// ---------------------------------------------------------------------------
// Circle layout
// ---------------------------------------------------------------------------

/// Overlap separation factor for 2-set layout (gap = r × this factor, ~20% overlap).
pub const TWO_SET_SEP_FACTOR: f64 = 1.1;

/// Radius scale factor for 3-set layout (radius = base_r × this).
pub const THREE_SET_R_FACTOR: f64 = 0.80;

/// Distance factor for 3-set layout (dist = r × this).
pub const THREE_SET_DIST_FACTOR: f64 = 1.05;

// ---------------------------------------------------------------------------
// Colour palette
// ---------------------------------------------------------------------------

/// Venn diagram set colour palette (venn1–venn8 from the default Mermaid theme).
pub const VENN_COLORS: &[&str] = &[
    "#4C8BF5", "#FF6B6B", "#48C9B0", "#F5A623", "#9B59B6", "#3498DB", "#E67E22", "#1ABC9C",
];

// ---------------------------------------------------------------------------
// SVG identifiers
// ---------------------------------------------------------------------------

/// Fixed id attribute for the venn diagram SVG root element.
pub const SVG_ID: &str = "mermaid-venn";
