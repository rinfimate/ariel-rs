//! SVG template functions for the treeView renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::esc;

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

pub fn build_css(svg_id: &str) -> String {
    format!(
        concat!(
            "#{id}{{font-family:Arial,sans-serif;font-size:16px;fill:#333;}}",
            "@keyframes edge-animation-frame{{from{{stroke-dashoffset:0;}}}}",
            "@keyframes dash{{to{{stroke-dashoffset:0;}}}}",
            "#{id} .edge-animation-slow{{stroke-dasharray:9,5!important;stroke-dashoffset:900;",
            "animation:dash 50s linear infinite;stroke-linecap:round;}}",
            "#{id} .edge-animation-fast{{stroke-dasharray:9,5!important;stroke-dashoffset:900;",
            "animation:dash 20s linear infinite;stroke-linecap:round;}}",
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
            "#{id} svg{{font-family:Arial,sans-serif;font-size:16px;}}",
            "#{id} p{{margin:0;}}",
            "#{id} .treeView-node-label{{font-size:16px;fill:black;}}",
            "#{id} .treeView-node-line{{stroke:black;}}",
            "#{id} .node .neo-node{{stroke:#9370DB;}}",
            "#{id} [data-look=\"neo\"].node rect,",
            "#{id} [data-look=\"neo\"].cluster rect,",
            "#{id} [data-look=\"neo\"].node polygon{{stroke:#9370DB;",
            "filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}",
            "#{id} [data-look=\"neo\"].node path{{stroke:#9370DB;stroke-width:1px;}}",
            "#{id} [data-look=\"neo\"].node .outer-path{{",
            "filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}",
            "#{id} [data-look=\"neo\"].node .neo-line path{{stroke:#9370DB;filter:none;}}",
            "#{id} [data-look=\"neo\"].node circle{{stroke:#9370DB;",
            "filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}",
            "#{id} [data-look=\"neo\"].node circle .state-start{{fill:#000000;}}",
            "#{id} [data-look=\"neo\"].icon-shape .icon{{fill:#9370DB;",
            "filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}",
            "#{id} [data-look=\"neo\"].icon-shape .icon-neo path{{stroke:#9370DB;",
            "filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}",
            "#{id} :root{{--mermaid-font-family:Arial,sans-serif;}}",
        ),
        id = svg_id,
    )
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG element for a treeView diagram.
///
/// `max_width` is the CSS `max-width` value (px), `vb_x/vb_y` are the viewBox
/// origin, and `vb_w/vb_h` are the viewBox dimensions.
pub fn svg_root(
    svg_id: &str,
    max_width: f64,
    vb_x: f64,
    vb_y: f64,
    vb_w: f64,
    vb_h: f64,
) -> String {
    format!(
        concat!(
            r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" "#,
            r#"xmlns:xlink="http://www.w3.org/1999/xlink" "#,
            r#"viewBox="{vx} {vy} {vw} {vh}" "#,
            r#"style="max-width: {mw}px;" "#,
            r#"role="graphics-document document" "#,
            r#"aria-roledescription="treeView">"#,
        ),
        id = svg_id,
        mw = max_width,
        vx = vb_x,
        vy = vb_y,
        vw = vb_w,
        vh = vb_h,
    )
}

// ---------------------------------------------------------------------------
// Node text
// ---------------------------------------------------------------------------

/// Render a node label `<text>` element.
pub fn node_text(x: f64, y: f64, label: &str) -> String {
    format!(
        r#"<text dominant-baseline="middle" class="treeView-node-label" x="{x}" y="{y}">{label}</text>"#,
        x = x,
        y = y,
        label = label,
    )
}

// ---------------------------------------------------------------------------
// Connector lines
// ---------------------------------------------------------------------------

/// Render a horizontal connector `<line>` element.
pub fn h_line(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    format!(
        r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke-width="1" class="treeView-node-line"></line>"#,
        x1 = x1,
        y1 = y1,
        x2 = x2,
        y2 = y2,
    )
}

/// Render a vertical connector `<line>` element.
pub fn v_line(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    format!(
        r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke-width="1" class="treeView-node-line"></line>"#,
        x1 = x1,
        y1 = y1,
        x2 = x2,
        y2 = y2,
    )
}
