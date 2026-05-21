// Faithful Rust port of mermaid/src/diagrams/treemap/renderer.ts
//
// Layout: D3 treemap with squarify tiling (phi ratio), round=true.
// Canvas: nodeWidth*SECTION_INNER_PADDING × nodeHeight*SECTION_INNER_PADDING
//         = 100×10 × 40×10 = 1000 × 400 (default config values).
// Padding: paddingInner=10, paddingTop=35 (sections), paddingLeft/Right/Bottom=10 (sections).
// Colors: ordinal scale keyed by node name; built by pre-order traversal.
// ViewBox: getBBox of visible elements + diagramPadding=8.

use super::constants::*;
use super::parser::{TreemapDiagram, TreemapNode};
use super::templates::{self, esc, fmt_f, fmt_i};
use crate::theme::Theme;

// ─── D3 hierarchy node ───────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct HNode {
    name: String,
    value: f64,
    depth: usize,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    children: Vec<HNode>,
    /// Original parsed value (for display)
    raw_value: Option<f64>,
}

impl HNode {
    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn w(&self) -> f64 {
        self.x1 - self.x0
    }

    fn h(&self) -> f64 {
        self.y1 - self.y0
    }
}

// ─── Build hierarchy from parsed nodes ───────────────────────────────────────

fn build_hnode(node: &TreemapNode, depth: usize) -> HNode {
    let children: Vec<HNode> = node
        .children
        .iter()
        .map(|c| build_hnode(c, depth + 1))
        .collect();
    let value = sum_value(node);
    HNode {
        name: node.name.clone(),
        value,
        depth,
        x0: 0.0,
        y0: 0.0,
        x1: 0.0,
        y1: 0.0,
        children,
        raw_value: node.value,
    }
}

fn sum_value(node: &TreemapNode) -> f64 {
    if node.children.is_empty() {
        node.value.unwrap_or(0.0).max(0.0)
    } else {
        node.children.iter().map(sum_value).sum::<f64>().max(0.0)
    }
}

// ─── D3 squarify tiling ───────────────────────────────────────────────────────
//
// Direct port of d3-hierarchy/src/treemap/squarify.js (squarifyRatio).
// Mutates node.children[i].{x0,y0,x1,y1} in place.

fn tile_squarify(parent: &mut HNode, x0: f64, y0: f64, x1: f64, y1: f64) {
    let n = parent.children.len();
    if n == 0 {
        return;
    }

    // Sort children descending by value (D3 .sort() call before treemap)
    parent.children.sort_by(|a, b| {
        b.value
            .partial_cmp(&a.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    squarify_ratio(PHI, parent, x0, y0, x1, y1);
}

fn squarify_ratio(ratio: f64, parent: &mut HNode, x0: f64, y0: f64, x1: f64, y1: f64) {
    let n = parent.children.len();
    let total_value = parent.value;
    if total_value <= 0.0 || n == 0 {
        return;
    }

    let mut x0 = x0;
    let mut y0 = y0;
    let mut remaining = total_value;
    let mut i0: usize = 0;

    while i0 < n {
        let dx = x1 - x0;
        let dy = y1 - y0;
        if dx <= 0.0 || dy <= 0.0 {
            break;
        }

        // Find first non-zero (mirrors D3 do-while loop)
        let mut i1 = i0;
        let mut sum_val;
        loop {
            sum_val = parent.children[i1].value;
            i1 += 1;
            if sum_val != 0.0 || i1 >= n {
                break;
            }
        }
        if sum_val == 0.0 {
            break;
        }

        let mut min_val = sum_val;
        let mut max_val = sum_val;
        let alpha = f64::max(dy / dx, dx / dy) / (remaining * ratio);
        let mut beta = sum_val * sum_val * alpha;
        let mut min_ratio = f64::max(max_val / beta, beta / min_val);

        // Extend row
        while i1 < n {
            let node_val = parent.children[i1].value;
            sum_val += node_val;
            if node_val < min_val {
                min_val = node_val;
            }
            if node_val > max_val {
                max_val = node_val;
            }
            beta = sum_val * sum_val * alpha;
            let new_ratio = f64::max(max_val / beta, beta / min_val);
            if new_ratio > min_ratio {
                sum_val -= node_val;
                break;
            }
            min_ratio = new_ratio;
            i1 += 1;
        }

        // Commit row: children[i0..i1], sum_val
        let is_dice = dx < dy;
        if is_dice {
            // dice: distribute along x, full y band
            let y1_row = if remaining > 0.0 {
                y0 + dy * sum_val / remaining
            } else {
                y1
            };
            let mut cur_x = x0;
            for idx in i0..i1 {
                let v = parent.children[idx].value;
                let nx1 = cur_x + v * (dx / sum_val);
                parent.children[idx].x0 = cur_x;
                parent.children[idx].y0 = y0;
                parent.children[idx].x1 = nx1;
                parent.children[idx].y1 = y1_row;
                cur_x = nx1;
            }
            // Advance y0
            y0 = y1_row;
        } else {
            // slice: distribute along y, x band = x0..x0+dx*sum/remaining
            let x1_band = if remaining > 0.0 {
                x0 + dx * sum_val / remaining
            } else {
                x1
            };
            let k_slice = if sum_val > 0.0 {
                (y1 - y0) / sum_val
            } else {
                0.0
            };
            let mut cur_y = y0;
            for idx in i0..i1 {
                let v = parent.children[idx].value;
                let ny1 = cur_y + v * k_slice;
                parent.children[idx].x0 = x0;
                parent.children[idx].y0 = cur_y;
                parent.children[idx].x1 = x1_band;
                parent.children[idx].y1 = ny1;
                cur_y = ny1;
            }
            // Advance x0
            x0 = x1_band;
        }

        remaining -= sum_val;
        i0 = i1;
    }
    let _ = remaining; // suppress warning
}

// ─── D3 treemap positionNode ──────────────────────────────────────────────────

fn position_node(node: &mut HNode, depth: usize, padding_stack: &mut Vec<f64>) {
    // Ensure padding_stack is large enough
    while padding_stack.len() <= depth {
        padding_stack.push(0.0);
    }

    let p = padding_stack[depth];
    node.x0 += p;
    node.y0 += p;
    node.x1 -= p;
    node.y1 -= p;

    // Clamp (x0 <= x1, y0 <= y1)
    if node.x1 < node.x0 {
        node.x0 = (node.x0 + node.x1) / 2.0;
        node.x1 = node.x0;
    }
    if node.y1 < node.y0 {
        node.y0 = (node.y0 + node.y1) / 2.0;
        node.y1 = node.y0;
    }

    if !node.children.is_empty() {
        // p for children = paddingInner / 2 = 5
        let child_p = PADDING_INNER / 2.0;

        // Update padding_stack for depth+1
        while padding_stack.len() <= depth + 1 {
            padding_stack.push(0.0);
        }
        padding_stack[depth + 1] = child_p;

        // Compute tiling area (inner content area for children)
        let (padding_left, padding_top, padding_right, padding_bottom) = (
            SECTION_PADDING - child_p,
            SECTION_HEADER_HEIGHT + SECTION_PADDING - child_p,
            SECTION_PADDING - child_p,
            SECTION_PADDING - child_p,
        );

        let mut cx0 = node.x0 + padding_left;
        let mut cy0 = node.y0 + padding_top;
        let mut cx1 = node.x1 - padding_right;
        let mut cy1 = node.y1 - padding_bottom;

        if cx1 < cx0 {
            cx0 = (cx0 + cx1) / 2.0;
            cx1 = cx0;
        }
        if cy1 < cy0 {
            cy0 = (cy0 + cy1) / 2.0;
            cy1 = cy0;
        }

        // Initialize children coordinates to the parent's tiling area
        for child in node.children.iter_mut() {
            child.x0 = cx0;
            child.y0 = cy0;
            child.x1 = cx1;
            child.y1 = cy1;
        }

        // Tile children via squarify
        tile_squarify(node, cx0, cy0, cx1, cy1);

        // Recurse into each child
        let children = std::mem::take(&mut node.children);
        let mut positioned_children = Vec::with_capacity(children.len());
        for mut child in children {
            position_node(&mut child, depth + 1, padding_stack);
            positioned_children.push(child);
        }
        node.children = positioned_children;
    }
}

// ─── Round all coordinates ───────────────────────────────────────────────────

fn round_node(node: &mut HNode) {
    node.x0 = node.x0.round();
    node.y0 = node.y0.round();
    node.x1 = node.x1.round();
    node.y1 = node.y1.round();
    for child in node.children.iter_mut() {
        round_node(child);
    }
}

// ─── Ordinal color scale ──────────────────────────────────────────────────────

struct OrdinalScale {
    domain: Vec<String>,
    range: Vec<&'static str>,
}

impl OrdinalScale {
    fn new(range: Vec<&'static str>) -> Self {
        OrdinalScale {
            domain: Vec::new(),
            range,
        }
    }

    fn get(&mut self, name: &str) -> &'static str {
        if let Some(pos) = self.domain.iter().position(|n| n == name) {
            let idx = pos % self.range.len();
            self.range[idx]
        } else {
            let pos = self.domain.len();
            self.domain.push(name.to_string());
            let idx = pos % self.range.len();
            self.range[idx]
        }
    }
}

// ─── Label-color ordinal scale ────────────────────────────────────────────────

struct LabelScale {
    domain: Vec<String>,
    theme: Theme,
}

impl LabelScale {
    fn new(theme: Theme) -> Self {
        LabelScale {
            domain: Vec::new(),
            theme,
        }
    }

    fn get(&mut self, name: &str) -> &'static str {
        if let Some(pos) = self.domain.iter().position(|n| n == name) {
            theme_c_scale_label(self.theme, pos)
        } else {
            let pos = self.domain.len();
            self.domain.push(name.to_string());
            theme_c_scale_label(self.theme, pos)
        }
    }
}

// ─── Main render function ─────────────────────────────────────────────────────

pub fn render(diag: &TreemapDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let title_h = if diag.title.is_some() {
        TITLE_HEIGHT
    } else {
        0.0
    };

    // Build virtual root
    let mut root = HNode {
        name: String::new(),
        value: diag.roots.iter().map(sum_value).sum(),
        depth: 0,
        x0: 0.0,
        y0: title_h,
        x1: CANVAS_W,
        y1: CANVAS_H + title_h,
        children: diag.roots.iter().map(|n| build_hnode(n, 1)).collect(),
        raw_value: None,
    };

    if root.value <= 0.0 {
        // Empty diagram
        return templates::svg_root_empty(
            CANVAS_W as i64,
            -(DIAGRAM_PADDING as i64),
            -(DIAGRAM_PADDING as i64),
            (CANVAS_W + 2.0 * DIAGRAM_PADDING) as i64,
            (CANVAS_H + 2.0 * DIAGRAM_PADDING) as i64,
        );
    }

    // D3 treemap layout
    let mut padding_stack: Vec<f64> = vec![0.0];
    position_node(&mut root, 0, &mut padding_stack);

    // Round all coordinates (round=true in D3 treemap)
    round_node(&mut root);

    // Build color scales (theme-aware)
    // colorScale range: ["transparent", cScale0, cScale1, ..., cScale11]
    let mut color_scale = {
        let mut range: Vec<&'static str> = vec!["transparent"];
        range.extend_from_slice(theme_c_scale(theme));
        OrdinalScale::new(range)
    };
    // colorScalePeer range: ["transparent", cScalePeer0, ...]
    let mut color_scale_peer = {
        let mut range: Vec<&'static str> = vec!["transparent"];
        range.extend_from_slice(theme_c_scale_peer(theme));
        OrdinalScale::new(range)
    };
    let mut color_scale_label = LabelScale::new(theme);

    // Prime the color scale for the virtual root (so "" → transparent)
    color_scale.get(&root.name);
    color_scale_peer.get(&root.name);

    // Collect sections (branch nodes, depth>=0) in pre-order
    let sections = collect_branches(&root);
    // Collect leaves in pre-order
    let leaves = collect_leaves(&root);

    // Compute visible bounding box for viewBox
    let mut bbox_x_min = f64::MAX;
    let mut bbox_y_min = f64::MAX;
    let mut bbox_x_max = f64::MIN;
    let mut bbox_y_max = f64::MIN;

    for s in &sections {
        if s.depth > 0 {
            bbox_x_min = bbox_x_min.min(s.x0);
            bbox_y_min = bbox_y_min.min(s.y0);
            bbox_x_max = bbox_x_max.max(s.x1);
            bbox_y_max = bbox_y_max.max(s.y1);
        }
    }
    for l in &leaves {
        bbox_x_min = bbox_x_min.min(l.x0);
        bbox_y_min = bbox_y_min.min(l.y0);
        bbox_x_max = bbox_x_max.max(l.x1);
        bbox_y_max = bbox_y_max.max(l.y1);
    }

    if bbox_x_min == f64::MAX {
        // All leaves at root level - fallback
        bbox_x_min = 0.0;
        bbox_y_min = 0.0;
        bbox_x_max = CANVAS_W;
        bbox_y_max = CANVAS_H;
    }

    let vb_x = bbox_x_min - DIAGRAM_PADDING;
    let vb_y = bbox_y_min - DIAGRAM_PADDING;
    let vb_w = (bbox_x_max - bbox_x_min) + DIAGRAM_PADDING * 2.0;
    let vb_h = (bbox_y_max - bbox_y_min) + DIAGRAM_PADDING * 2.0;

    let max_w = vb_w;

    let mut out = String::new();

    // SVG root
    out.push_str(&templates::svg_root(
        fmt_i(max_w),
        fmt_i(vb_x),
        fmt_i(vb_y),
        fmt_i(vb_w),
        fmt_i(vb_h),
    ));

    // Title
    if let Some(t) = &diag.title {
        out.push_str(&templates::title_text(
            &fmt_f(CANVAS_W / 2.0),
            &fmt_f(title_h / 2.0),
            vars.primary_text,
            &esc(t),
        ));
    }

    // Container group (mirrors g.append("g").attr("transform", `translate(0, ${titleHeight})`))
    out.push_str(&templates::container_group_open(&fmt_f(title_h)));

    // ── Render sections (branch nodes in pre-order, including depth=0 root) ──

    for (sec_idx, sec) in sections.iter().enumerate() {
        let sec_w = sec.w();
        let sec_h = sec.h();

        // Compute fill/stroke colors for this section
        let fill_color = color_scale.get(&sec.name);
        let stroke_color = color_scale_peer.get(&sec.name);

        // Section label color (only called for depth > 0)
        let label_color = if sec.depth > 0 {
            color_scale_label.get(&sec.name)
        } else {
            ""
        };

        // Group with transform
        out.push_str(&templates::section_group_open(
            &fmt_f(sec.x0),
            &fmt_f(sec.y0),
        ));

        // Section header rect
        let header_display = if sec.depth == 0 { "display: none;" } else { "" };
        out.push_str(&templates::section_header_rect(
            &fmt_f(sec_w),
            &fmt_f(SECTION_HEADER_HEIGHT),
            header_display,
        ));

        // Clip path for section header (width - SECTION_CLIP_MARGIN)
        out.push_str(&templates::section_clip_path(
            sec_idx,
            &fmt_f(f64::max(0.0, sec_w - SECTION_CLIP_MARGIN)),
            &fmt_f(SECTION_HEADER_HEIGHT),
        ));

        // Full section background rect
        let section_display = if sec.depth == 0 {
            "display: none;".to_string()
        } else {
            String::new()
        };
        out.push_str(&templates::section_rect(
            &fmt_f(sec_w),
            &fmt_f(sec_h),
            sec_idx,
            fill_color,
            stroke_color,
            &section_display,
        ));

        // Section label text
        let section_label_style = if sec.depth == 0 {
            "display: none;".to_string()
        } else {
            format!(
                "dominant-baseline: middle; font-size: 12px; fill:{}; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                label_color
            )
        };
        let section_label_text = if sec.depth == 0 {
            String::new()
        } else {
            esc(&sec.name)
        };
        out.push_str(&templates::section_label_text(
            &fmt_f(SECTION_HEADER_HEIGHT / 2.0),
            &section_label_style,
            &section_label_text,
        ));

        // Section value text (showValues=true by default)
        let section_value = sec.value;
        let section_value_str = format_value(section_value);
        let section_value_style = if sec.depth == 0 {
            "display: none;".to_string()
        } else {
            format!(
                "text-anchor: end; dominant-baseline: middle; font-size: 10px; fill:{}; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                label_color
            )
        };
        out.push_str(&templates::section_value_text(
            &fmt_f(sec_w - SECTION_VALUE_X_MARGIN),
            &fmt_f(SECTION_HEADER_HEIGHT / 2.0),
            &section_value_style,
            &section_value_str,
        ));

        out.push_str("</g>");
    }

    // ── Render leaf nodes ──────────────────────────────────────────────────────

    for (leaf_idx, leaf) in leaves.iter().enumerate() {
        let leaf_w = leaf.w();
        let leaf_h = leaf.h();

        // Find parent name for fill color
        let parent_name = find_parent_name(&root, &leaf.name, leaf.x0, leaf.y0);
        let leaf_fill = color_scale.get(parent_name.as_deref().unwrap_or(""));

        // Leaf label color: use each leaf's own name so siblings get distinct
        // scale indices (not the shared parent index).
        // - Default/Neutral: cycle cScaleLabel per leaf name — produces the
        //   correct mix of white/black (Default) or #F4F4F4/#333 (Neutral).
        // - Forest/Dark: always use text_color; cScaleLabel[0/3] = white which
        //   would be invisible on light green/transparent backgrounds.
        let leaf_label_color = match theme {
            Theme::Default | Theme::Neutral | Theme::Dark => color_scale_label.get(&leaf.name),
            _ => vars.text_color,
        };

        out.push_str(&templates::leaf_group_open(
            leaf_idx,
            &fmt_f(leaf.x0),
            &fmt_f(leaf.y0),
        ));

        // Leaf background rect
        out.push_str(&templates::leaf_rect(
            &fmt_f(leaf_w),
            &fmt_f(leaf_h),
            leaf_fill,
        ));

        // Clip path (width - LEAF_CLIP_MARGIN, height - LEAF_CLIP_MARGIN)
        out.push_str(&templates::leaf_clip_path(
            leaf_idx,
            &fmt_f(f64::max(0.0, leaf_w - LEAF_CLIP_MARGIN)),
            &fmt_f(f64::max(0.0, leaf_h - LEAF_CLIP_MARGIN)),
        ));

        // Label text
        let center_x = leaf_w / 2.0;
        let center_y = leaf_h / 2.0;

        // Check if tile is large enough (TILE_INNER_PAD each side)
        let avail_w = leaf_w - TILE_INNER_PAD * 2.0;
        let avail_h = leaf_h - TILE_INNER_PAD * 2.0;

        if avail_w >= MIN_TILE_AVAIL && avail_h >= MIN_TILE_AVAIL {
            // Mirror JS: start at MAX_LABEL_FONT (38px) and decrement while text is too wide.
            // JS: let currentLabelFontSize = 38; while (getComputedTextLength() > availableWidth) currentLabelFontSize--;
            let min_label_font = 8_u32;
            let mut label_font_px = MAX_LABEL_FONT as u32;
            loop {
                if label_font_px <= min_label_font {
                    break;
                }
                let (tw, _) = crate::text::measure(&leaf.name, label_font_px as f64);
                let tw = tw * 1.117; // browser (Arial) ~11.7% wider than Liberation Sans (matches LABEL_SCALE)
                if tw <= avail_w {
                    break;
                }
                label_font_px -= 1;
            }
            let label_font = label_font_px as f64;

            out.push_str(&templates::leaf_label_text(
                &fmt_f(center_x),
                &fmt_f(center_y),
                label_font as u32,
                leaf_label_color,
                leaf_idx,
                &esc(&leaf.name),
            ));

            // Value text (showValues=true)
            if let Some(v) = leaf.raw_value {
                let value_y = center_y + label_font / 2.0 + VALUE_Y_GAP;
                let val_font = ((label_font * VALUE_FONT_SCALE).round() as u32).max(VALUE_FONT_MIN);
                let val_str = format_value(v);
                out.push_str(&templates::leaf_value_text(
                    &fmt_f(center_x),
                    &fmt_f(value_y),
                    val_font,
                    leaf_label_color,
                    leaf_idx,
                    &esc(&val_str),
                ));
            }
        }

        out.push_str("</g>");
    }

    out.push_str("</g>"); // close treemapContainer
    out.push_str("</svg>");
    out
}

// ─── Helper: collect branch nodes in pre-order ───────────────────────────────

fn collect_branches(node: &HNode) -> Vec<&HNode> {
    let mut result = Vec::new();
    collect_branches_inner(node, &mut result);
    result
}

fn collect_branches_inner<'a>(node: &'a HNode, result: &mut Vec<&'a HNode>) {
    if !node.is_leaf() {
        result.push(node);
        // Children were sorted desc by value; iterate in that order
        for child in &node.children {
            collect_branches_inner(child, result);
        }
    }
}

fn collect_leaves(node: &HNode) -> Vec<&HNode> {
    let mut result = Vec::new();
    collect_leaves_inner(node, &mut result);
    result
}

fn collect_leaves_inner<'a>(node: &'a HNode, result: &mut Vec<&'a HNode>) {
    if node.is_leaf() {
        result.push(node);
    } else {
        for child in &node.children {
            collect_leaves_inner(child, result);
        }
    }
}

// ─── Helper: find parent name for a leaf ─────────────────────────────────────

fn find_parent_name(root: &HNode, leaf_name: &str, lx0: f64, ly0: f64) -> Option<String> {
    find_parent_inner(root, leaf_name, lx0, ly0)
}

fn find_parent_inner(node: &HNode, leaf_name: &str, lx0: f64, ly0: f64) -> Option<String> {
    for child in &node.children {
        if child.is_leaf() && child.name == leaf_name && child.x0 == lx0 && child.y0 == ly0 {
            return Some(node.name.clone());
        }
        if let Some(found) = find_parent_inner(child, leaf_name, lx0, ly0) {
            return Some(found);
        }
    }
    None
}

/// Format a value with comma thousand separators (d3-format(","))
fn format_value(v: f64) -> String {
    if v == v.floor() && v.is_finite() {
        format_with_commas(v as i64)
    } else {
        format!("{:.2}", v)
    }
}

fn format_with_commas(n: i64) -> String {
    let s = format!("{}", n.abs());
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    if n < 0 {
        result.push('-');
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    const TREEMAP_BASIC: &str =
        "treemap-beta\n  \"Documents\": 500\n  \"Pictures\": 800\n  \"Videos\": 1200";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(TREEMAP_BASIC).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "missing <svg tag");
        assert!(svg.contains("Documents"), "missing treemap item");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(TREEMAP_BASIC).diagram;
        let svg = render(&diag, Theme::Dark);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn snapshot_default_theme() {
        let diag = parser::parse(TREEMAP_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
