//! Default configuration constants for the xychart renderer.

// ---------------------------------------------------------------------------
// Canvas geometry
// ---------------------------------------------------------------------------

/// Default SVG canvas width (from config.schema.yaml, px).
pub const WIDTH: f64 = 700.0;

/// Default SVG canvas height (from config.schema.yaml, px).
pub const HEIGHT: f64 = 500.0;

// ---------------------------------------------------------------------------
// Chart title
// ---------------------------------------------------------------------------

/// Font size for the chart title (px).
pub const TITLE_FONT_SIZE: f64 = 20.0;

/// Padding above and below the chart title (px).
pub const TITLE_PADDING: f64 = 10.0;

// ---------------------------------------------------------------------------
// Plot area
// ---------------------------------------------------------------------------

/// Percentage of the available space reserved for the plot area.
pub const PLOT_RESERVED_SPACE_PERCENT: f64 = 50.0;

// ---------------------------------------------------------------------------
// Axis — general flags and sizes
// ---------------------------------------------------------------------------

/// Whether to show axis tick labels by default.
pub const AXIS_SHOW_LABEL: bool = true;

/// Font size for axis tick labels (px).
pub const AXIS_LABEL_FONT_SIZE: f64 = 14.0;

/// Padding between an axis label and the adjacent tick/line (px).
pub const AXIS_LABEL_PADDING: f64 = 5.0;

/// Whether to show axis titles by default.
pub const AXIS_SHOW_TITLE: bool = true;

/// Font size for axis titles (px).
pub const AXIS_TITLE_FONT_SIZE: f64 = 16.0;

/// Padding around axis titles (px).
pub const AXIS_TITLE_PADDING: f64 = 5.0;

/// Whether to show axis tick marks by default.
pub const AXIS_SHOW_TICK: bool = true;

/// Length of each tick mark (px).
pub const AXIS_TICK_LENGTH: f64 = 5.0;

/// Stroke width of each tick mark (px).
pub const AXIS_TICK_WIDTH: f64 = 2.0;

/// Whether to show the axis line by default.
pub const AXIS_SHOW_AXIS_LINE: bool = true;

/// Stroke width of the axis line (px).
pub const AXIS_LINE_WIDTH: f64 = 2.0;

// ---------------------------------------------------------------------------
// Bar plot geometry
// ---------------------------------------------------------------------------

/// Ratio of bar width to tick distance used to compute bar outer padding.
pub const BAR_WIDTH_TO_TICK_WIDTH_RATIO: f64 = 0.7;

/// Maximum fraction of the available span used for outer padding (for labels).
pub const MAX_OUTER_PADDING_PERCENT_FOR_WRT_LABEL: f64 = 0.2;

/// Fraction of tick distance kept as padding between adjacent bars.
pub const BAR_PADDING_PERCENT: f64 = 0.05;

// ---------------------------------------------------------------------------
// Theme colours (Mermaid default theme xyChart)
// ---------------------------------------------------------------------------

/// Background fill colour for the chart canvas.
pub const BG_COLOR: &str = "white";

/// Fill colour for the chart title text.
pub const TITLE_COLOR: &str = "#131300";

/// Fill colour for x-axis tick labels.
pub const X_AXIS_LABEL_COLOR: &str = "#131300";

/// Fill colour for the x-axis title.
pub const X_AXIS_TITLE_COLOR: &str = "#131300";

/// Stroke colour for x-axis tick marks.
pub const X_AXIS_TICK_COLOR: &str = "#131300";

/// Stroke colour for the x-axis line.
pub const X_AXIS_LINE_COLOR: &str = "#131300";

/// Fill colour for y-axis tick labels.
pub const Y_AXIS_LABEL_COLOR: &str = "#131300";

/// Fill colour for the y-axis title.
pub const Y_AXIS_TITLE_COLOR: &str = "#131300";

/// Stroke colour for y-axis tick marks.
pub const Y_AXIS_TICK_COLOR: &str = "#131300";

/// Stroke colour for the y-axis line.
pub const Y_AXIS_LINE_COLOR: &str = "#131300";

// ---------------------------------------------------------------------------
// SVG identifiers
// ---------------------------------------------------------------------------

/// Fixed id attribute for the xychart SVG root element.
pub const SVG_ID: &str = "mermaid-xychart";
