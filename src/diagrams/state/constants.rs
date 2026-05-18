//! Layout and styling constants for the state diagram renderer.

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Default font size for all state diagram text (px).
pub const FONT_SIZE: f64 = 16.0;

/// Text-width scale factor: browser Arial metrics are ~11.7% wider than ab_glyph raw.
pub const LABEL_SCALE: f64 = 1.117;

// ---------------------------------------------------------------------------
// Node geometry
// ---------------------------------------------------------------------------

/// Height of a standard (Normal) state node rectangle (px).
pub const NODE_H: f64 = 40.0;

/// Horizontal padding on each side of a state node label (px).
pub const H_PAD: f64 = 8.0;

/// Radius of the start-state filled circle (px).
pub const START_R: f64 = 7.0;

/// Radius of the end-state outer circle (px).
pub const END_R: f64 = 7.0;

/// Radius of the end-state inner filled dot (px).
pub const END_INNER_R: f64 = 2.5;

/// Width of a fork/join horizontal bar (px).
pub const FORK_W: f64 = 70.0;

/// Height of a fork/join horizontal bar (px).
pub const FORK_H: f64 = 10.0;

/// Half-size (radius) of a choice (diamond) node (px).
pub const CHOICE_SIZE: f64 = 14.0;

// ---------------------------------------------------------------------------
// Composite (cluster) geometry
// ---------------------------------------------------------------------------

/// Height reserved at the top of a composite state for its title label (px).
pub const CLUSTER_TITLE_H: f64 = 34.0;

/// Padding applied around the cluster rect boundary (px).
pub const CLUSTER_PAD: f64 = 8.0;

// ---------------------------------------------------------------------------
// Layout / dagre parameters
// ---------------------------------------------------------------------------

/// Node separation used by dagre for all layout levels (px).
pub const NODESEP: f64 = 50.0;

/// Rank separation for the outer dagre layout (px).
pub const RANKSEP: f64 = 50.0;

/// Graph margin applied on each axis (px).
pub const MARGIN: f64 = 8.0;

/// Rank separation for inner (composite) dagre sub-graphs (px).
pub const INNER_RANKSEP: f64 = 75.0;

/// Y-axis margin for inner (composite) dagre sub-graphs (px).
pub const INNER_MARGINY: f64 = 45.5;

/// X-axis margin for inner (composite) dagre sub-graphs (px).
pub const INNER_MARGINX: f64 = 44.0;

// ---------------------------------------------------------------------------
// SVG identifiers
// ---------------------------------------------------------------------------

/// Fixed id attribute for the state diagram SVG root element.
pub const SVG_ID: &str = "mermaid-svg";
