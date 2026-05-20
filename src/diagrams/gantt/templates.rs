//! SVG template functions for the Gantt chart renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

use crate::theme::ThemeVars;

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::esc;

pub fn escape_id(s: &str) -> String {
    s.replace(' ', "-")
}

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

pub fn build_style(id: &str, vars: &ThemeVars) -> String {
    let ff = vars.font_family;
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
            "#{id} .mermaid-main-font{{font-family:{ff};}}",
            "#{id} .exclude-range{{fill:#eeeeee;}}",
            "#{id} .section{{stroke:none;opacity:0.2;}}",
            "#{id} .section0{{fill:rgba(102, 102, 255, 0.49);}}",
            "#{id} .section2{{fill:#fff400;}}",
            "#{id} .section1,#{id} .section3{{fill:white;opacity:0.2;}}",
            "#{id} .sectionTitle0{{fill:#333;}}",
            "#{id} .sectionTitle1{{fill:#333;}}",
            "#{id} .sectionTitle2{{fill:#333;}}",
            "#{id} .sectionTitle3{{fill:#333;}}",
            "#{id} .sectionTitle{{text-anchor:start;font-family:{ff};}}",
            "#{id} .grid .tick{{stroke:lightgrey;opacity:0.8;shape-rendering:crispEdges;}}",
            "#{id} .grid .tick text{{font-family:{ff};fill:#333;}}",
            "#{id} .grid path{{stroke-width:0;}}",
            "#{id} .today{{fill:none;stroke:red;stroke-width:2px;}}",
            "#{id} .task{{stroke-width:2;}}",
            "#{id} .taskText{{text-anchor:middle;font-family:{ff};}}",
            "#{id} .taskTextOutsideRight{{fill:black;text-anchor:start;font-family:{ff};}}",
            "#{id} .taskTextOutsideLeft{{fill:black;text-anchor:end;}}",
            "#{id} .task.clickable{{cursor:pointer;}}",
            "#{id} .taskText.clickable{{cursor:pointer;fill:#003163!important;font-weight:bold;}}",
            "#{id} .taskTextOutsideLeft.clickable{{cursor:pointer;fill:#003163!important;font-weight:bold;}}",
            "#{id} .taskTextOutsideRight.clickable{{cursor:pointer;fill:#003163!important;font-weight:bold;}}",
            "#{id} .taskText0,#{id} .taskText1,#{id} .taskText2,#{id} .taskText3{{fill:white;}}",
            "#{id} .task0,#{id} .task1,#{id} .task2,#{id} .task3{{fill:#8a90dd;stroke:#534fbc;}}",
            "#{id} .taskTextOutside0,#{id} .taskTextOutside2{{fill:black;}}",
            "#{id} .taskTextOutside1,#{id} .taskTextOutside3{{fill:black;}}",
            "#{id} .active0,#{id} .active1,#{id} .active2,#{id} .active3{{fill:#bfc7ff;stroke:#534fbc;}}",
            "#{id} .activeText0,#{id} .activeText1,#{id} .activeText2,#{id} .activeText3{{fill:black!important;}}",
            "#{id} .done0,#{id} .done1,#{id} .done2,#{id} .done3{{stroke:grey;fill:lightgrey;stroke-width:2;}}",
            "#{id} .doneText0,#{id} .doneText1,#{id} .doneText2,#{id} .doneText3{{fill:black!important;}}",
            "#{id} .doneText0.taskTextOutsideLeft,#{id} .doneText0.taskTextOutsideRight,",
            "#{id} .doneText1.taskTextOutsideLeft,#{id} .doneText1.taskTextOutsideRight,",
            "#{id} .doneText2.taskTextOutsideLeft,#{id} .doneText2.taskTextOutsideRight,",
            "#{id} .doneText3.taskTextOutsideLeft,#{id} .doneText3.taskTextOutsideRight{{fill:black!important;}}",
            "#{id} .crit0,#{id} .crit1,#{id} .crit2,#{id} .crit3{{stroke:#ff8888;fill:red;stroke-width:2;}}",
            "#{id} .activeCrit0,#{id} .activeCrit1,#{id} .activeCrit2,#{id} .activeCrit3{{stroke:#ff8888;fill:#bfc7ff;stroke-width:2;}}",
            "#{id} .doneCrit0,#{id} .doneCrit1,#{id} .doneCrit2,#{id} .doneCrit3{{stroke:#ff8888;fill:lightgrey;stroke-width:2;cursor:pointer;shape-rendering:crispEdges;}}",
            "#{id} .milestone{{transform:rotate(45deg) scale(0.8,0.8);}}",
            "#{id} .milestoneText{{font-style:italic;}}",
            "#{id} .doneCritText0,#{id} .doneCritText1,#{id} .doneCritText2,#{id} .doneCritText3{{fill:black!important;}}",
            "#{id} .doneCritText0.taskTextOutsideLeft,#{id} .doneCritText0.taskTextOutsideRight,",
            "#{id} .doneCritText1.taskTextOutsideLeft,#{id} .doneCritText1.taskTextOutsideRight,",
            "#{id} .doneCritText2.taskTextOutsideLeft,#{id} .doneCritText2.taskTextOutsideRight,",
            "#{id} .doneCritText3.taskTextOutsideLeft,#{id} .doneCritText3.taskTextOutsideRight{{fill:black!important;}}",
            "#{id} .vert{{stroke:navy;}}",
            "#{id} .vertText{{font-size:15px;text-anchor:middle;fill:navy!important;}}",
            "#{id} .activeCritText0,#{id} .activeCritText1,#{id} .activeCritText2,#{id} .activeCritText3{{fill:black!important;}}",
            "#{id} .titleText{{text-anchor:middle;font-size:18px;fill:#333;font-family:{ff};}}",
            "#{id} :root{{--mermaid-font-family:{ff};}}",
        ),
        id = id,
        ff = ff,
    )
}

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
