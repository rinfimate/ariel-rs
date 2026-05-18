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
// Colours (Mermaid default theme variables)
// ---------------------------------------------------------------------------

/// Fill colour for quadrant-1 (top-right). Matches cScale0 primary colour.
pub const QUADRANT1_FILL: &str = "#ECECFF";

/// Fill colour for quadrant-2 (top-left). Matches cScale1 colour.
pub const QUADRANT2_FILL: &str = "#f1f1ff";

/// Fill colour for quadrant-3 (bottom-left). Matches cScale2 colour.
pub const QUADRANT3_FILL: &str = "#f6f6ff";

/// Fill colour for quadrant-4 (bottom-right). Matches cScale3 colour.
pub const QUADRANT4_FILL: &str = "#fbfbff";

/// Text fill colour for quadrant-1 labels.
pub const QUADRANT1_TEXT_FILL: &str = "#131300";

/// Text fill colour for quadrant-2 labels.
pub const QUADRANT2_TEXT_FILL: &str = "#0e0e00";

/// Text fill colour for quadrant-3 labels.
pub const QUADRANT3_TEXT_FILL: &str = "#090900";

/// Text fill colour for quadrant-4 labels.
pub const QUADRANT4_TEXT_FILL: &str = "#040400";

/// Fill colour for data point circles.
/// Mermaid emits `hsl(240, 100%, NaN%)` when the theme variable is unresolved.
pub const QUADRANT_POINT_FILL: &str = "hsl(240, 100%, NaN%)";

/// Text fill for data point labels.
pub const QUADRANT_POINT_TEXT_FILL: &str = "#131300";

/// Text fill for x-axis labels.
pub const QUADRANT_X_AXIS_TEXT_FILL: &str = "#131300";

/// Text fill for y-axis labels.
pub const QUADRANT_Y_AXIS_TEXT_FILL: &str = "#131300";

/// Text fill for the diagram title.
pub const QUADRANT_TITLE_FILL: &str = "#131300";

/// Stroke colour for the internal (dividing) border lines.
pub const QUADRANT_INTERNAL_BORDER_STROKE_FILL: &str = "rgb(199, 199, 241)";

/// Stroke colour for the external (outer) border lines.
pub const QUADRANT_EXTERNAL_BORDER_STROKE_FILL: &str = "rgb(199, 199, 241)";
