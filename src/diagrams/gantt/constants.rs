//! Layout and styling constants for the Gantt chart renderer.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// SVG canvas
// ---------------------------------------------------------------------------

/// Total SVG width (Mermaid default, px).
pub const SVG_WIDTH: f64 = 1984.0;

/// Space reserved on the left for section labels (px).
pub const LEFT_PAD: f64 = 75.0;

/// Right margin (px).
pub const RIGHT_PAD: f64 = 75.0;

/// Drawable chart width = SVG_WIDTH − LEFT_PAD − RIGHT_PAD (px).
pub const DRAW_WIDTH: f64 = SVG_WIDTH - LEFT_PAD - RIGHT_PAD; // = 1834.0

// ---------------------------------------------------------------------------
// Chart geometry
// ---------------------------------------------------------------------------

/// Y position of the diagram title text (px).
pub const TITLE_TOP: f64 = 25.0;

/// Y where the first task band starts, after title + axis label area (px).
/// Mermaid reference SVG places the grid at y=98 = 48 + 2*24 + 2 (not 50 as defaultConfig says).
pub const CHART_TOP: f64 = 48.0;

/// Height of each task row (px).
pub const ROW_HEIGHT: f64 = 24.0;

/// Height of the task bar rectangle within each row (px).
pub const BAR_HEIGHT: f64 = 20.0;

/// Top offset of the bar within its row — centres the bar vertically (px).
pub const BAR_OFFSET: f64 = 2.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for task bar label text (px).
pub const FONT_SIZE: f64 = 11.0;

/// Font size for section title labels (px).
pub const SECTION_FONT_SIZE: f64 = 11.0;

/// Font size for the diagram title (px).
#[allow(dead_code)]
pub const TITLE_FONT_SIZE: f64 = 18.0;

/// Font size for the x-axis tick labels (px).
pub const AXIS_FONT_SIZE: f64 = 10.0;

// ---------------------------------------------------------------------------
// Grid
// ---------------------------------------------------------------------------

/// Space below the grid baseline for axis tick labels (px).
pub const GRID_BOTTOM_PAD: f64 = 25.0;

/// D3 axis adds 2px padding before the tick domain line (px).
pub const GRID_AXIS_OFFSET: f64 = 2.0;

// ---------------------------------------------------------------------------
// Exclude-range shading
// ---------------------------------------------------------------------------

/// Y position where weekend/exclusion shading starts (= TITLE_TOP + 10, px).
pub const EXCL_TOP: f64 = TITLE_TOP + 10.0; // = 35.0

// Section fills and exclude fill are now in ThemeVars:
//   vars.gantt_section_fill0, vars.gantt_section_fill1, vars.gantt_exclude_fill

// ---------------------------------------------------------------------------
// Band width
// ---------------------------------------------------------------------------

/// Width of section background bands = SVG_WIDTH − RIGHT_PAD / 2 (px).
pub const BAND_WIDTH: f64 = SVG_WIDTH - RIGHT_PAD / 2.0; // = 1946.5
