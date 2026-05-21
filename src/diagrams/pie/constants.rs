//! Layout and styling constants for the pie renderer.

// ---------------------------------------------------------------------------
// Chart geometry
// ---------------------------------------------------------------------------

/// Margin between the SVG edge and the chart area (px). Matches Mermaid default.
pub const MARGIN: f64 = 40.0;

/// Side length of legend colour swatches (px).
pub const LEGEND_RECT_SIZE: f64 = 18.0;

/// Vertical gap between a legend swatch and the text beside it (px).
pub const LEGEND_SPACING: f64 = 4.0;

/// SVG height (square chart canvas, px).
pub const HEIGHT: f64 = 450.0;

/// SVG width (equals HEIGHT — square canvas, px).
pub const PIE_WIDTH: f64 = HEIGHT;

/// Fractional distance from the pie centre to the percentage label position.
/// Matches `pieConfig.textPosition` default of 0.75.
pub const TEXT_POSITION: f64 = 0.75;

/// Stroke width drawn on the outer pie circle border (px).
pub const OUTER_STROKE_WIDTH: f64 = 2.0;

/// Scale factor applied to measured legend text widths to account for browser
/// rendering differences relative to ab_glyph glyph metrics at 17 px Arial.
pub const LEGEND_TEXT_SCALE: f64 = 1.1173;

/// Font size used for legend text labels (px).
pub const LEGEND_FONT_SIZE: f64 = 17.0;

/// Font size used for the diagram title (px).
pub const TITLE_FONT_SIZE: f64 = 25.0;

/// Horizontal distance from the pie centre to the legend column start (px).
/// Equals 12 × LEGEND_RECT_SIZE = 216.
pub const LEGEND_HORIZONTAL_OFFSET: f64 = 12.0 * LEGEND_RECT_SIZE; // 216.0

// ---------------------------------------------------------------------------
// Colour palette
// ---------------------------------------------------------------------------

/// The 10 theme-independent pie colors (indices 2–11 in Mermaid's palette).
///
/// Indices 0 and 1 (pie1/pie2) are theme-dependent: they map to
/// `vars.primary_color` and `vars.secondary_color` respectively and are
/// supplied by the renderer at runtime.
///
/// HSL values from Mermaid khroma-adjusted default theme (adjusted from #ECECFF):
///   pie3  = hsl(80, 100%, 56.27%)         → khroma tertiary adjusted
///   pie4  = hsl(240, 100%, 86.27%)        → primary+l-10
///   pie5  = hsl(60, 100%, 63.53%)         → secondary+l-30
///   pie6  = hsl(80, 100%, 76.27%)         → tertiary+l-20
///   pie7  = hsl(300, 100%, 76.27%)        → primary+h+60+l-20
///   pie8  = hsl(180, 100%, 56.27%)        → primary+h-60+l-40
///   pie9  = hsl(0, 100%, 56.27%)          → primary+h+120+l-40
///   pie10 = hsl(300, 100%, 56.27%)        → primary+h+60+l-40
///   pie11 = hsl(150, 100%, 56.27%)        → primary+h-90+l-40
///   pie12 = hsl(0, 100%, 66.27%)          → primary+h+120+l-30
pub const PIE_COLORS_STATIC: [&str; 10] = [
    "hsl(80, 100%, 56.2745098039%)",  // pie3
    "hsl(240, 100%, 86.2745098039%)", // pie4
    "hsl(60, 100%, 63.5294117647%)",  // pie5
    "hsl(80, 100%, 76.2745098039%)",  // pie6
    "hsl(300, 100%, 76.2745098039%)", // pie7
    "hsl(180, 100%, 56.2745098039%)", // pie8
    "hsl(0, 100%, 56.2745098039%)",   // pie9
    "hsl(300, 100%, 56.2745098039%)", // pie10
    "hsl(150, 100%, 56.2745098039%)", // pie11
    "hsl(0, 100%, 66.2745098039%)",   // pie12
];
