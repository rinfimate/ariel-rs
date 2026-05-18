//! SVG template functions for the packet renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element for a packet diagram.
pub fn svg_root(svg_id: &str, w: &str, h: &str, max_w: &str) -> String {
    format!(
        "<svg id=\"{svg_id}\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" viewBox=\"0 0 {w} {h}\" style=\"max-width: {max_w}px;\" role=\"graphics-document document\" aria-roledescription=\"packet\" aria-labelledby=\"chart-title-{svg_id}\"><title id=\"chart-title-{svg_id}\">Packet</title>",
    )
}

/// Render an empty packet SVG placeholder.
pub fn empty_svg(svg_id: &str, ff: &str) -> String {
    format!(
        "<svg id=\"{svg_id}\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 50\"><text x=\"10\" y=\"30\" font-family=\"{ff}\" font-size=\"14\">Empty packet diagram</text></svg>",
    )
}

// ---------------------------------------------------------------------------
// Field elements
// ---------------------------------------------------------------------------

/// Render a packet field background rectangle.
pub fn field_rect(x: &str, y: &str, w: &str, h: &str) -> String {
    format!("<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" class=\"packetBlock\"></rect>",)
}

/// Render a packet field label (centered in the box).
pub fn field_label(x: &str, y: &str, text: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" class=\"packetLabel\" dominant-baseline=\"middle\" text-anchor=\"middle\">{text}</text>",
    )
}

/// Render a bit-number label for a single-bit field (centered, middle anchor).
pub fn bit_number_single(x: &str, y: &str, bit: u32) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" class=\"packetByte start\" dominant-baseline=\"auto\" text-anchor=\"middle\">{bit}</text>",
    )
}

/// Render the start bit-number label for a multi-bit field (left-aligned).
pub fn bit_number_start(x: &str, y: &str, bit: u32) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" class=\"packetByte start\" dominant-baseline=\"auto\" text-anchor=\"start\">{bit}</text>",
    )
}

/// Render the end bit-number label for a multi-bit field (right-aligned).
pub fn bit_number_end(x: &str, y: &str, bit: u32) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" class=\"packetByte end\" dominant-baseline=\"auto\" text-anchor=\"end\">{bit}</text>",
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title text element.
pub fn title(x: &str, y: &str, text: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" dominant-baseline=\"middle\" text-anchor=\"middle\" class=\"packetTitle\">{text}</text>",
    )
}
