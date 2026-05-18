//! Layout and styling constants for the ER diagram renderer.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for entity names and attribute text (px).
pub const FONT_SIZE: f64 = 16.0;

/// ab_glyph Arial measures ~1.117x narrower than the browser; apply this factor to match.
#[allow(dead_code)]
pub const TEXT_SCALE: f64 = 1.1172;

// ---------------------------------------------------------------------------
// Column padding
// ---------------------------------------------------------------------------

/// Padding on each side of a column's text (left and right of each column, px).
/// Measured from reference SVGs: each column = text_width + 2 × COL_PAD_X.
pub const COL_PAD_X: f64 = 12.5;

/// Padding on each side of the entity name in the header (no-attr entities, px).
/// Measured from reference SVGs: no-attr entity width = label_width + 2 × HEADER_PAD_X.
pub const HEADER_PAD_X: f64 = 20.0;

// ---------------------------------------------------------------------------
// Entity geometry
// ---------------------------------------------------------------------------

/// Height of each attribute row (px).  Measured from reference SVGs: 42.75px consistently.
pub const ATTR_ROW_H: f64 = 42.75;

/// Height of the entity header row (same as attribute row height, px).
pub const HEADER_ROW_H: f64 = 42.75;

/// Height for entities with no attributes (px).  Matches Mermaid reference: 84px.
pub const NO_ATTR_ENTITY_H: f64 = 84.0;

/// Minimum entity width — minEntityWidth config default (px).
pub const MIN_ENTITY_W: f64 = 100.0;

// ---------------------------------------------------------------------------
// Dagre layout parameters
// ---------------------------------------------------------------------------

/// Node separation used by dagre.  Matches Mermaid erRenderer defaults (px).
pub const NODE_SEP: f64 = 140.0;

/// Edge separation used by dagre (px).
pub const EDGE_SEP: f64 = 100.0;

/// Rank separation used by dagre (px).  Calibrated to match reference node spacing.
pub const RANK_SEP: f64 = 101.0;

/// Graph margin on x axis (px).
pub const MARGIN_X: f64 = 8.0;

/// Graph margin on y axis (px).
pub const MARGIN_Y: f64 = 8.0;

// ---------------------------------------------------------------------------
// Self-loop dummy nodes
// ---------------------------------------------------------------------------

/// Width of dummy waypoint nodes used for self-loop routing (px).
pub const SELF_LOOP_DUMMY_W: f64 = 84.0;

// ---------------------------------------------------------------------------
// Colours
// ---------------------------------------------------------------------------

/// Fill colour for entity boxes.
pub const ENTITY_FILL: &str = "#ECECFF";

/// Stroke colour for entity box borders.
pub const ENTITY_STROKE: &str = "#9370DB";

/// Colour of relationship lines.
pub const REL_LINE_COLOR: &str = "#333333";

/// Font colour for text inside entity boxes.
pub const FONT_COLOR: &str = "#333";

/// Fill colour for odd attribute rows.
pub const ATTR_ROW_ODD: &str = "hsl(240, 100%, 100%)";

/// Fill colour for even attribute rows.
pub const ATTR_ROW_EVEN: &str = "hsl(240, 100%, 97.2745098039%)";

// ---------------------------------------------------------------------------
// ForeignObject label geometry
// ---------------------------------------------------------------------------

/// Height of foreignObject label blocks (px).
pub const FO_HEIGHT: f64 = 24.0;

/// Height of relationship label foreignObject blocks (px).
pub const REL_LABEL_FO_H: f64 = 21.0;
