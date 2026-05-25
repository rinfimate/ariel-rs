//! Layout and styling constants for the requirement renderer.

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size used throughout the requirement diagram (px).
/// defaultConfig says 14 but Mermaid renders at the global CSS font-size (16px effective).
pub const FONT_SIZE: f64 = 16.0;

// ---------------------------------------------------------------------------
// Box geometry
// ---------------------------------------------------------------------------

/// Horizontal padding inside each requirement/element box (px).
pub const PAD_X: f64 = 10.0;

/// Row height (vertical spacing between body items inside a box, px).
/// Calibrated from ref: test_req body item spacing = 23 (FONT_SIZE*1.1 + 5.4 padding).
pub const ROW_H: f64 = 23.0;

/// Height of the header section inside each node box (px).
/// Calibrated from ref divider position: test_req divider at y=-20 with hh=86 → HEADER=66.
pub const HEADER_H: f64 = 66.0;

/// First body row center offset from divider line (px).
/// Calibrated from ref: divider at -20, first body row center at -11.5 → 8.5.
pub const BODY_FIRST_ROW_OFFSET: f64 = 8.5;

/// Body bottom padding constant: last_row_center to box bottom = ROW_H/2 + this.
/// Calibrated from ref: test_req last row at 57.5, box bottom 86 → 28.5 = ROW_H/2 + 17.
pub const BODY_BOTTOM_PAD: f64 = 17.0;

/// Minimum box width (px). Natural sizing — Mermaid's rect_min_width=200 is overridden by theme.
#[allow(dead_code)]
pub const MIN_WIDTH: f64 = 0.0;
/// Minimum box height (px). Natural sizing — Mermaid's rect_min_height=200 is overridden by theme.
#[allow(dead_code)]
pub const MIN_HEIGHT: f64 = 0.0;

// ---------------------------------------------------------------------------
// Dagre layout parameters
// ---------------------------------------------------------------------------

/// Node separation used by dagre (px).
pub const NODE_SEP: f64 = 50.0;

/// Rank separation used by dagre (px). Dagre adds edge-label height (18) giving total gap.
/// Calibrated: ref test_req center y=268 with test_entity y=59.5 → gap 208.5 → ranksep=53.
pub const RANK_SEP: f64 = 53.0;

/// Horizontal margin around the dagre graph (px).
pub const MARGIN_X: f64 = 8.0;

/// Vertical margin around the dagre graph (px).
pub const MARGIN_Y: f64 = 8.0;

// ---------------------------------------------------------------------------
// Colours — resolved at render time from ThemeVars
// ---------------------------------------------------------------------------
// box_fill      = vars.primary_color   (#ECECFF default / #1f2020 dark)
// box_stroke    = vars.primary_border  (#9370DB default / #81B1DB dark)
// font_color    = vars.primary_text    (#333333 default / #cde498 dark)
// line_color    = vars.line_color      (#333333 default / #81B1DB dark)

// ---------------------------------------------------------------------------
// Text measurement
// ---------------------------------------------------------------------------

/// Safety margin added to text width measurements to prevent last-letter clipping (px).
/// Set to 0: browser-measured character widths already account for glyph advance, so no
/// extra margin is needed (adding 6px caused boxes to be ~6px wider than the reference).
pub const TEXT_SAFETY_MARGIN: f64 = 0.0;
