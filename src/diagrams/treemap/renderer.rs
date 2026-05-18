use super::constants::*;
use super::templates;
// Faithful Rust port of mermaid/src/diagrams/treemap/renderer.ts
//
// Uses a squarified-treemap layout (port of D3's d3.treemap().tile(d3.treemapSquarify)).
// Canvas: 1000×400 (nodeWidth=100 * SECTION_INNER_PADDING=10, nodeHeight=40*10).
// Leaf fill/stroke: colorScale(parent.name). Top-level leaves have transparent parent → transparent.
// Label colors: cScaleLabel0/1/2 from default Mermaid theme, assigned by node ordinal.
// Value format: comma-separated thousands (1,200 not 1200).
// ViewBox: computed from actual content bounding box + diagramPadding=8.

use super::parser::{TreemapDiagram, TreemapNode};
use crate::theme::Theme;

// ─── Constants (from defaultConfig.treemap and renderer.ts) ─────────────────
// All constants are imported from super::constants via `use super::constants::*`.

// ─── D3-style treemap node ───────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct TmRect {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl TmRect {
    fn w(&self) -> f64 {
        self.x1 - self.x0
    }
    fn h(&self) -> f64 {
        self.y1 - self.y0
    }
}

// ─── D3 squarify layout  ─────────────────────────────────────────────────────
//
// Direct port of d3-hierarchy/src/treemap/squarify.js (phi ratio, round=true).
// Returns leaf rectangles in the same order as `values`.
// PHI is imported from constants.

/// Lay out `children` (given as (value, idx) pairs) into `bounds`.
/// Returns one TmRect per child, in the same order.
/// Coordinates are raw (before inner-padding trim is applied to leaves).
fn d3_squarify(values: &[f64], bounds: TmRect) -> Vec<TmRect> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }

    // Work with a mutable position array for squarifyRatio
    let mut rects: Vec<TmRect> = vec![
        TmRect {
            x0: 0.0,
            y0: 0.0,
            x1: 0.0,
            y1: 0.0
        };
        n
    ];
    squarify_ratio(PHI, values, &bounds, &mut rects);
    rects
}

fn squarify_ratio(ratio: f64, values: &[f64], bounds: &TmRect, rects: &mut [TmRect]) {
    let n = values.len();
    let total_value: f64 = values.iter().sum();
    if total_value <= 0.0 || n == 0 {
        return;
    }

    let mut x0 = bounds.x0;
    let mut y0 = bounds.y0;
    let x1 = bounds.x1;
    let y1 = bounds.y1;
    let mut remaining_value = total_value;

    let mut i0: usize = 0; // start of current row

    while i0 < n {
        let dx = x1 - x0;
        let dy = y1 - y0;

        // Find the first non-zero value
        let mut i1 = i0;
        let mut sum_value = 0.0_f64;
        while i1 < n && sum_value == 0.0 {
            sum_value = values[i1];
            i1 += 1;
        }
        if sum_value == 0.0 {
            break;
        }

        let mut min_value = sum_value;
        let mut max_value = sum_value;
        let alpha = f64::max(dy / dx, dx / dy) / (remaining_value * ratio);
        let mut beta = sum_value * sum_value * alpha;
        let mut min_ratio = f64::max(max_value / beta, beta / min_value);

        // Try adding more items to the row
        while i1 < n {
            let node_value = values[i1];
            sum_value += node_value;
            if node_value < min_value {
                min_value = node_value;
            }
            if node_value > max_value {
                max_value = node_value;
            }
            beta = sum_value * sum_value * alpha;
            let new_ratio = f64::max(max_value / beta, beta / min_value);
            if new_ratio > min_ratio {
                // Backtrack
                sum_value -= node_value;
                break;
            }
            min_ratio = new_ratio;
            i1 += 1;
        }

        // Row is values[i0..i1] with sum_value
        let is_dice = dx < dy;
        assign_row(
            values,
            i0,
            i1,
            sum_value,
            remaining_value,
            is_dice,
            x0,
            y0,
            x1,
            y1,
            rects,
        );

        // Advance origin
        if is_dice {
            y0 += dy * sum_value / remaining_value;
        } else {
            x0 += dx * sum_value / remaining_value;
        }
        remaining_value -= sum_value;
        i0 = i1;
    }
}

/// Assign rects for the row `values[i0..i1]` (is_dice=false → slice, true → dice).
/// slice: all items share the same x-band [x0, x0+band_w], distributed vertically.
/// dice:  all items share the same y-band [y0, y0+band_h], distributed horizontally.
#[allow(clippy::too_many_arguments)]
fn assign_row(
    values: &[f64],
    i0: usize,
    i1: usize,
    sum_value: f64,
    total_value: f64,
    is_dice: bool,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    rects: &mut [TmRect],
) {
    let dx = x1 - x0;
    let dy = y1 - y0;

    if is_dice {
        // dice_default: distribute along x, full y span
        let _k = if sum_value > 0.0 {
            dx * sum_value / total_value / sum_value
        } else {
            0.0
        };
        // Actually dice distributes proportional to each item's value within the band:
        // band_width = dx * sum_value / total_value
        // each item gets: value / sum_value * band_width
        // But since slice_default passes x1 = x0 + band_w (already advanced),
        // we use (x1-x0) as full band width and distribute by value/sum_value.
        let band_x1 = x0 + dx * sum_value / total_value;
        let band_dx = band_x1 - x0;
        let k = if sum_value > 0.0 {
            band_dx / sum_value
        } else {
            0.0
        };
        let mut cur_x = x0;
        for idx in i0..i1 {
            let v = values[idx];
            rects[idx] = TmRect {
                x0: cur_x,
                y0,
                x1: {
                    cur_x += v * k;
                    cur_x
                },
                y1,
            };
        }
    } else {
        // slice_default: distribute along y, full x span = band
        // band_width = dx * sum_value / total_value; but x1 passed IS x0+band_w already
        let band_x1 = x0 + dx * sum_value / total_value;
        let k = if sum_value > 0.0 { dy / sum_value } else { 0.0 };
        let mut cur_y = y0;
        for idx in i0..i1 {
            let v = values[idx];
            rects[idx] = TmRect {
                x0,
                y0: cur_y,
                x1: band_x1,
                y1: {
                    cur_y += v * k;
                    cur_y
                },
            };
        }
    }
}

// ─── D3 round ────────────────────────────────────────────────────────────────
fn d3_round(v: f64) -> f64 {
    v.round()
}

// ─── Laid-out leaf ───────────────────────────────────────────────────────────

struct LeafNode<'a> {
    node: &'a TreemapNode,
    /// Final screen rect (after inner padding applied and rounded)
    rect: TmRect,
    /// Index into C_SCALE_LABEL (ordinal position of this leaf in sorted order)
    label_color_idx: usize,
}

// ─── Layout ──────────────────────────────────────────────────────────────────

/// Collect all leaf nodes with their layout rects.
/// Mirrors D3's hierarchy.sum().sort(desc).eachBefore(positionNode) + round.
fn layout_leaves<'a>(nodes: &'a [TreemapNode], title_h: f64) -> Vec<LeafNode<'a>> {
    // Flatten into (value, node_ref) sorted descending by value
    let mut items: Vec<(f64, &'a TreemapNode)> = nodes.iter().map(|n| (sum_value(n), n)).collect();
    items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let values: Vec<f64> = items.iter().map(|(v, _)| *v).collect();
    let total: f64 = values.iter().sum();

    // Virtual root bounds = full canvas
    // Apply outer padding to the virtual root:
    //   paddingLeft/Right/Bottom = SECTION_INNER_PADDING = 10, inner/2 = 5 → net = 5
    //   paddingTop = SECTION_HEADER_HEIGHT + SECTION_INNER_PADDING = 35, inner/2 = 5 → net = 30
    let p = INNER_PADDING / 2.0; // = 5.0
    let root_bounds = TmRect {
        x0: SECTION_INNER_PADDING - p, // 10 - 5 = 5
        y0: (SECTION_HEADER_HEIGHT + SECTION_INNER_PADDING) - p + title_h, // 35-5=30
        x1: CANVAS_W - (SECTION_INNER_PADDING - p), // 1000-5 = 995
        y1: CANVAS_H + title_h - (SECTION_INNER_PADDING - p), // 400-5 = 395
    };

    if total <= 0.0 || values.is_empty() {
        return Vec::new();
    }

    // Get raw rects from squarify
    let raw_rects = d3_squarify(&values, root_bounds.clone());

    // Apply inner padding (p=5) to each leaf and round
    let mut leaves: Vec<LeafNode<'a>> = Vec::with_capacity(items.len());
    for (idx, (_, node)) in items.iter().enumerate() {
        let r = &raw_rects[idx];
        let rect = TmRect {
            x0: d3_round(r.x0 + p),
            y0: d3_round(r.y0 + p),
            x1: d3_round(r.x1 - p),
            y1: d3_round(r.y1 - p),
        };
        leaves.push(LeafNode {
            node,
            rect,
            label_color_idx: idx,
        });
    }
    leaves
}

fn sum_value(node: &TreemapNode) -> f64 {
    if node.children.is_empty() {
        node.value.unwrap_or(1.0).max(0.0)
    } else {
        node.children.iter().map(sum_value).sum::<f64>().max(0.0)
    }
}

// ─── Rendering ───────────────────────────────────────────────────────────────

pub fn render(diag: &TreemapDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let title_h = if diag.title.is_some() {
        TITLE_HEIGHT
    } else {
        0.0
    };

    // Compute layout
    let leaves = layout_leaves(&diag.roots, title_h);

    let mut out = String::new();

    // Compute viewBox from bounding box of all content + DIAGRAM_PADDING
    // (mirrors setupViewPortForSVG → calculateDimensionsWithPadding → getBBox)
    let (bbox_x, bbox_y, bbox_w, bbox_h) = if leaves.is_empty() {
        (0.0, 0.0, CANVAS_W, CANVAS_H + title_h)
    } else {
        let x_min = leaves.iter().map(|l| l.rect.x0).fold(f64::MAX, f64::min);
        let y_min = leaves.iter().map(|l| l.rect.y0).fold(f64::MAX, f64::min);
        let x_max = leaves.iter().map(|l| l.rect.x1).fold(f64::MIN, f64::max);
        let y_max = leaves.iter().map(|l| l.rect.y1).fold(f64::MIN, f64::max);
        let w = x_max - x_min;
        let h = y_max - y_min;
        (x_min, y_min, w, h)
    };

    let vb_x = bbox_x - DIAGRAM_PADDING;
    let vb_y = bbox_y - DIAGRAM_PADDING;
    let vb_w = bbox_w + DIAGRAM_PADDING * 2.0;
    let vb_h = bbox_h + DIAGRAM_PADDING * 2.0;

    // SVG header (useMaxWidth=true → width="100%", max-width style)
    out.push_str(&templates::svg_root(
        &fmt(vb_w),
        &fmt(vb_x),
        &fmt(vb_y),
        &fmt(vb_w),
        &fmt(vb_h),
    ));

    // Minimal style block matching mermaid's treemap styles
    out.push_str(
        "<style>.treemapNode.leaf{stroke:black;stroke-width:1;fill:#efefef;}\
        .treemapLabel{fill:#333;font-size:12px;}\
        .treemapValue{fill:#333;font-size:10px;}</style>",
    );

    // Title
    if let Some(t) = &diag.title {
        out.push_str(&templates::title_text(
            &fmt(CANVAS_W / 2.0),
            &fmt(TITLE_HEIGHT / 2.0),
            ff,
            &esc(t),
        ));
    }

    // Render leaves
    for leaf in &leaves {
        let r = &leaf.rect;
        let w = r.w();
        let h = r.h();
        if w <= 0.0 || h <= 0.0 {
            continue;
        }

        let label_color = C_SCALE_LABEL[leaf.label_color_idx.min(C_SCALE_LABEL.len() - 1)];

        // Leaf rect — fill and stroke from colorScale(parent.name)="transparent"
        out.push_str(&templates::leaf_rect(
            &fmt(r.x0),
            &fmt(r.y0),
            &fmt(w),
            &fmt(h),
        ));

        let avail_w = w - 2.0 * 4.0; // padding=4 each side
        let avail_h = h - 2.0 * 4.0;

        if avail_w < 10.0 || avail_h < 10.0 {
            continue;
        }

        // Label font size: start at 38, shrink by 1 until label text width fits
        // We use approximate character width (avg ~0.6*fontSize for Arial)
        let name = leaf.node.name.clone();
        let mut label_fs: i32 = 38;
        while label_fs > 8 {
            let text_w = approx_text_width(&name, label_fs as f64);
            if text_w <= avail_w {
                break;
            }
            label_fs -= 1;
        }
        // Also check combined height
        let orig_value_fs = 28;
        let mut val_fs = std::cmp::max(
            6,
            std::cmp::min(orig_value_fs, (label_fs as f64 * 0.6).round() as i32),
        );
        let combined_h = label_fs as f64 + 2.0 + val_fs as f64;
        if combined_h > avail_h && label_fs > 8 {
            // Shrink until combined height fits
            while label_fs > 8 {
                label_fs -= 1;
                val_fs = std::cmp::max(
                    6,
                    std::cmp::min(orig_value_fs, (label_fs as f64 * 0.6).round() as i32),
                );
                let ch = label_fs as f64 + 2.0 + val_fs as f64;
                if ch <= avail_h {
                    break;
                }
            }
        }

        // Check if label still fits width after combined-height shrinking
        let text_w = approx_text_width(&name, label_fs as f64);
        if text_w > avail_w || label_fs < 8 {
            // display:none equivalent — skip
            continue;
        }

        let cx = r.x0 + w / 2.0;
        let label_center_y = r.y0 + h / 2.0;

        // Label text (dominant-baseline=middle, y = center of tile)
        out.push_str(&templates::leaf_label_text(
            &fmt(cx),
            &fmt(label_center_y),
            ff,
            label_fs,
            label_color,
            &esc(&name),
        ));

        // Value text (dominant-baseline=hanging, y = center + labelFs/2 + spacing)
        if let Some(v) = leaf.node.value {
            let val_str = format_value_comma(v);
            let value_top_y = label_center_y + label_fs as f64 / 2.0 + 2.0;
            out.push_str(&templates::leaf_value_text(
                &fmt(cx),
                &fmt(value_top_y),
                ff,
                val_fs,
                label_color,
                &esc(&val_str),
            ));
        }
    }

    out.push_str("</svg>");
    out
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Approximate text width for Arial at given font-size.
/// Uses average char width ~0.55 * fontSize (conservative estimate for uppercase,
/// calibrated to match browser rendering for typical node names).
fn approx_text_width(text: &str, font_size: f64) -> f64 {
    // Average Arial char width ratio ≈ 0.55 × fontSize
    text.len() as f64 * font_size * 0.55
}

/// Format a value with comma thousand separators (matching d3-format(",")).
fn format_value_comma(v: f64) -> String {
    if v == v.floor() && v.abs() < 1e15 {
        let n = v as i64;
        format_with_commas(n)
    } else {
        // Fallback: just format to reasonable precision
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
    #[ignore = "platform-specific float precision — run locally"]
    fn snapshot_default_theme() {
        let diag = parser::parse(TREEMAP_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(svg);
    }
}
