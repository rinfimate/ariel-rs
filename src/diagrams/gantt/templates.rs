//! SVG template functions for the Gantt chart renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for a Gantt chart.
pub fn svg_root(id: &str, w: f64, h: i64) -> String {
    format!(
        r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 {w} {h}" style="max-width: {w}px;" role="graphics-document document" aria-roledescription="gantt">"#,
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
        r#"<g class="grid" transform="translate({left_pad}, {grid_y})" fill="none" font-size="{axis_font_size}" font-family="sans-serif" text-anchor="middle">"#,
        left_pad = left_pad,
        grid_y = grid_y,
        axis_font_size = axis_font_size,
    )
}

/// Render the x-axis domain `<path>` (horizontal baseline with top-tick caps).
pub fn grid_domain_path(top: i64, right: f64) -> String {
    format!(
        r#"<path class="domain" stroke="currentColor" d="M0.5,{top}V0.5H{right}V{top}"></path>"#,
        top = top,
        right = right,
    )
}

/// Render a single x-axis tick `<g>` (line + label).
pub fn grid_tick(x: f64, top: i64, axis_font_size: i64, label: &str) -> String {
    format!(
        "<g class=\"tick\" opacity=\"1\" transform=\"translate({x},0)\"><line stroke=\"currentColor\" y2=\"{top}\"></line><text fill=\"#000\" y=\"3\" dy=\"1em\" stroke=\"none\" font-size=\"{afs}\" style=\"text-anchor: middle;\">{label}</text></g>",
        x = x, top = top, afs = axis_font_size, label = label,
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
) -> String {
    format!(
        r#"<rect id="{id}-exclude-{date}" x="{x}" y="{y}" width="{w}" height="{h}" transform-origin="{ox}px {oy}px" class="exclude-range"></rect>"#,
        id = id,
        date = date,
        x = x,
        y = y,
        w = w,
        h = h,
        ox = ox,
        oy = oy,
    )
}

// ---------------------------------------------------------------------------
// Section bands
// ---------------------------------------------------------------------------

/// Render a section background band `<rect>`.
pub fn section_band_rect(y: i64, w: f64, h: i64, class_idx: usize) -> String {
    format!(
        r#"<rect x="0" y="{y}" width="{w}" height="{h}" class="section section{ci}"></rect>"#,
        y = y,
        w = w,
        h = h,
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
) -> String {
    format!(
        r#"<rect id="{id}-{tid}" rx="0" ry="0" x="{rx}" y="{ry}" width="{size}" height="{size}" transform-origin="{ox}px {oy}px" transform="rotate(45)" class="task {tc} milestone"></rect>"#,
        id = id,
        tid = tid,
        rx = rx,
        ry = ry,
        size = size,
        ox = ox,
        oy = oy,
        tc = tc,
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
) -> String {
    format!(
        r#"<rect id="{id}-{tid}" rx="3" ry="3" x="{bx}" y="{by}" width="{bw}" height="{bh}" transform-origin="{cx}px {cy}px" class="task {tc} "></rect>"#,
        id = id,
        tid = tid,
        bx = bx,
        by = by,
        bw = bw,
        bh = bh,
        cx = cx,
        cy = cy,
        tc = tc,
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
) -> String {
    format!(
        r#"<text id="{id}-{tid}-text" font-size="{fs}" x="{tx}" y="{ty}" text-height="{bh}" class="{tc}">{label}</text>"#,
        id = id,
        tid = tid,
        fs = fs,
        tx = tx,
        ty = ty,
        bh = bh,
        tc = tc,
        label = label,
    )
}

// ---------------------------------------------------------------------------
// Section title labels
// ---------------------------------------------------------------------------

/// Render a section title `<text>` with `<tspan>`.
pub fn section_title(y: i64, fs: i64, class_idx: usize, label: &str) -> String {
    format!(
        r#"<text dy="0em" x="10" y="{y}" font-size="{fs}" class="sectionTitle sectionTitle{ci}"><tspan alignment-baseline="central" x="10">{label}</tspan></text>"#,
        y = y,
        fs = fs,
        ci = class_idx,
        label = label,
    )
}

// ---------------------------------------------------------------------------
// Today line
// ---------------------------------------------------------------------------

/// Render the today marker `<line>`.
pub fn today_line(tx: i64, top: i64, bot: i64) -> String {
    format!(
        r#"<g class="today"><line x1="{tx}" x2="{tx}" y1="{top}" y2="{bot}" class="today"></line></g>"#,
        tx = tx,
        top = top,
        bot = bot,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the Gantt diagram title `<text>`.
pub fn title_text(cx: i64, ty: i64, title: &str) -> String {
    format!(
        r#"<text x="{cx}" y="{ty}" class="titleText">{title}</text>"#,
        cx = cx,
        ty = ty,
        title = title,
    )
}

// ---------------------------------------------------------------------------
// Empty diagram fallback
// ---------------------------------------------------------------------------

/// Render the empty Gantt fallback SVG.
pub fn empty_svg() -> &'static str {
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50"><text x="10" y="30">Empty Gantt</text></svg>"#
}
