//! SVG template functions for the Gantt chart renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::esc;

pub fn escape_id(s: &str) -> String {
    s.replace(' ', "-")
}

// ---------------------------------------------------------------------------
// Inline style helpers (replaces CSS class rules)
// ---------------------------------------------------------------------------

/// Return the inline fill and stroke attributes for a section band, by class index (0–3).
/// Matches Mermaid's .section0 (colour A), .section2 (colour B), .section1/.section3 (background).
/// `section_color` — fill for index 0 (and 4, 8, …).
/// `section_color2` — fill for index 2 (and 6, 10, …). May differ from section_color (e.g. yellow in default theme).
/// `bg_color` — fill for odd indices (1, 3, …); typically white/background.
pub fn section_band_style(
    class_idx: usize,
    section_color: &str,
    section_color2: &str,
    bg_color: &str,
) -> String {
    match class_idx % 4 {
        0 => format!(r##"fill="{section_color}" stroke="none" opacity="0.2""##),
        2 => format!(r##"fill="{section_color2}" stroke="none" opacity="0.2""##),
        _ => format!(r##"fill="{bg_color}" stroke="none" opacity="0.2""##),
    }
}

/// Return the inline style for a task bar class string (e.g. "task0", "done2").
/// Colors match Mermaid's per-theme gantt CSS (.task0, .active0, .done0, .crit0, etc.).
pub fn task_bar_style(tc: &str, theme: crate::theme::Theme) -> String {
    use crate::theme::Theme;
    match theme {
        Theme::Dark => {
            if tc.starts_with("doneCrit") {
                "stroke:#E83737;fill:lightgrey;stroke-width:2"
            } else if tc.starts_with("activeCrit") {
                "stroke:#E83737;fill:#81B1DB;stroke-width:2"
            } else if tc.starts_with("done") {
                "stroke:grey;fill:lightgrey;stroke-width:2"
            } else if tc.starts_with("active") {
                "fill:#81B1DB;stroke:#ffffff;stroke-width:2"
            } else if tc.starts_with("crit") {
                "stroke:#E83737;fill:#E83737;stroke-width:2"
            } else {
                "fill:hsl(180,1.5873015873%,35.3529411765%);stroke:#ffffff;stroke-width:2"
            }
        }
        Theme::Forest => {
            if tc.starts_with("doneCrit") {
                "stroke:#ff8888;fill:lightgrey;stroke-width:2"
            } else if tc.starts_with("activeCrit") {
                "stroke:#ff8888;fill:#cde498;stroke-width:2"
            } else if tc.starts_with("done") {
                "stroke:grey;fill:lightgrey;stroke-width:2"
            } else if tc.starts_with("active") {
                "fill:#cde498;stroke:#13540c;stroke-width:2"
            } else if tc.starts_with("crit") {
                "stroke:#ff8888;fill:red;stroke-width:2"
            } else {
                "fill:#487e3a;stroke:#13540c;stroke-width:2"
            }
        }
        Theme::Neutral => {
            if tc.starts_with("doneCrit") {
                "stroke:hsl(10.9090909091,73.3333333333%,40%);fill:#bbb;stroke-width:2"
            } else if tc.starts_with("activeCrit") {
                "stroke:hsl(10.9090909091,73.3333333333%,40%);fill:#eee;stroke-width:2"
            } else if tc.starts_with("done") {
                "stroke:#666;fill:#bbb;stroke-width:2"
            } else if tc.starts_with("active") {
                "fill:#eee;stroke:hsl(0,0%,33.9215686275%);stroke-width:2"
            } else if tc.starts_with("crit") {
                "stroke:hsl(10.9090909091,73.3333333333%,40%);fill:#d42;stroke-width:2"
            } else {
                "fill:#707070;stroke:hsl(0,0%,33.9215686275%);stroke-width:2"
            }
        }
        Theme::Default => {
            if tc.starts_with("doneCrit") {
                "stroke:#ff8888;fill:lightgrey;stroke-width:2"
            } else if tc.starts_with("activeCrit") {
                "stroke:#ff8888;fill:#bfc7ff;stroke-width:2"
            } else if tc.starts_with("done") {
                "stroke:grey;fill:lightgrey;stroke-width:2"
            } else if tc.starts_with("active") {
                "fill:#bfc7ff;stroke:#534fbc;stroke-width:2"
            } else if tc.starts_with("crit") {
                "stroke:#ff8888;fill:red;stroke-width:2"
            } else {
                "fill:#8a90dd;stroke:#534fbc;stroke-width:2"
            }
        }
    }
    .to_string()
}

/// Return inline fill for task text class string.
/// `contrast_color` is the per-theme dark text colour used for done/active/crit states.
pub fn task_text_fill<'a>(
    cls: &str,
    outside: bool,
    text_color: &'a str,
    contrast_color: &'a str,
) -> &'a str {
    if outside {
        text_color
    } else if cls.contains("doneCritText")
        || cls.contains("doneText")
        || cls.contains("activeCritText")
        || cls.contains("activeText")
    {
        contrast_color
    } else {
        "white"
    }
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for a Gantt chart.
pub fn svg_root(id: &str, w: f64, h: i64) -> String {
    format!(
        r##"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" font-family="Arial, sans-serif" viewBox="0 0 {w} {h}" style="max-width: {w}px;" role="graphics-document document" aria-roledescription="gantt">"##,
        id = id,
        w = w,
        h = h,
    )
}

// ---------------------------------------------------------------------------
// Grid rendering
// ---------------------------------------------------------------------------

/// Render the x-axis grid `<g>` opening tag with translate and font attributes.
pub fn grid_group_open(left_pad: i64, grid_y: i64, axis_font_size: i64) -> String {
    format!(
        r##"<g class="grid" transform="translate({left_pad}, {grid_y})" fill="none" font-size="{axis_font_size}" font-family="sans-serif" text-anchor="middle">"##,
        left_pad = left_pad,
        grid_y = grid_y,
        axis_font_size = axis_font_size,
    )
}

/// Render the x-axis domain `<path>` (horizontal baseline with top-tick caps).
pub fn grid_domain_path(top: i64, right: f64) -> String {
    format!(
        r##"<path class="domain" stroke="none" d="M0.5,{top}V0.5H{right}V{top}"></path>"##,
        top = top,
        right = right,
    )
}

/// Render a single x-axis tick `<g>` (line + label).
pub fn grid_tick(x: f64, top: i64, axis_font_size: i64, label: &str, text_color: &str) -> String {
    format!(
        "<g class=\"tick\" opacity=\"1\" transform=\"translate({x},0)\"><line stroke=\"currentColor\" y2=\"{top}\"></line><text fill=\"{text_color}\" y=\"3\" dy=\"1em\" stroke=\"none\" font-size=\"{afs}\" style=\"text-anchor: middle;\">{label}</text></g>",
        x = x, top = top, afs = axis_font_size, label = label, text_color = text_color,
    )
}

// ---------------------------------------------------------------------------
// Exclude-range shading
// ---------------------------------------------------------------------------

/// Render a weekend/exclusion shading `<rect>`.
#[allow(clippy::too_many_arguments)]
pub fn exclude_rect(
    id: &str,
    date: &str,
    x: i64,
    y: i64,
    w: i64,
    h: i64,
    ox: i64,
    oy: i64,
    exclude_color: &str,
) -> String {
    format!(
        r##"<rect id="{id}-exclude-{date}" x="{x}" y="{y}" width="{w}" height="{h}" transform-origin="{ox}px {oy}px" fill="{exclude_color}" class="exclude-range"></rect>"##,
        id = id,
        date = date,
        x = x,
        y = y,
        w = w,
        h = h,
        ox = ox,
        oy = oy,
        exclude_color = exclude_color,
    )
}

// ---------------------------------------------------------------------------
// Section bands
// ---------------------------------------------------------------------------

/// Render a section background band `<rect>`.
pub fn section_band_rect(
    y: i64,
    w: f64,
    h: i64,
    class_idx: usize,
    section_color: &str,
    section_color2: &str,
    bg_color: &str,
) -> String {
    let style = section_band_style(class_idx, section_color, section_color2, bg_color);
    format!(
        r##"<rect x="0" y="{y}" width="{w}" height="{h}" {style} class="section section{ci}"></rect>"##,
        y = y,
        w = w,
        h = h,
        style = style,
        ci = class_idx,
    )
}

// ---------------------------------------------------------------------------
// Task bars
// ---------------------------------------------------------------------------

/// Render a milestone diamond `<rect>` (rotated 45° to create diamond shape).
#[allow(clippy::too_many_arguments)]
pub fn milestone_rect(
    id: &str,
    tid: &str,
    rx: f64,
    ry: f64,
    size: f64,
    ox: f64,
    oy: f64,
    tc: &str,
    theme: crate::theme::Theme,
) -> String {
    let bar_style = task_bar_style(tc, theme);
    format!(
        r##"<rect id="{id}-{tid}" rx="3" ry="3" x="{rx}" y="{ry}" width="{size}" height="{size}" transform-origin="{ox}px {oy}px" transform="rotate(45)" style="{bar_style}" class="task {tc} milestone"></rect>"##,
        id = id,
        tid = tid,
        rx = rx,
        ry = ry,
        size = size,
        ox = ox,
        oy = oy,
        tc = tc,
        bar_style = bar_style,
    )
}

/// Render a normal task bar `<rect>`.
#[allow(clippy::too_many_arguments)]
pub fn task_bar_rect(
    id: &str,
    tid: &str,
    bx: i64,
    by: i64,
    bw: i64,
    bh: i64,
    cx: i64,
    cy: i64,
    tc: &str,
    theme: crate::theme::Theme,
) -> String {
    let bar_style = task_bar_style(tc, theme);
    format!(
        r##"<rect id="{id}-{tid}" rx="3" ry="3" x="{bx}" y="{by}" width="{bw}" height="{bh}" transform-origin="{cx}px {cy}px" style="{bar_style}" class="task {tc} "></rect>"##,
        id = id,
        tid = tid,
        bx = bx,
        by = by,
        bw = bw,
        bh = bh,
        cx = cx,
        cy = cy,
        tc = tc,
        bar_style = bar_style,
    )
}

/// Render a task label `<text>`.
#[allow(clippy::too_many_arguments)]
pub fn task_text(
    id: &str,
    tid: &str,
    fs: i64,
    tx: i64,
    ty: i64,
    bh: i64,
    tc: &str,
    label: &str,
    text_color: &str,
    contrast_color: &str,
) -> String {
    // Determine text fill from class name (replaces CSS class rules)
    let outside = tc.contains("Outside");
    let fill = task_text_fill(tc, outside, text_color, contrast_color);
    let anchor = if tc.contains("OutsideRight") {
        "start"
    } else if tc.contains("OutsideLeft") {
        "end"
    } else {
        "middle"
    };
    let italic = if tc.contains("milestone") {
        " font-style=\"italic\""
    } else {
        ""
    };
    format!(
        r##"<text id="{id}-{tid}-text" font-size="{fs}" x="{tx}" y="{ty}" text-height="{bh}" fill="{fill}" text-anchor="{anchor}"{italic} class="{tc}">{label}</text>"##,
        id = id,
        tid = tid,
        fs = fs,
        tx = tx,
        ty = ty,
        bh = bh,
        fill = fill,
        anchor = anchor,
        italic = italic,
        tc = tc,
        label = label,
    )
}

// ---------------------------------------------------------------------------
// Section title labels
// ---------------------------------------------------------------------------

/// Render a section title `<text>` with `<tspan>`.
pub fn section_title(y: i64, fs: i64, class_idx: usize, label: &str, text_color: &str) -> String {
    format!(
        r##"<text dy="0em" x="10" y="{y}" font-size="{fs}" fill="{text_color}" text-anchor="start" class="sectionTitle sectionTitle{ci}"><tspan alignment-baseline="central" x="10">{label}</tspan></text>"##,
        y = y,
        fs = fs,
        ci = class_idx,
        label = label,
        text_color = text_color,
    )
}

// ---------------------------------------------------------------------------
// Today line
// ---------------------------------------------------------------------------

/// Render the today marker `<line>`.
pub fn today_line(tx: i64, top: i64, bot: i64) -> String {
    format!(
        r##"<g class="today"><line x1="{tx}" x2="{tx}" y1="{top}" y2="{bot}" fill="none" stroke="red" stroke-width="2" class="today"></line></g>"##,
        tx = tx,
        top = top,
        bot = bot,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the Gantt diagram title `<text>`.
pub fn title_text(cx: i64, ty: i64, title: &str, text_color: &str) -> String {
    format!(
        r##"<text x="{cx}" y="{ty}" text-anchor="middle" font-size="18" fill="{text_color}" class="titleText">{title}</text>"##,
        cx = cx,
        ty = ty,
        title = title,
        text_color = text_color,
    )
}

// ---------------------------------------------------------------------------
// Empty diagram fallback
// ---------------------------------------------------------------------------

/// Render the empty Gantt fallback SVG.
pub fn empty_svg() -> &'static str {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50"><text x="10" y="30">Empty Gantt</text></svg>"##
}
