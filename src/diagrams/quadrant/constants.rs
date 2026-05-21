//! Layout and styling constants for the quadrant renderer.

// ---------------------------------------------------------------------------
// Chart geometry (matches defaultConfig.quadrantChart in Mermaid)
// ---------------------------------------------------------------------------

/// Total SVG width of the quadrant chart (px).
pub const CHART_WIDTH: f64 = 500.0;

/// Total SVG height of the quadrant chart (px).
pub const CHART_HEIGHT: f64 = 500.0;

/// Padding above/below the title text (px).
pub const TITLE_PADDING: f64 = 10.0;

/// Font size of the diagram title (px).
pub const TITLE_FONT_SIZE: f64 = 20.0;

/// Padding between quadrant areas (px).
pub const QUADRANT_PADDING: f64 = 5.0;

/// Padding between the axis label and the chart edge (px).
pub const X_AXIS_LABEL_PADDING: f64 = 5.0;

/// Padding between the y-axis label and the chart edge (px).
pub const Y_AXIS_LABEL_PADDING: f64 = 5.0;

/// Font size for x-axis labels (px).
pub const X_AXIS_LABEL_FONT_SIZE: f64 = 16.0;

/// Font size for y-axis labels (px).
pub const Y_AXIS_LABEL_FONT_SIZE: f64 = 16.0;

/// Font size for quadrant labels (px).
pub const QUADRANT_LABEL_FONT_SIZE: f64 = 16.0;

/// Top padding applied to quadrant text when data points are present (px).
pub const QUADRANT_TEXT_TOP_PADDING: f64 = 5.0;

/// Padding between a data point circle and its label text (px).
pub const POINT_TEXT_PADDING: f64 = 5.0;

/// Font size for data point labels (px).
pub const POINT_LABEL_FONT_SIZE: f64 = 12.0;

/// Radius of data point circles (px).
pub const POINT_RADIUS: f64 = 5.0;

/// Stroke width for internal (dividing) border lines (px).
pub const QUADRANT_INTERNAL_BORDER_STROKE_WIDTH: f64 = 1.0;

/// Stroke width for external (outer) border lines (px).
pub const QUADRANT_EXTERNAL_BORDER_STROKE_WIDTH: f64 = 2.0;

// ---------------------------------------------------------------------------
// Colours
// ---------------------------------------------------------------------------
// All quadrant fill colours, text colours, and border colours are resolved
// from the active Theme at render time (via Theme::resolve() → ThemeVars).
// No hardcoded palette constants are needed here.
