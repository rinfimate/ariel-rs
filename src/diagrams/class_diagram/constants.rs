//! Layout and styling constants for the class diagram renderer.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for class box text — foreignObjects inherit SVG root font-size:16px (px).
pub const FONT_SIZE: f64 = 16.0;

/// Font size for the class name row (bold, px).
pub const TITLE_FONT_SIZE: f64 = 16.0;

// ---------------------------------------------------------------------------
// Node geometry
// ---------------------------------------------------------------------------

/// Height per member or method row in a class box (px).
pub const MEMBER_ROW_H: f64 = 24.0;

/// Height of the class header section (annotation label + padding, px).
pub const HEADER_H: f64 = 48.0;

/// Height per annotation row (px).
pub const ANNOTATION_H: f64 = 24.0;

/// Horizontal padding inside the class box on each side (px).
pub const H_PAD: f64 = 12.0;

/// Minimum class box width — Mermaid has no minimum; bbox drives the width.
pub const MIN_BOX_W: f64 = 0.0;

/// Minimum height for an empty members or methods section (Mermaid uses 18px).
pub const EMPTY_SECTION_H: f64 = 18.0;

// ---------------------------------------------------------------------------
// Text scaling
// ---------------------------------------------------------------------------

/// Browser renders foreignObject HTML content at ~1.117× our ab_glyph metric for regular text.
/// Empirically derived by comparing reference foreignObject widths with ab_glyph measurements.
pub const CONTENT_SCALE: f64 = 1.117;

/// Browser renders bold class-name foreignObject at ~1.207× our ab_glyph metric.
/// Empirically calibrated across all class diagrams; 1.207 minimises the weighted RMS.
pub const NAME_SCALE: f64 = 1.207;

// ---------------------------------------------------------------------------
// Layout / dagre parameters
// ---------------------------------------------------------------------------

/// Node separation used by dagre (px).
pub const NODE_SEP: f64 = 50.0;

/// Rank separation used by dagre (px).
pub const RANK_SEP: f64 = 50.0;

/// Graph margin on x axis (px).
pub const MARGIN_X: f64 = 8.0;

/// Graph margin on y axis (px).
pub const MARGIN_Y: f64 = 8.0;

// ---------------------------------------------------------------------------
// Edge trim distances
// ---------------------------------------------------------------------------

/// Arrowhead overhang for Extension, Composition, and Aggregation end markers (px).
pub const TRIM_DIAMOND_TRIANGLE: f64 = 17.0;

/// Arrowhead overhang for Arrow (dependency) end markers (px).
pub const TRIM_ARROW: f64 = 8.0;

// ---------------------------------------------------------------------------
// Terminal label sizing
// ---------------------------------------------------------------------------

/// Terminal label marker size used in Mermaid (px).
pub const TERMINAL_MARKER_SIZE: f64 = 10.0;

/// Scale factor for terminal label text (11px CSS class `.edgeTerminals`).
pub const TERMINAL_SCALE: f64 = 1.117;
