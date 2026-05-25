//! SVG template functions for the git graph renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::esc;

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for a git graph.
pub fn svg_root(id: &str, w: f64, h: f64) -> String {
    format!(
        concat!(
            r##"<svg id="{id}" xmlns="http://www.w3.org/2000/svg""##,
            r##" width="100%" height="{h:.1}" viewBox="0 0 {w:.1} {h:.1}""##,
            r##" style="max-width: {w:.1}px;""##,
            r##" role="graphics-document document" aria-roledescription="git-graph">"##
        ),
        id = id,
        w = w,
        h = h,
    )
}

// ---------------------------------------------------------------------------
// Main layout groups
// ---------------------------------------------------------------------------

/// Render the outer translate `<g>` that positions all git content.
pub fn main_translate_group(x: f64, y: f64) -> String {
    format!(
        r##"<g transform="translate({x:.1},{y:.1})">"##,
        x = x,
        y = y,
    )
}

// ---------------------------------------------------------------------------
// Arrow marker
// ---------------------------------------------------------------------------

/// Render the `<defs>` block with arrowhead marker for git graph edges.
pub fn arrowhead_defs(id: &str, line_color: &str) -> String {
    format!(
        "<defs><marker id=\"{id}-arrowhead\" markerWidth=\"10\" markerHeight=\"7\" refX=\"10\" refY=\"3.5\" orient=\"auto\"><polygon points=\"0 0, 10 3.5, 0 7\" fill=\"{line_color}\"/></marker></defs>",
        id = id,
        line_color = line_color,
    )
}

// ---------------------------------------------------------------------------
// Branch rendering (LR mode)
// ---------------------------------------------------------------------------

/// Render a branch spine line for LR (horizontal) mode.
pub fn branch_spine_lr(y: f64, max_pos: f64, line_color: &str) -> String {
    format!(
        "<line x1=\"0\" y1=\"{y:.1}\" x2=\"{mx:.1}\" y2=\"{y:.1}\" stroke=\"{line_color}\" stroke-width=\"1\" stroke-dasharray=\"2\"/>",
        y = y,
        mx = max_pos,
        line_color = line_color,
    )
}

/// Render a branch label background rectangle for LR mode.
/// Height matches Mermaid's `bbox.height + 4` ≈ 21 for 16px Arial.
pub fn branch_label_rect_lr(bx: f64, by: f64, bw: f64, fill: &str) -> String {
    format!(
        concat!(
            r##"<rect x="{bx:.1}" y="{by:.1}" width="{bw:.1}" height="21""##,
            r##" rx="4" ry="4" fill="{f}"/>"##
        ),
        bx = bx,
        by = by,
        bw = bw,
        f = fill,
    )
}

/// Render a branch label text for LR mode.
pub fn branch_label_text_lr(tx: f64, ty: f64, tc: &str, ff: &str, name: &str) -> String {
    format!(
        "<text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"16\" fill=\"{tc}\" font-family=\"{ff}\" text-anchor=\"start\">{name}</text>",
        tx = tx, ty = ty, tc = tc, ff = ff, name = name,
    )
}

// ---------------------------------------------------------------------------
// Branch rendering (TB mode)
// ---------------------------------------------------------------------------

/// Render a branch spine line for TB (vertical) mode.
pub fn branch_spine_tb(x: f64, default_pos: f64, max_pos: f64, stroke: &str) -> String {
    format!(
        concat!(
            r##"<line x1="{x:.1}" y1="{dp:.1}" x2="{x:.1}" y2="{mx:.1}""##,
            r##" stroke="{s}" stroke-width="2" stroke-dasharray="4 2"/>"##
        ),
        x = x,
        dp = default_pos,
        mx = max_pos,
        s = stroke,
    )
}

/// Render a branch label background rectangle for TB mode.
pub fn branch_label_rect_tb(bx: f64, bw: f64, fill: &str, stroke: &str) -> String {
    format!(
        concat!(
            r##"<rect x="{bx:.1}" y="0" width="{bw:.1}" height="20""##,
            r##" rx="4" fill="{f}" stroke="{s}" stroke-width="1"/>"##
        ),
        bx = bx,
        bw = bw,
        f = fill,
        s = stroke,
    )
}

/// Render a branch label text for TB mode.
pub fn branch_label_text_tb(tx: f64, ff: &str, name: &str, text_color: &str) -> String {
    format!(
        "<text x=\"{tx:.1}\" y=\"14\" font-size=\"14\" fill=\"{text_color}\" font-family=\"{ff}\" text-anchor=\"start\">{name}</text>",
        tx = tx,
        ff = ff,
        name = name,
        text_color = text_color,
    )
}

// ---------------------------------------------------------------------------
// Branch rendering (BT mode)
// ---------------------------------------------------------------------------

/// Render a branch spine line for BT (bottom-to-top) mode.
pub fn branch_spine_bt(x: f64, default_pos: f64, max_pos: f64, stroke: &str) -> String {
    format!(
        concat!(
            r##"<line x1="{x:.1}" y1="{dp:.1}" x2="{x:.1}" y2="{mx:.1}""##,
            r##" stroke="{s}" stroke-width="2" stroke-dasharray="4 2"/>"##
        ),
        x = x,
        dp = default_pos,
        mx = max_pos,
        s = stroke,
    )
}

/// Render a branch label background rectangle for BT mode.
pub fn branch_label_rect_bt(bx: f64, my: f64, bw: f64, fill: &str, stroke: &str) -> String {
    format!(
        concat!(
            r##"<rect x="{bx:.1}" y="{my:.1}" width="{bw:.1}" height="20""##,
            r##" rx="4" fill="{f}" stroke="{s}" stroke-width="1"/>"##
        ),
        bx = bx,
        my = my,
        bw = bw,
        f = fill,
        s = stroke,
    )
}

/// Render a branch label text for BT mode.
pub fn branch_label_text_bt(tx: f64, ty: f64, ff: &str, name: &str, text_color: &str) -> String {
    format!(
        "<text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"14\" fill=\"{text_color}\" font-family=\"{ff}\" text-anchor=\"start\">{name}</text>",
        tx = tx,
        ty = ty,
        ff = ff,
        name = name,
        text_color = text_color,
    )
}

// ---------------------------------------------------------------------------
// Arrow path
// ---------------------------------------------------------------------------

/// Render an arrow connecting two commits.
pub fn commit_arrow(d: &str, ci: usize, stroke: &str, branch_idx: usize) -> String {
    format!(
        "<path d=\"{d}\" class=\"arrow arrow{ci}\" fill=\"none\" stroke=\"{s}\" stroke-width=\"8\" stroke-linecap=\"round\"/>",
        d = d, s = stroke, ci = branch_idx % ci, // note: ci arg is THEME_COLOR_LIMIT
    )
}

// ---------------------------------------------------------------------------
// Commit bullets
// ---------------------------------------------------------------------------

/// Render a highlight commit outer rectangle.
pub fn commit_highlight_outer(x: f64, y: f64, fill: &str, stroke: &str) -> String {
    format!(
        r##"<rect x="{:.1}" y="{:.1}" width="20" height="20" fill="{}" stroke="{}" stroke-width="2"/>"##,
        x, y, fill, stroke,
    )
}

/// Render a highlight commit inner rectangle.
pub fn commit_highlight_inner(x: f64, y: f64, fill: &str, stroke: &str) -> String {
    format!(
        r##"<rect x="{:.1}" y="{:.1}" width="12" height="12" fill="{}" stroke="{}" stroke-width="1"/>"##,
        x, y, fill, stroke,
    )
}

/// Render a normal/merge/reverse commit circle.
pub fn commit_circle(cx: f64, cy: f64, r: f64, fill: &str, stroke: &str, id: &str) -> String {
    format!(
        r##"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{}" stroke="{}" stroke-width="2" class="commit commit-{}"/>"##,
        cx, cy, r, fill, stroke, id,
    )
}

/// Render the merge inner circle overlay using the diagram background (primary_color).
pub fn commit_merge_inner(cx: f64, cy: f64, id: &str, primary_color: &str) -> String {
    format!(
        "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"6\" class=\"commit-merge commit{}\" stroke=\"{primary_color}\" fill=\"{primary_color}\"/>",
        cx, cy, id, primary_color = primary_color,
    )
}

/// Render the reverse commit X-cross path.
pub fn commit_reverse_cross(cx: f64, cy: f64, c: f64) -> String {
    format!(
        "<path d=\"M {:.1},{:.1}L{:.1},{:.1}M{:.1},{:.1}L{:.1},{:.1}\" stroke=\"#fff\" stroke-width=\"2\"/>",
        cx - c, cy - c, cx + c, cy + c,
        cx - c, cy + c, cx + c, cy - c,
    )
}

/// Render a cherry-pick commit base circle.
pub fn commit_cherry_circle(cx: f64, cy: f64, r: f64, fill: &str, stroke: &str) -> String {
    format!(
        r##"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{}" stroke="{}" stroke-width="2"/>"##,
        cx, cy, r, fill, stroke,
    )
}

/// Render a cherry-pick eye circle (white inner dot).
pub fn commit_cherry_eye(cx: f64, cy: f64) -> String {
    format!(
        "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"2.75\" fill=\"#fff\"/>",
        cx, cy,
    )
}

/// Render a cherry-pick stem line.
pub fn commit_cherry_stem(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    format!(
        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#fff\" stroke-width=\"1.5\"/>",
        x1, y1, x2, y2,
    )
}

// ---------------------------------------------------------------------------
// Commit labels
// ---------------------------------------------------------------------------

/// Render the commit label group wrapper (LR mode, rotated).
pub fn commit_label_group_lr(tx: f64, ty: f64, rx: f64, ry: f64) -> String {
    format!(
        "<g transform=\"translate({tx:.3},{ty:.3}) rotate(-45,{rx:.3},{ry:.3})\">",
        tx = tx,
        ty = ty,
        rx = rx,
        ry = ry,
    )
}

/// Render a commit label background rectangle.
pub fn commit_label_bkg(rect_x: f64, rect_y: f64, rect_w: f64, secondary_color: &str) -> String {
    format!(
        "<rect class=\"commit-label-bkg\" x=\"{:.3}\" y=\"{:.3}\" width=\"{:.3}\" height=\"15\" fill=\"{secondary_color}\" opacity=\"0.5\"/>",
        rect_x, rect_y, rect_w, secondary_color = secondary_color,
    )
}

/// Render the commit label text.
pub fn commit_label_text(
    text_x: f64,
    text_y: f64,
    label: &str,
    text_color: &str,
    ff: &str,
) -> String {
    format!(
        "<text x=\"{:.3}\" y=\"{:.3}\" font-family=\"{ff}\" font-size=\"10\" fill=\"{text_color}\" class=\"commit-label\">{}</text>",
        text_x, text_y, label, text_color = text_color,
    )
}

/// Render a commit label background rectangle (TB/BT mode).
pub fn commit_label_bkg_tb(lx: f64, ly: f64, lw: f64, lh: f64, secondary_color: &str) -> String {
    format!(
        "<rect class=\"commit-label-bkg\" x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{secondary_color}\" opacity=\"0.5\"/>",
        lx, ly, lw, lh, secondary_color = secondary_color,
    )
}

/// Render a commit label text (TB/BT mode).
pub fn commit_label_text_tb(lx: f64, ly: f64, ff: &str, label: &str, text_color: &str) -> String {
    format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"{text_color}\" font-family=\"{ff}\" class=\"commit-label\">{}</text>",
        lx, ly, label, ff = ff, text_color = text_color,
    )
}

// ---------------------------------------------------------------------------
// Tag badges (LR mode)
// ---------------------------------------------------------------------------

/// Render a tag badge polygon (price-tag shape) for LR mode.
#[allow(clippy::too_many_arguments)]
pub fn tag_badge_polygon(
    x0: f64,
    yb: f64,
    yt: f64,
    bl: f64,
    br: f64,
    bt: f64,
    bb: f64,
    primary_color: &str,
    primary_border: &str,
) -> String {
    format!(
        "<polygon class=\"tag-label-bkg\" points=\"{x0:.3},{yb:.3} {x0:.3},{yt:.3} {bl:.3},{bt:.3} {br:.3},{bt:.3} {br:.3},{bb:.3} {bl:.3},{bb:.3}\" fill=\"{primary_color}\" stroke=\"{primary_border}\" stroke-width=\"1\"/>",
        x0 = x0, yb = yb, yt = yt, bl = bl, br = br, bt = bt, bb = bb,
        primary_color = primary_color, primary_border = primary_border,
    )
}

/// Render the hole circle on a tag badge.
pub fn tag_hole_circle(cy: f64, cx: f64, line_color: &str) -> String {
    format!(
        "<circle cy=\"{:.3}\" cx=\"{:.3}\" r=\"1.5\" class=\"tag-hole\" fill=\"{line_color}\"/>",
        cy,
        cx,
        line_color = line_color,
    )
}

/// Render the tag text label.
pub fn tag_text_lr(y: f64, x: f64, ff: &str, text: &str, text_color: &str) -> String {
    format!(
        "<text y=\"{:.3}\" class=\"tag-label\" x=\"{:.3}\" font-size=\"10\" fill=\"{text_color}\" font-family=\"{ff}\">{}</text>",
        y, x, text, ff = ff, text_color = text_color,
    )
}

// ---------------------------------------------------------------------------
// Tag badges (TB/BT mode)
// ---------------------------------------------------------------------------

/// Render a tag badge rectangle for TB/BT mode.
pub fn tag_badge_rect_tb(tx: f64, ty: f64, tw: f64) -> String {
    format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"14\" rx=\"2\" fill=\"#ffffe0\" stroke=\"#cc9900\" stroke-width=\"1\"/>",
        tx - 2.0, ty - 7.0, tw + 4.0,
    )
}

/// Render a tag text label for TB/BT mode.
pub fn tag_text_tb(tx: f64, ty: f64, ff: &str, text: &str, text_color: &str) -> String {
    format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"{text_color}\" font-family=\"{ff}\">{}</text>",
        tx, ty, text, ff = ff, text_color = text_color,
    )
}
