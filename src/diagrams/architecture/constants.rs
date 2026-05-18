//! Layout and styling constants for the architecture renderer.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Node geometry
// ---------------------------------------------------------------------------

/// Service icon width and height in px.
pub const ICON_SIZE: f64 = 80.0;

/// Font size for node labels from Mermaid's default CSS (px).
pub const LABEL_FONT_SIZE: f64 = 16.0;

// ---------------------------------------------------------------------------
// Layout / step distances
// ---------------------------------------------------------------------------

/// Base horizontal centre-to-centre gap between free (ungrouped) nodes (px).
pub const H_STEP: f64 = 200.68;

/// Vertical centre-to-centre gap; also used for same-group vertical edges (px).
pub const V_STEP: f64 = 201.08;

/// Extra horizontal step added when an edge crosses a group boundary (px).
/// Derived from reference: cross-group H_STEP = 215.94, free H_STEP = 200.68, extra = 15.26.
pub const H_STEP_CROSS_EXTRA: f64 = 15.26;

/// Extra space from icon bottom to label text baseline + descender (px).
pub const LABEL_SPACE: f64 = 23.5;

/// Extra space from icon bottom to label baseline used inside group bbox calculation (px).
pub const LABEL_SPACE_IN_GROUP: f64 = 16.0;

// ---------------------------------------------------------------------------
// Group / viewBox geometry
// ---------------------------------------------------------------------------

/// Padding inside a group boundary around child node bboxes (px).
pub const GROUP_PAD: f64 = 42.5;

/// Outer margin around the whole diagram (viewBox padding on each side, px).
pub const OUTER_MARGIN: f64 = 40.0;
