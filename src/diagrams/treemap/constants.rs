//! Layout and styling constants for the treemap renderer.

// ---------------------------------------------------------------------------
// Canvas geometry
// ---------------------------------------------------------------------------

/// Canvas width (nodeWidth × 10 = 100 × 10, px).
pub(crate) const CANVAS_W: f64 = 1000.0;

/// Canvas height (nodeHeight × 10 = 40 × 10, px).
pub(crate) const CANVAS_H: f64 = 400.0;

// ---------------------------------------------------------------------------
// Padding / layout
// ---------------------------------------------------------------------------

/// Inner padding between adjacent tiles (paddingInner, px).
pub(crate) const PADDING_INNER: f64 = 10.0;

/// Height of the section header area inside branch nodes (paddingTop, px).
pub(crate) const SECTION_HEADER_HEIGHT: f64 = 25.0;

/// Side/bottom padding inside branch nodes (paddingLeft/Right/Bottom, px).
pub(crate) const SECTION_PADDING: f64 = 10.0;

/// Height reserved above the treemap for the diagram title (px).
pub(crate) const TITLE_HEIGHT: f64 = 30.0;

/// Padding added around the content bounding box for the SVG viewBox (px).
pub(crate) const DIAGRAM_PADDING: f64 = 8.0;

// ---------------------------------------------------------------------------
// Squarify layout
// ---------------------------------------------------------------------------

/// Golden-ratio constant used by the D3 squarify algorithm (φ = (1+√5)/2).
pub(crate) const PHI: f64 = 1.618_033_988_749_895;

// ---------------------------------------------------------------------------
// Font metric
// ---------------------------------------------------------------------------

/// Character-width scaling factor applied to raw font measurements.
/// Approximates an average 0.55 px/px ratio per character at any font size.
#[allow(dead_code)]
pub(crate) const CHAR_WIDTH_FACTOR: f64 = 0.55;

/// Maximum initial font size for leaf labels before shrinking (px).
pub(crate) const MAX_LABEL_FONT: f64 = 38.0;

/// Minimum available width/height for a leaf tile to render a label (px).
pub(crate) const MIN_TILE_AVAIL: f64 = 10.0;

/// Inner tile padding on each edge used to compute available label space (px).
pub(crate) const TILE_INNER_PAD: f64 = 4.0;

/// Gap between the leaf label and the value text (px).
pub(crate) const VALUE_Y_GAP: f64 = 2.0;

/// Scale factor from label font size to value font size.
pub(crate) const VALUE_FONT_SCALE: f64 = 0.6;

/// Minimum value font size (px).
pub(crate) const VALUE_FONT_MIN: u32 = 10;

// ---------------------------------------------------------------------------
// Section clip path
// ---------------------------------------------------------------------------

/// Horizontal clip margin inside section header clip paths (px).
pub(crate) const SECTION_CLIP_MARGIN: f64 = 12.0;

/// Horizontal clip margin inside leaf clip paths (px).
pub(crate) const LEAF_CLIP_MARGIN: f64 = 4.0;

/// X margin from right edge for section value text (px).
pub(crate) const SECTION_VALUE_X_MARGIN: f64 = 10.0;

// ---------------------------------------------------------------------------
// Color palettes (Mermaid default theme v11.15, double-updateColors)
// ---------------------------------------------------------------------------

/// cScale0..11 background colors from the Mermaid default theme.
pub(crate) const C_SCALE: &[&str] = &[
    "hsl(240, 100%, 76.2745098039%)",
    "hsl(60, 100%, 73.5294117647%)",
    "hsl(80, 100%, 76.2745098039%)",
    "hsl(270, 100%, 76.2745098039%)",
    "hsl(300, 100%, 76.2745098039%)",
    "hsl(330, 100%, 76.2745098039%)",
    "hsl(0, 100%, 76.2745098039%)",
    "hsl(30, 100%, 76.2745098039%)",
    "hsl(90, 100%, 76.2745098039%)",
    "hsl(150, 100%, 76.2745098039%)",
    "hsl(180, 100%, 76.2745098039%)",
    "hsl(210, 100%, 76.2745098039%)",
];

/// cScalePeer0..11 border/stroke colors from the Mermaid default theme.
pub(crate) const C_SCALE_PEER: &[&str] = &[
    "hsl(240, 100%, 61.2745098039%)",
    "hsl(60, 100%, 48.5294117647%)",
    "hsl(80, 100%, 56.2745098039%)",
    "hsl(270, 100%, 61.2745098039%)",
    "hsl(300, 100%, 61.2745098039%)",
    "hsl(330, 100%, 61.2745098039%)",
    "hsl(0, 100%, 61.2745098039%)",
    "hsl(30, 100%, 61.2745098039%)",
    "hsl(90, 100%, 61.2745098039%)",
    "hsl(150, 100%, 61.2745098039%)",
    "hsl(180, 100%, 61.2745098039%)",
    "hsl(210, 100%, 61.2745098039%)",
];

/// Label text colors (cScaleLabel): indices 0 and 3 are white, all others black.
pub(crate) fn c_scale_label(idx: usize) -> &'static str {
    match idx % 12 {
        0 | 3 => "#ffffff",
        _ => "black",
    }
}

// Neutral theme fill palette — all grays, confirmed from reference SVGs.
const C_SCALE_NEUTRAL: &[&str] = &[
    "#555", "#F4F4F4", "#555", "#BBB", "#777", "#999", "#DDD", "#FFF", "#DDD", "#BBB", "#999",
    "#777",
];

// Neutral theme stroke palette — peer lightness = fill lightness − 10%.
const C_SCALE_PEER_NEUTRAL: &[&str] = &[
    "hsl(0, 0%, 23.3333333333%)",
    "hsl(0, 0%, 85.6862745098%)",
    "hsl(0, 0%, 23.3333333333%)",
    "hsl(0, 0%, 63.3333333333%)",
    "hsl(0, 0%, 36.6666666667%)",
    "hsl(0, 0%, 50%)",
    "hsl(0, 0%, 76.6666666667%)",
    "hsl(0, 0%, 90%)",
    "hsl(0, 0%, 76.6666666667%)",
    "hsl(0, 0%, 63.3333333333%)",
    "hsl(0, 0%, 50%)",
    "hsl(0, 0%, 36.6666666667%)",
];

// Forest theme fill palette — green HSL values, confirmed from reference SVGs.
const C_SCALE_FOREST: &[&str] = &[
    "hsl(78.1578947368, 58.4615384615%, 64.5098039216%)",
    "hsl(98.961038961, 100%, 74.9019607843%)",
    "hsl(78.1578947368, 58.4615384615%, 74.5098039216%)",
    "hsl(118.1578947368, 58.4615384615%, 64.5098039216%)",
    "hsl(58.1578947368, 58.4615384615%, 64.5098039216%)",
    "hsl(138.961038961, 100%, 74.9019607843%)",
    "hsl(78.1578947368, 58.4615384615%, 84.5098039216%)",
    "hsl(98.961038961, 100%, 84.9019607843%)",
    "hsl(78.1578947368, 40%, 64.5098039216%)",
    "hsl(98.961038961, 80%, 74.9019607843%)",
    "hsl(78.1578947368, 58.4615384615%, 54.5098039216%)",
    "hsl(98.961038961, 100%, 64.9019607843%)",
];

// Forest theme stroke palette — peer lightness = fill lightness − 25%.
const C_SCALE_PEER_FOREST: &[&str] = &[
    "hsl(78.1578947368, 58.4615384615%, 39.5098039216%)",
    "hsl(98.961038961, 100%, 39.9019607843%)",
    "hsl(78.1578947368, 58.4615384615%, 44.5098039216%)",
    "hsl(118.1578947368, 58.4615384615%, 39.5098039216%)",
    "hsl(58.1578947368, 58.4615384615%, 39.5098039216%)",
    "hsl(138.961038961, 100%, 39.9019607843%)",
    "hsl(78.1578947368, 58.4615384615%, 59.5098039216%)",
    "hsl(98.961038961, 100%, 59.9019607843%)",
    "hsl(78.1578947368, 40%, 39.5098039216%)",
    "hsl(98.961038961, 80%, 49.9019607843%)",
    "hsl(78.1578947368, 58.4615384615%, 29.5098039216%)",
    "hsl(98.961038961, 100%, 39.9019607843%)",
];

// ---------------------------------------------------------------------------
// Theme-aware color palette accessors
// ---------------------------------------------------------------------------

pub(crate) fn theme_c_scale(theme: crate::theme::Theme) -> &'static [&'static str] {
    match theme {
        crate::theme::Theme::Dark => &[
            "#1f2020", "#0b0000", "#4d1037", "#3f5258", "#4f2f1b", "#6e0a0a", "#3b0048", "#995a01",
            "#154706", "#161722", "#00296f", "#01629c",
        ],
        crate::theme::Theme::Neutral => C_SCALE_NEUTRAL,
        crate::theme::Theme::Forest => C_SCALE_FOREST,
        _ => C_SCALE,
    }
}

pub(crate) fn theme_c_scale_peer(theme: crate::theme::Theme) -> &'static [&'static str] {
    match theme {
        crate::theme::Theme::Dark => &[
            "hsl(180, 1.5873015873%, 22.3529411765%)",
            "hsl(0, 100%, 12.1568627451%)",
            "hsl(321.6393442623, 65.5913978495%, 28.2352941176%)",
            "hsl(196.3636363636, 14.0625%, 28.0392156863%)",
            "hsl(24.8571428571, 46.0784313725%, 21.9607843137%)",
            "hsl(0, 83.5820895522%, 23.1372549020%)",
            "hsl(288, 100%, 14.1176470588%)",
            "hsl(34.2857142857, 100%, 29.8039215686%)",
            "hsl(125.2173913043, 56.3218390805%, 14.9019607843%)",
            "hsl(238.4615384615, 13.5922330097%, 17.2549019608%)",
            "hsl(232.7272727273, 100%, 21.7647058824%)",
            "hsl(204.9230769231, 100%, 31.1764705882%)",
        ],
        crate::theme::Theme::Neutral => C_SCALE_PEER_NEUTRAL,
        crate::theme::Theme::Forest => C_SCALE_PEER_FOREST,
        _ => C_SCALE_PEER,
    }
}

/// Return the cScaleLabel text color for the given theme and palette index.
///
/// Dark: all labels use `lightgrey`.
/// Neutral: white for dark fills (#555 at indices 0, 2, 4, 11), `#333` elsewhere.
/// Default/Forest: indices 0 and 3 are white, all others black.
pub(crate) fn theme_c_scale_label(theme: crate::theme::Theme, idx: usize) -> &'static str {
    match theme {
        crate::theme::Theme::Dark => "lightgrey",
        crate::theme::Theme::Forest => "black",
        // Neutral: dark fills (#555) at indices 0 and 2 need light text; all others use dark text
        crate::theme::Theme::Neutral => match idx % 12 {
            0 | 2 => "#F4F4F4",
            _ => "#333",
        },
        _ => c_scale_label(idx),
    }
}
