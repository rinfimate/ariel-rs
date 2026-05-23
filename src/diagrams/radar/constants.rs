//! Layout and styling constants for the radar renderer.
//!
//! Values are taken directly from the Mermaid JS default config:
//! chunk-Q52JI7PB.mjs lines 5060-5071 and theme variable defaults.

// ---------------------------------------------------------------------------
// SVG / chart dimensions (all match Mermaid JS defaults)
// ---------------------------------------------------------------------------

/// Inner chart width (px) — "config.width" in Mermaid JS.
pub const CHART_WIDTH: f64 = 600.0;

/// Inner chart height (px) — "config.height" in Mermaid JS.
pub const CHART_HEIGHT: f64 = 600.0;

/// Top margin (px).
pub const MARGIN_TOP: f64 = 50.0;

/// Right margin (px).
pub const MARGIN_RIGHT: f64 = 50.0;

/// Bottom margin (px).
pub const MARGIN_BOTTOM: f64 = 50.0;

/// Left margin (px).
pub const MARGIN_LEFT: f64 = 50.0;

// Derived totals
/// Total SVG width = CHART_WIDTH + MARGIN_LEFT + MARGIN_RIGHT.
pub const SVG_WIDTH: f64 = CHART_WIDTH + MARGIN_LEFT + MARGIN_RIGHT; // 700

/// Total SVG height = CHART_HEIGHT + MARGIN_TOP + MARGIN_BOTTOM.
pub const SVG_HEIGHT: f64 = CHART_HEIGHT + MARGIN_TOP + MARGIN_BOTTOM; // 700

// ---------------------------------------------------------------------------
// Axis rendering
// ---------------------------------------------------------------------------

/// Factor applied to radius for the end of the axis spoke.
/// axisScaleFactor = 1 (default). Can be overridden per-diagram.
#[allow(dead_code)]
pub const AXIS_SCALE_FACTOR: f64 = 1.0;

/// Factor applied to radius for axis label placement.
/// axisLabelFactor = 1.05 (default).
#[allow(dead_code)]
pub const AXIS_LABEL_FACTOR: f64 = 1.05;

// ---------------------------------------------------------------------------
// Curve rendering
// ---------------------------------------------------------------------------

/// Catmull-Rom tension — matches Mermaid JS curveTension = 0.17.
/// Note: Mermaid uses this directly (NOT divided by 3).
pub const CURVE_TENSION: f64 = 0.17;

/// Curve fill opacity.
pub const CURVE_OPACITY: f64 = 0.5;

/// Curve stroke width (px).
pub const CURVE_STROKE_WIDTH: f64 = 2.0;

// ---------------------------------------------------------------------------
// Graticule styling
// ---------------------------------------------------------------------------

/// Fill/stroke colour for graticule rings (default fallback — renderer uses theme vars).
#[allow(dead_code)]
pub const GRATICULE_COLOR: &str = "#DEDEDE";

/// Stroke colour for axis spoke lines — matches Mermaid's .radarAxisLine { stroke:#333333 }
pub const AXIS_LINE_COLOR: &str = "#333333";

/// Graticule fill opacity.
pub const GRATICULE_OPACITY: f64 = 0.3;

/// Graticule stroke width (px).
pub const GRATICULE_STROKE_WIDTH: f64 = 1.0;

// ---------------------------------------------------------------------------
// Legend
// ---------------------------------------------------------------------------

/// Side length of each legend colour swatch (px). Emitted directly in templates.
#[allow(dead_code)]
pub const LEGEND_BOX_SIZE: f64 = 12.0;

/// Font size for legend labels (px).
pub const LEGEND_FONT_SIZE: f64 = 12.0;

/// Vertical spacing between legend rows (px).
pub const LEGEND_LINE_HEIGHT: f64 = 20.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for axis labels (px).
pub const AXIS_LABEL_FONT_SIZE: f64 = 12.0;

/// Axis stroke width (px). Emitted directly in CSS styles.
#[allow(dead_code)]
pub const AXIS_STROKE_WIDTH: f64 = 2.0;
