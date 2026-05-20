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
use super::templates::{
    self, arrowhead_marker, box_group, esc, fmt, relation_path, style_block, svg_root,
    swimlane as swimlane_tmpl, title_text,
};
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

    out.push_str(&svg_root(&fmt(canvas_w), &fmt(canvas_h)));

    // Styles
    out.push_str(&style_block(vars.font_family, FONT_SIZE, vars.text_color));

    // Defs: arrowhead marker
    let arrowhead_id = "em-arrowhead";
    let arrowhead_color = vars.line_color;
    out.push_str(&arrowhead_marker(arrowhead_id, arrowhead_color));

    // Swimlane backgrounds
    for sl in &swimlane_layouts {
        let odd_bg = if sl.label.len() % 2 == 0 {
            "rgb(250,250,250)"
        } else {
            "rgb(245,245,248)"
        };
        out.push_str(&swimlane_tmpl(
            &fmt(sl.y),
            &fmt(canvas_w),
            &fmt(sl.height),
            odd_bg,
            &fmt(SWIMLANE_LABEL_X),
            &fmt(sl.y + SWIMLANE_LABEL_Y_OFFSET),
            vars.font_family,
            FONT_SIZE,
            vars.text_color,
            &esc(&sl.label),
        ));
    }

    // Boxes
    for bl in &box_layouts {
        out.push_str(&box_group(
            &fmt(bl.x),
            &fmt(bl.y),
            &fmt(bl.w),
            &fmt(bl.h),
            &bl.stroke,
            &bl.fill,
            &fmt(bl.x + BOX_PADDING),
            &fmt(bl.y + 10.0),
            &fmt(bl.w - BOX_PADDING * 2.0),
            &fmt(bl.h - BOX_PADDING * 2.0),
            vars.font_family,
            FONT_SIZE,
            &esc(&bl.text),
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

        out.push_str(&relation_path(
            "none",
            vars.line_color,
            arrowhead_id,
            &fmt(sx),
            &fmt(sy),
            &fmt(tx),
            &fmt(ty),
        ));
    }

    // Title
    if let Some(t) = &diag.title {
        out.push_str(&title_text(
            &fmt(canvas_w / 2.0),
            vars.font_family,
            FONT_SIZE + 2.0,
            vars.title_color,
            &esc(t),
        ));
    }

    out.push_str("</svg>");
    out
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
    fn snapshot_default_theme() {
        let diag = parser::parse(EM_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
