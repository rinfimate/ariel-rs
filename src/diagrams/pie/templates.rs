//! SVG template functions for the pie renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::{esc, fmt};

/// Format a value for showData display: integers show without decimal, floats show as-is.
pub fn fmt_value(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        // Strip trailing zeros
        let s = format!("{:.10}", v);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

pub fn build_style(id: &str, ff: &str) -> String {
    format!(
        concat!(
            "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}",
            "@keyframes edge-animation-frame{{from{{stroke-dashoffset:0;}}}}",
            "@keyframes dash{{to{{stroke-dashoffset:0;}}}}",
            "#{id} .edge-animation-slow{{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 50s linear infinite;stroke-linecap:round;}}",
            "#{id} .edge-animation-fast{{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 20s linear infinite;stroke-linecap:round;}}",
            "#{id} .error-icon{{fill:#552222;}}",
            "#{id} .error-text{{fill:#552222;stroke:#552222;}}",
            "#{id} .edge-thickness-normal{{stroke-width:1px;}}",
            "#{id} .edge-thickness-thick{{stroke-width:3.5px;}}",
            "#{id} .edge-pattern-solid{{stroke-dasharray:0;}}",
            "#{id} .edge-thickness-invisible{{stroke-width:0;fill:none;}}",
            "#{id} .edge-pattern-dashed{{stroke-dasharray:3;}}",
            "#{id} .edge-pattern-dotted{{stroke-dasharray:2;}}",
            "#{id} .marker{{fill:#333333;stroke:#333333;}}",
            "#{id} .marker.cross{{stroke:#333333;}}",
            "#{id} svg{{font-family:{ff};font-size:16px;}}",
            "#{id} p{{margin:0;}}",
            "#{id} .pieCircle{{stroke:black;stroke-width:2px;opacity:0.7;}}",
            "#{id} .pieOuterCircle{{stroke:black;stroke-width:2px;fill:none;}}",
            "#{id} .pieTitleText{{text-anchor:middle;font-size:25px;fill:black;font-family:{ff};}}",
            "#{id} .slice{{font-family:{ff};fill:#333;font-size:17px;}}",
            "#{id} .legend text{{fill:black;font-family:{ff};font-size:17px;}}",
            "#{id} :root{{--mermaid-font-family:{ff};}}",
        ),
        id = id,
        ff = ff,
    )
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element for a pie diagram.
pub fn svg_root(id: &str, vbx: &str, vbw: &str, vbh: &str, mw: &str) -> String {
    format!(
        r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="{vbx} 0 {vbw} {vbh}" style="max-width: {mw}px;" role="graphics-document document" aria-roledescription="pie">"#,
    )
}

// ---------------------------------------------------------------------------
// Chart elements
// ---------------------------------------------------------------------------

/// Render the main content group translated to the pie centre.
pub fn main_group(tx: &str, ty: &str) -> String {
    format!("<g transform=\"translate({tx},{ty})\">")
}

/// Render the outer circle border of the pie.
pub fn outer_circle(r: &str) -> String {
    format!(r#"<circle cx="0" cy="0" r="{r}" class="pieOuterCircle"></circle>"#)
}

/// Render a pie slice `<path>`.
pub fn pie_slice(d: &str, color: &str) -> String {
    format!(r#"<path d="{d}" fill="{color}" class="pieCircle"></path>"#)
}

/// Render a percentage label for a pie slice.
pub fn slice_label(cx: &str, cy: &str, pct: u64) -> String {
    format!(
        r#"<text transform="translate({cx},{cy})" class="slice" style="text-anchor: middle;">{pct}%</text>"#,
    )
}

/// Render the diagram title text.
pub fn title_text(y: &str, text: &str) -> String {
    format!(r#"<text x="0" y="{y}" class="pieTitleText">{text}</text>"#)
}

/// Render one legend item (coloured rect + label text).
pub fn legend_item(lx: &str, vert: &str, color: &str, text: &str) -> String {
    format!(
        r#"<g class="legend" transform="translate({lx},{vert})"><rect width="18" height="18" style="fill: {color}; stroke: {color};"></rect><text x="22" y="14">{text}</text></g>"#,
    )
}
