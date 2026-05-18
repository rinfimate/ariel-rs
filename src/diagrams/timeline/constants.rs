//! Layout and styling constants for the timeline renderer.

// ---------------------------------------------------------------------------
// Layout geometry
// ---------------------------------------------------------------------------

/// Left margin for the timeline canvas (conf.timeline.leftMargin = 150, px).
pub const LEFT_MARGIN: f64 = 150.0;

/// Padding added around the viewBox by setupGraphViewbox (px).
pub const VIEWBOX_PADDING: f64 = 50.0;

/// Width of a node before padding is applied (px).
pub const NODE_WIDTH: f64 = 150.0;

/// Padding applied on each side of a node (px).
pub const NODE_PADDING: f64 = 20.0;

/// Rendered width of a node including padding on both sides (px).
/// Equals NODE_WIDTH + 2 × NODE_PADDING = 190.
pub const RENDERED_WIDTH: f64 = NODE_WIDTH + 2.0 * NODE_PADDING;

/// Horizontal step between consecutive tasks (masterX increment, px).
pub const TASK_STEP: f64 = 200.0;

/// Initial Y coordinate for sections (px).
pub const SECTION_START_Y: f64 = 50.0;

/// Initial X coordinate for the master layout cursor (px).
/// Equal to 50 + LEFT_MARGIN.
pub const MASTER_START_X: f64 = 50.0 + LEFT_MARGIN;

/// Initial Y coordinate for the master layout cursor (px).
pub const MASTER_START_Y: f64 = 50.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Default font size for timeline nodes and labels (px).
pub const FONT_SIZE: f64 = 16.0;

// ---------------------------------------------------------------------------
// Node path geometry
// ---------------------------------------------------------------------------

/// Corner radius for rounded-rectangle node background paths (px).
pub const NODE_CORNER_R: f64 = 5.0;

// ---------------------------------------------------------------------------
// SVG identifiers
// ---------------------------------------------------------------------------

/// Fixed id attribute for the timeline diagram SVG root element.
pub const DIAGRAM_ID: &str = "mermaid-timeline";
