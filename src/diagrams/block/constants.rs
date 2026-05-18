//! Layout and styling constants for the block diagram renderer.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Node geometry
// ---------------------------------------------------------------------------

/// Node height matching Mermaid block diagram defaults: 2em at 16px (px).
pub const NODE_H: f64 = 32.0;

/// Horizontal padding per side inside a node (px).
pub const H_PAD: f64 = 4.0;

/// Horizontal gap between nodes (px).
pub const H_GAP: f64 = 8.0;

/// Vertical gap between rows (px).
pub const V_GAP: f64 = 8.0;

/// Outer margin around all nodes (px).
pub const MARGIN: f64 = 5.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size matching Mermaid block diagram default (px).
pub const FONT_SIZE: f64 = 16.0;

/// ab_glyph Arial measures ~11.7% narrower than browser; apply this factor to match.
pub const TEXT_SCALE: f64 = 1.117;

// ---------------------------------------------------------------------------
// Cylinder shape
// ---------------------------------------------------------------------------

/// Cylinder ellipse ry for block shapes (px).
pub const CYLINDER_RY: f64 = 7.0;

/// Fill color for cylinder shapes.
pub const CYLINDER_FILL: &str = "#ECECFF";

/// Stroke color for cylinder shapes.
pub const CYLINDER_STROKE: &str = "#9370DB";

// ---------------------------------------------------------------------------
// Edge trim
// ---------------------------------------------------------------------------

/// Pixels to trim before target left edge so arrowhead tip touches the block.
/// block-pointEnd: viewBox=10, refX=6, markerWidth=12 → overhang=(10-6)*1.2=4.8px;
/// empirically 4px trim produces 0.8px overlap matching the reference.
pub const EDGE_END_TRIM: f64 = 4.0;
