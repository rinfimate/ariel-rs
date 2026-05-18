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

/// The 12-colour Mermaid default-theme pie palette, pre-computed as RGB strings.
/// HSL values have been converted to their exact browser-rendered RGB equivalents
/// so the renderer never needs to parse or convert color strings.
///
/// Conversion: hsl(h, s%, l%) → rgb(r, g, b) using standard HSL→RGB formula.
///   pie1  = #ECECFF                       → rgb(236, 236, 255)
///   pie2  = #ffffde                       → rgb(255, 255, 222)
///   pie3  = hsl(80, 100%, 56.27%)         → rgb(143, 255, 0)   (approx browser)
///   pie4  = hsl(240, 100%, 86.27%)        → rgb(184, 184, 255)
///   pie5  = hsl(60, 100%, 63.53%)         → rgb(255, 255, 70)  (approx browser)
///   pie6  = hsl(80, 100%, 76.27%)         → rgb(204, 255, 102) (approx browser)
///   pie7  = hsl(300, 100%, 76.27%)        → rgb(255, 102, 255) (approx browser)
///   pie8  = hsl(180, 100%, 56.27%)        → rgb(0, 255, 255)   (approx browser)
///   pie9  = hsl(0, 100%, 56.27%)          → rgb(255, 51, 51)   (approx browser)
///   pie10 = hsl(300, 100%, 56.27%)        → rgb(235, 28, 235)  (approx browser)
///   pie11 = hsl(150, 100%, 56.27%)        → rgb(28, 235, 143)  (approx browser)
///   pie12 = hsl(0, 100%, 66.27%)          → rgb(255, 102, 102) (approx browser)
pub const PIE_COLORS: [&str; 12] = [
    "rgb(236, 236, 255)", // pie1  — primaryColor (#ECECFF)
    "rgb(255, 255, 222)", // pie2  — secondaryColor (#ffffde)
    "rgb(143, 255, 0)",   // pie3  — hsl(80, 100%, 56.2745098039%)
    "rgb(184, 184, 255)", // pie4  — hsl(240, 100%, 86.2745098039%)
    "rgb(255, 255, 70)",  // pie5  — hsl(60, 100%, 63.5294117647%)
    "rgb(204, 255, 102)", // pie6  — hsl(80, 100%, 76.2745098039%)
    "rgb(255, 102, 255)", // pie7  — hsl(300, 100%, 76.2745098039%)
    "rgb(0, 255, 255)",   // pie8  — hsl(180, 100%, 56.2745098039%)
    "rgb(255, 51, 51)",   // pie9  — hsl(0, 100%, 56.2745098039%)
    "rgb(235, 28, 235)",  // pie10 — hsl(300, 100%, 56.2745098039%)
    "rgb(28, 235, 143)",  // pie11 — hsl(150, 100%, 56.2745098039%)
    "rgb(255, 102, 102)", // pie12 — hsl(0, 100%, 66.2745098039%)
];
