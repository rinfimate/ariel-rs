//! Layout and styling constants for the class diagram renderer.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for class member/method/annotation text — matches Mermaid CSS `g.classGroup text { font-size: 10px }`.
pub const FONT_SIZE: f64 = 10.0;

/// Font size for the class name row — matches Mermaid CSS `.classTitleText { font-size: 18px }`.
pub const TITLE_FONT_SIZE: f64 = 18.0;

// ---------------------------------------------------------------------------
// Node geometry
// ---------------------------------------------------------------------------

/// Height per member or method row. Calibrated from ref output: Animal-Dog
/// row-diff = 20px (FONT_SIZE * 1.1 + 9 = 20). Matches Mermaid's lineHeight=1.1.
pub const MEMBER_ROW_H: f64 = FONT_SIZE * 1.1 + 9.0; // 11 + 9 = 20

/// Height of the class header section. Calibrated from ref class_basic Dog: dividers at
/// y=±20.5 with hh=61.5 → header_h = -20.5 − (−61.5) = 41.
pub const HEADER_H: f64 = 41.0;

/// Height per annotation row: same line-height as member rows.
pub const ANNOTATION_H: f64 = FONT_SIZE * 1.1 + 9.0; // 11 + 9 = 20

/// Horizontal padding inside the class box on each side (px).
pub const H_PAD: f64 = 12.0;

/// Minimum class box width — Mermaid has no minimum; bbox drives the width.
pub const MIN_BOX_W: f64 = 0.0;

/// Minimum height for an empty members or methods section (Mermaid uses 18px).
pub const EMPTY_SECTION_H: f64 = 18.0;

// ---------------------------------------------------------------------------
// Text scaling
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Mermaid-canonical colors (not theme-derived)
// ---------------------------------------------------------------------------

/// Namespace cluster background fill (Mermaid `class-diagram-v2.scss`).
pub const NAMESPACE_FILL: &str = "#ffffde";

/// Namespace cluster border stroke (shared with notes).
pub const NAMESPACE_STROKE: &str = "#aaaa33";

/// Note background fill (Mermaid `class-diagram-v2.scss` `.note { fill }`).
pub const NOTE_FILL: &str = "#fff5ad";

/// Note border stroke (same as namespace).
pub const NOTE_STROKE: &str = "#aaaa33";

/// Drop-shadow flood colour applied to outer-class shadow filters.
pub const SHADOW_FLOOD_COLOR: &str = "#000000";
