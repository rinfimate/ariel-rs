//! Layout and styling constants for the flowchart renderer.

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Default font size for node labels (px).
pub const FONT_SIZE: f64 = 16.0;

/// TEXT_SCALE: browser renders Arial 16px ~11.7% wider than ab_glyph raw metrics.
/// Now using monospace (Courier New) metrics via measure_mono.
#[allow(dead_code)]
pub const TEXT_SCALE: f64 = 1.117;

// ---------------------------------------------------------------------------
// Node geometry
// ---------------------------------------------------------------------------

/// Height of a standard rectangular node (px).
pub const RECT_H: f64 = 54.0;

/// Height of compact nodes: Stadium, Subroutine, Asymmetric (px).
pub const COMPACT_H: f64 = 39.0;

/// Horizontal padding (each side) for Rectangle / Default nodes (px).
pub const H_PAD: f64 = 30.0;

/// Horizontal padding (each side) for RoundedRect and Stadium nodes (px).
pub const SMALL_PAD: f64 = 15.0;

/// Horizontal padding (each side) for Diamond and Hexagon nodes (px).
pub const DIAMOND_PAD: f64 = 27.0;

/// Total horizontal padding for Subroutine nodes (inner pad + line offsets, px).
/// = inner_pad(7.5 each side) * 2 + line_offset(8 each side) * 2 = 15 + 16 = 31.
pub const SUBROUTINE_H_PAD: f64 = 31.0;

/// Inset from the left/right edge for the decorative vertical lines on Subroutine nodes (px).
pub const SUBROUTINE_LINE_INSET: f64 = 6.0;

/// Total horizontal padding for Asymmetric nodes (base pad, px).
/// Matches Mermaid reference: total extra = 15px (same as SMALL_PAD).
pub const ASYMMETRIC_BASE_PAD: f64 = 15.0;

/// Depth of the V-notch on the left side of an Asymmetric node (px).
#[allow(dead_code)]
pub const ASYMMETRIC_NOTCH_DEPTH: f64 = 10.0;

/// Minimum radius for a Circle node (px).
pub const CIRCLE_MIN_RADIUS: f64 = 27.0;

/// Extra padding added to the text half-width to compute the Circle radius (px).
pub const CIRCLE_LABEL_PAD: f64 = 7.5;

/// Horizontal padding for Cylinder nodes (px).
/// Matches Mermaid reference: cylinder width = label_w + 15 (total, 7.5 each side).
pub const CYLINDER_H_PAD: f64 = 15.0;

/// Fraction of the half-width used as the ellipse ry for Cylinder nodes.
/// Calibrated to 0.2398: for half-width 41.75 gives ry=10.012, matching the reference.
pub const CYLINDER_RY_FACTOR: f64 = 0.2398;

/// Minimum ellipse ry for Cylinder nodes (px).
pub const CYLINDER_MIN_RY: f64 = 7.0;

/// Body height of a Cylinder node, excluding the top/bottom ellipses (px).
pub const CYLINDER_BODY_H: f64 = 49.0;

// ---------------------------------------------------------------------------
// Label geometry
// ---------------------------------------------------------------------------

/// Height of a `<foreignObject>` label box (px).
pub const LABEL_FO_HEIGHT: f64 = 24.0;

/// Y-offset applied to the label `<g>` element (= -LABEL_FO_HEIGHT / 2, px).
pub const LABEL_Y_OFFSET: i32 = -12;

#[allow(dead_code)]
/// Y-offset for Cylinder node labels, which sit lower to centre in the body (px).
pub const CYLINDER_LABEL_Y_OFFSET: i32 = -2;

/// Y baseline for plain SVG `<text>` node labels, relative to the group origin (px).
pub const TEXT_LABEL_Y: i32 = 5;

/// Y offset added to the cluster-rect top to position the plain-text cluster label (px).
pub const CLUSTER_LABEL_TEXT_DY: f64 = 14.0;

// ---------------------------------------------------------------------------
// Layout / dagre parameters
// ---------------------------------------------------------------------------

/// Graph margin on each axis (marginx = marginy, px).
pub const GRAPH_MARGIN: f64 = 8.0;

/// 2 × GRAPH_MARGIN: added to a subgraph's cluster-rect size to get the full group size.
pub const SG_LAYOUT_MARGIN: f64 = 16.0;

/// Node separation used by dagre for all layout levels (px).
pub const NODE_SEP: f64 = 50.0;

/// Rank separation for the outermost dagre layout (px).
pub const OUTER_RANKSEP: f64 = 50.0;

/// Amount added to the ranksep at each recursive subgraph layout level (px).
pub const RANKSEP_INCREMENT: f64 = 25.0;

// ---------------------------------------------------------------------------
// Edge geometry
// ---------------------------------------------------------------------------

/// How far (px) to trim the path end so the arrowhead tip lands on the node boundary.
/// pointEnd: viewBox 10, refX 5, tip at x=10, markerWidth 8 → (10-5)*8/10 = 4 px.
pub const POINT_END_TRIM: f64 = 4.0;

/// How far (px) to trim the path start for a reverse arrow.
/// pointStart: viewBox 10, refX 4.5, tip at x=0, markerWidth 8 → 4.5*8/10 = 3.6 px.
#[allow(dead_code)]
pub const POINT_START_TRIM: f64 = 3.6;

// ---------------------------------------------------------------------------
// Subgraph padding (legacy — kept for compute_subgraph_bbox)
// ---------------------------------------------------------------------------

/// Horizontal padding around a subgraph bounding box (px).
#[allow(dead_code)]
pub const SG_PAD_H: f64 = 10.0;

/// Top padding above the subgraph bounding box (px).
#[allow(dead_code)]
pub const SG_PAD_T: f64 = 24.0;

/// Bottom padding below the subgraph bounding box (px).
#[allow(dead_code)]
pub const SG_PAD_B: f64 = 10.0;
