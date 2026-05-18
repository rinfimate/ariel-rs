// Faithful Rust port of mermaid/src/diagrams/eventmodeling/renderer.ts
//
// The original renderer uses D3.js to draw:
//   - Swimlane background strips (em-swimlane) with labels
//   - Boxes (em-box) per frame, rendered as coloured rectangles with centred text
//   - Relation arrows (em-relation) between boxes with arrowhead marker
//
// Layout:
//   - Each swimlane occupies a horizontal strip; swimlanes are stacked vertically.
//   - Boxes are placed left-to-right within their swimlane.
//   - Relations are drawn as straight lines between box 2/3 and box 1/3 x-positions.
//
// Mirrors the CSS class structure of the original:
//   <g class="em-swimlane">, <g class="em-box">, <path class="em-relation">

use super::constants::*;
use super::parser::EventModelDiagram;
#[allow(unused_imports)]
use super::templates;
use crate::text::measure;
use crate::theme::Theme;
use std::collections::HashMap;

// ─── Computed layout ──────────────────────────────────────────────────────────

struct SwimlaneLayout {
    label: String,
    y: f64,
    height: f64,
}

struct BoxLayout {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    swimlane_y: f64,
    text: String,
    fill: String,
    stroke: String,
}

// ─── Renderer ─────────────────────────────────────────────────────────────────

pub fn render(diag: &EventModelDiagram, theme: Theme) -> String {
    let vars = theme.resolve();

    // Assign column positions to boxes: each box gets column index within its swimlane.
    // Boxes are ordered by their declaration order.
    let mut swimlane_col: HashMap<String, usize> = HashMap::new();

    // Compute box dimensions and positions
    let box_layouts: Vec<BoxLayout> = diag
        .boxes
        .iter()
        .map(|bx| {
            let col = {
                let c = swimlane_col.entry(bx.swimlane.clone()).or_insert(0);
                let v = *c;
                *c += 1;
                v
            };
            let (tw, _) = measure(&bx.text, FONT_SIZE);
            let w = (tw + BOX_PADDING * 2.0 + 4.0).clamp(BOX_MIN_WIDTH, BOX_MAX_WIDTH);
            // Height scaled to text — boxes with more text get taller
            let lines = (bx.text.len() as f64 / 20.0).ceil().max(1.0);
            let h = (lines * (FONT_SIZE + 4.0) + BOX_PADDING * 2.0 + 8.0)
                .clamp(BOX_MIN_HEIGHT, BOX_MAX_HEIGHT);

            // x position: content_start + col * (box_min_width + gap)
            let x = CONTENT_START_X + col as f64 * (BOX_MIN_WIDTH + 16.0);

            BoxLayout {
                x,
                y: 0.0, // will fill in below after swimlane heights known
                w,
                h,
                swimlane_y: 0.0, // will fill in below
                text: bx.text.clone(),
                fill: bx.box_type.fill().to_string(),
                stroke: bx.box_type.stroke().to_string(),
            }
        })
        .collect();

    // Determine each swimlane's height from the maximum box height in it
    let mut swimlane_max_h: HashMap<&str, f64> = HashMap::new();
    for (bx, bl) in diag.boxes.iter().zip(box_layouts.iter()) {
        let entry = swimlane_max_h
            .entry(bx.swimlane.as_str())
            .or_insert(BOX_MIN_HEIGHT);
        *entry = entry.max(bl.h);
    }

    // Build swimlane y positions
    let mut current_y = DIAGRAM_PADDING;
    let swimlane_layouts: Vec<SwimlaneLayout> = diag
        .swimlanes
        .iter()
        .map(|sl| {
            let h = swimlane_max_h
                .get(sl.label.as_str())
                .copied()
                .unwrap_or(BOX_MIN_HEIGHT)
                + SWIMLANE_PADDING * 2.0
                + SWIMLANE_GAP;
            let layout = SwimlaneLayout {
                label: sl.label.clone(),
                y: current_y,
                height: h,
            };
            current_y += h;
            layout
        })
        .collect();

    // Build swimlane label → y lookup
    let sl_y: HashMap<&str, f64> = swimlane_layouts
        .iter()
        .map(|sl| (sl.label.as_str(), sl.y))
        .collect();

    // Now fill in y positions for boxes
    let box_layouts: Vec<BoxLayout> = diag
        .boxes
        .iter()
        .zip(box_layouts)
        .map(|(bx, mut bl)| {
            let sy = sl_y
                .get(bx.swimlane.as_str())
                .copied()
                .unwrap_or(DIAGRAM_PADDING);
            bl.swimlane_y = sy;
            bl.y = sy + SWIMLANE_PADDING;
            bl
        })
        .collect();

    // Build box id → layout lookup for relations
    let box_id_to_idx: HashMap<&str, usize> = diag
        .boxes
        .iter()
        .enumerate()
        .map(|(i, bx)| (bx.id.as_str(), i))
        .collect();

    // Build box label → idx for relation lookups (relations may reference box text)
    let box_label_to_idx: HashMap<&str, usize> = diag
        .boxes
        .iter()
        .enumerate()
        .map(|(i, bx)| (bx.text.as_str(), i))
        .collect();

    let resolve_box_idx = |name: &str| -> Option<usize> {
        box_id_to_idx
            .get(name)
            .copied()
            .or_else(|| box_label_to_idx.get(name).copied())
    };

    // Compute canvas dimensions
    let max_x = box_layouts
        .iter()
        .map(|bl| bl.x + bl.w)
        .fold(CONTENT_START_X, f64::max);
    let canvas_w = (max_x + DIAGRAM_PADDING).max(CONTENT_START_X + 200.0);
    let canvas_h = current_y + DIAGRAM_PADDING;

    let mut out = String::new();

    out.push_str(&format!(
        r#"<svg id="mermaid-eventmodeling" xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" role="graphics-document">"#,
        w = fmt(canvas_w), h = fmt(canvas_h),
    ));

    // Styles
    out.push_str(&format!(
        r#"<style>
#mermaid-eventmodeling {{ font-family: {ff}; font-size: {fs}px; }}
.em-swimlane rect {{ fill-opacity: 0.08; }}
.em-swimlane text {{ font-weight: bold; fill: {tc}; }}
.em-box rect {{ rx: 3; ry: 3; }}
.em-relation {{ stroke-width: 1; }}
</style>"#,
        ff = vars.font_family,
        fs = FONT_SIZE,
        tc = vars.text_color,
    ));

    // Defs: arrowhead marker
    let arrowhead_id = "em-arrowhead";
    let arrowhead_color = vars.line_color;
    out.push_str(&format!(
        r#"<defs><marker id="{mid}" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="{color}"/></marker></defs>"#,
        mid = arrowhead_id,
        color = arrowhead_color,
    ));

    // Swimlane backgrounds
    for sl in &swimlane_layouts {
        let odd_bg = if sl.label.len() % 2 == 0 {
            "rgb(250,250,250)"
        } else {
            "rgb(245,245,248)"
        };
        out.push_str(&format!(
            r#"<g class="em-swimlane"><rect x="0" y="{y}" width="{w}" height="{h}" rx="3" fill="{bg}" stroke="rgb(240,240,240)"/><text font-weight="bold" x="{lx}" y="{ly}" font-family="{ff}" font-size="{fs}" fill="{tc}">{label}</text></g>"#,
            y = fmt(sl.y),
            w = fmt(canvas_w),
            h = fmt(sl.height),
            bg = odd_bg,
            lx = fmt(SWIMLANE_LABEL_X),
            ly = fmt(sl.y + SWIMLANE_LABEL_Y_OFFSET),
            ff = vars.font_family,
            fs = FONT_SIZE,
            tc = vars.text_color,
            label = esc(&sl.label),
        ));
    }

    // Boxes
    for bl in &box_layouts {
        let _text_x = bl.x + bl.w / 2.0;
        let _text_y = bl.y + bl.h / 2.0;
        out.push_str(&format!(
            r#"<g class="em-box"><rect x="{x}" y="{y}" rx="3" width="{w}" height="{h}" stroke="{stroke}" fill="{fill}"/><foreignObject x="{fx}" y="{fy}" width="{fw}" height="{fh}"><div xmlns="http://www.w3.org/1999/xhtml" style="display:table;height:100%;width:100%;"><span style="display:table-cell;text-align:center;vertical-align:middle;font-family:{ff};font-size:{fs}px;color:#fff;">{text}</span></div></foreignObject></g>"#,
            x = fmt(bl.x), y = fmt(bl.y),
            w = fmt(bl.w), h = fmt(bl.h),
            stroke = bl.stroke,
            fill = bl.fill,
            fx = fmt(bl.x + BOX_PADDING),
            fy = fmt(bl.y + 10.0),
            fw = fmt(bl.w - BOX_PADDING * 2.0),
            fh = fmt(bl.h - BOX_PADDING * 2.0),
            ff = vars.font_family,
            fs = FONT_SIZE,
            text = esc(&bl.text),
        ));
    }

    // Relations
    for rel in &diag.relations {
        let src_idx = match resolve_box_idx(&rel.source) {
            Some(i) => i,
            None => continue,
        };
        let tgt_idx = match resolve_box_idx(&rel.target) {
            Some(i) => i,
            None => continue,
        };

        let src = &box_layouts[src_idx];
        let tgt = &box_layouts[tgt_idx];

        // Source x: 2/3 of the way across source box
        let sx = src.x + src.w * 2.0 / 3.0;
        // Target x: 1/3 of the way across target box
        let tx = tgt.x + tgt.w / 3.0;

        // Determine direction: upwards if source y > target y
        let upwards = src.swimlane_y > tgt.swimlane_y;
        let (sy, ty) = if upwards {
            (src.y, tgt.y + tgt.h)
        } else {
            (src.y + src.h, tgt.y)
        };

        out.push_str(&format!(
            r#"<path class="em-relation" fill="{fill}" stroke="{stroke}" stroke-width="1" marker-end="url(#{arrowhead})" d="M{sx} {sy} L{tx} {ty}"/>"#,
            fill = "none",
            stroke = vars.line_color,
            arrowhead = arrowhead_id,
            sx = fmt(sx), sy = fmt(sy),
            tx = fmt(tx), ty = fmt(ty),
        ));
    }

    // Title
    if let Some(t) = &diag.title {
        out.push_str(&format!(
            r#"<text class="em-title" x="{cx}" y="16" text-anchor="middle" font-family="{ff}" font-size="{fs}" fill="{tc}">{text}</text>"#,
            cx = fmt(canvas_w / 2.0),
            ff = vars.font_family,
            fs = FONT_SIZE + 2.0,
            tc = vars.title_color,
            text = esc(t),
        ));
    }

    out.push_str("</svg>");
    out
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn fmt(v: f64) -> String {
    let s = format!("{:.3}", v);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    const EM_BASIC: &str = "eventmodeling\n    title Order Flow\n    swimlane \"Commands\"\n    box cmd1[\"Place Order\"]: command";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(EM_BASIC).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(EM_BASIC).diagram;
        let svg = render(&diag, Theme::Dark);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    #[ignore = "platform-specific float precision — run locally"]
    fn snapshot_default_theme() {
        let diag = parser::parse(EM_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(svg);
    }
}
