//! Layout and styling constants for the block diagram renderer.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Node geometry
// ---------------------------------------------------------------------------

/// Node height matching Mermaid block default. Empirical from ref/block_basic.svg:
/// single-line text "A" at 16px renders to 25×20 (FONT_SIZE*1.1 + ~7.4 padding).
pub const NODE_H: f64 = 25.0;

/// Horizontal padding per side inside a node (px).
pub const H_PAD: f64 = 4.0;

/// Horizontal gap between nodes (px).
pub const H_GAP: f64 = 8.0;

/// Vertical gap between rows (px).
pub const V_GAP: f64 = 8.0;

/// Outer margin around all nodes for viewBox calculation (px).
/// defaultConfig.block.padding = 8 but the block renderer applies 5 to the
/// viewBox margin — confirmed from Mermaid JS ref SVG (viewBox x=-5, y=-(NODE_H/2+5)).
pub const MARGIN: f64 = 5.0;

/// Padding inside a group node (between border and children), px.
pub const GROUP_PAD: f64 = 8.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size matching Mermaid block diagram default (px).
pub const FONT_SIZE: f64 = 16.0;

// ---------------------------------------------------------------------------
// Cylinder shape
// ---------------------------------------------------------------------------

/// Cylinder ellipse ry for block shapes (px).
pub const CYLINDER_RY: f64 = 7.0;

// CYLINDER_FILL and CYLINDER_STROKE removed — use vars.primary_color / vars.primary_border at render time.

// ---------------------------------------------------------------------------
// Edge trim
// ---------------------------------------------------------------------------

/// Pixels to trim before target left edge so arrowhead tip touches the block.
/// block-pointEnd: viewBox=10, refX=6, markerWidth=12 → overhang=(10-6)*1.2=4.8px;
/// empirically 4px trim produces 0.8px overlap matching the reference.
pub const EDGE_END_TRIM: f64 = 4.0;
