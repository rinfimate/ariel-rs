/// Faithful Rust port of Mermaid's c4Renderer.js / c4Db.js.
///
/// Config constants from Mermaid's defaultConfig.c4:
///   diagramMarginX=50, diagramMarginY=10, c4ShapeMargin=50, c4ShapePadding=20
///   width=216, height=60, c4ShapeInRow=4, c4BoundaryInRow=2
use super::constants::*;
use super::parser::{C4Diagram, C4Element, C4ElementType, C4Rel, C4RelType};
use super::templates::{
    all_markers, boundary_label_text, boundary_rect, boundary_type_text, esc, fmt_int, rel_curve,
    rel_label, rel_line, rel_techn_label, shape_db_path, shape_descr_text, shape_label_text,
    shape_person_image, shape_rect, shape_stereo_text, svg_root, symbol_clock, symbol_computer,
    symbol_database, title_text,
};
use crate::theme::Theme;
use std::collections::HashMap;

fn text_line_height(font_size: f64) -> f64 {
    if font_size >= 16.0 {
        font_size
    } else {
        (font_size * 1.21).ceil()
    }
}

fn calc_text_width(text: &str, font_size: f64, bold: bool) -> f64 {
    calc_text_width_style(text, font_size, bold, false)
}

fn calc_text_width_style(text: &str, font_size: f64, bold: bool, italic: bool) -> f64 {
    // Calibrated for "Open Sans" against reference SVG element widths.
    // Non-bold 14px: avg ≈ 7.47 px/char → factor = 7.47/14 ≈ 0.534
    // Bold 16px: avg ≈ 9.92 px/char → factor = 9.92/16 ≈ 0.62
    // Italic 12px (stereo labels): factor ≈ 0.484 (calibrated from reference textLength values)
    let avg = if bold {
        font_size * 0.62
    } else if italic {
        font_size * 0.484
    } else {
        font_size * 0.532
    };
    text.chars()
        .map(|c| {
            if italic {
                match c {
                    'i' | '!' | '|' | '1' | 'j' | ':' | ';' | ',' | '.' | '\'' | '"' => avg * 0.45,
                    'l' => avg * 0.65, // italic 'l' is wider than normal
                    'M' | 'W' | 'm' | 'w' => avg * 1.35,
                    'G' | 'O' | 'Q' | 'D' => avg * 1.15,
                    ' ' => avg * 0.35,
                    '<' | '>' => avg * 0.65,
                    '(' | ')' | '[' | ']' | '-' => avg * 0.55,
                    '_' => avg * 0.80, // italic '_' is wider than normal
                    _ => avg,
                }
            } else {
                match c {
                    'i' | 'l' | '!' | '|' | '1' | 'j' | ':' | ';' | ',' | '.' | '\'' | '"' => {
                        avg * 0.45
                    }
                    'M' | 'W' | 'm' | 'w' => avg * 1.35,
                    'G' | 'O' | 'Q' | 'D' => avg * 1.15,
                    ' ' => avg * 0.35,
                    '<' | '>' => avg * 0.65,
                    '(' | ')' | '[' | ']' | '-' | '_' => avg * 0.55,
                    _ => avg,
                }
            }
        })
        .sum()
}

fn calc_text_height(text: &str, font_size: f64) -> f64 {
    let n = if text.is_empty() {
        0
    } else {
        text.lines().count().max(1)
    };
    text_line_height(font_size) * n as f64
}

fn count_text_lines(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.lines().count().max(1)
    }
}

#[derive(Clone, Debug, Default)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[derive(Clone, Debug)]
struct ShapeLayout {
    rect: Rect,
    stereo_y: f64,
    stereo_w: f64,
    image_y: Option<f64>,
    label_y: f64,
    descr_y: Option<f64>,
}

#[derive(Clone, Debug)]
struct BoundaryLayout {
    rect: Rect,
    label_y: f64,
    type_y: f64,
}

#[derive(Clone, Debug)]
struct Bounds {
    startx: f64,
    stopx: f64,
    starty: f64,
    stopy: f64,
    width_limit: f64,
    next_startx: f64,
    next_stopx: f64,
    next_starty: f64,
    next_stopy: f64,
    next_cnt: usize,
}

impl Bounds {
    fn new(startx: f64, stopx: f64, starty: f64, stopy: f64, width_limit: f64) -> Self {
        Bounds {
            startx,
            stopx,
            starty,
            stopy,
            width_limit,
            next_startx: startx,
            next_stopx: stopx,
            next_starty: starty,
            next_stopy: stopy,
            next_cnt: 0,
        }
    }

    fn insert(&mut self, w: f64, h: f64, margin: f64) -> (f64, f64) {
        self.next_cnt += 1;
        let mut sx = if self.next_startx == self.next_stopx {
            self.next_stopx + margin
        } else {
            self.next_stopx + margin * 2.0
        };
        let mut ex = sx + w;
        let mut sy = self.next_starty + margin * 2.0;
        let mut ey = sy + h;
        if sx >= self.width_limit || ex >= self.width_limit || self.next_cnt > C4_SHAPE_IN_ROW {
            sx = self.next_startx + margin + NEXT_LINE_PADDING_X;
            sy = self.next_stopy + margin * 2.0;
            ex = sx + w;
            self.next_stopx = ex;
            self.next_starty = self.next_stopy;
            ey = sy + h;
            self.next_stopy = ey;
            self.next_cnt = 1;
        }
        self.startx = self.startx.min(sx);
        self.starty = self.starty.min(sy);
        self.stopx = self.stopx.max(ex);
        self.stopy = self.stopy.max(ey);
        self.next_startx = self.next_startx.min(sx);
        self.next_starty = self.next_starty.min(sy);
        self.next_stopx = self.next_stopx.max(ex);
        self.next_stopy = self.next_stopy.max(ey);
        (sx, sy)
    }

    fn bump_last_margin(&mut self, margin: f64) {
        self.stopx += margin;
        self.stopy += margin;
    }
}

fn compute_shape_size(el: &C4Element) -> (f64, f64) {
    let is_person = matches!(el.el_type, C4ElementType::Person | C4ElementType::PersonExt);
    // Mermaid: c4Shape.typeC4Shape.height = fontSize + 2 (hardcoded, NOT text_line_height)
    let stereo_h = STEREO_FONT_SIZE + 2.0; // = 12 + 2 = 14
    let mut y = C4_SHAPE_PADDING + stereo_h - 4.0; // = 20 + 14 - 4 = 30
    if is_person {
        y += 48.0;
    }
    let label_w = calc_text_width(&el.label, LABEL_FONT_SIZE, true);
    let label_h = calc_text_height(&el.label, LABEL_FONT_SIZE);
    y += 8.0 + label_h;
    let text_limit_w = MIN_WIDTH - C4_SHAPE_PADDING * 2.0;
    let (rect_w, rect_h) = if !el.descr.trim().is_empty() {
        let descr_w = calc_text_width(&el.descr, DESCR_FONT_SIZE, false);
        let descr_h = calc_text_height(&el.descr, DESCR_FONT_SIZE);
        let descr_lines = count_text_lines(&el.descr);
        y += 20.0 + descr_h;
        let rw = label_w.max(descr_w).max(text_limit_w) + C4_SHAPE_PADDING;
        (rw, y - descr_lines as f64 * 5.0)
    } else {
        (label_w + C4_SHAPE_PADDING, y)
    };
    (MIN_WIDTH.max(rect_w), MIN_HEIGHT.max(rect_h))
}

fn build_shape_layout(el: &C4Element, x: f64, y_pos: f64) -> ShapeLayout {
    let is_person = matches!(el.el_type, C4ElementType::Person | C4ElementType::PersonExt);
    let type_name = el_type_name(&el.el_type);
    let stereo_text = format!("<<{}>>", type_name);
    let stereo_w = calc_text_width_style(&stereo_text, STEREO_FONT_SIZE, false, true);
    let stereo_h = STEREO_FONT_SIZE + 2.0; // Mermaid: fontSize + 2 = 14 (hardcoded)
    let mut y = C4_SHAPE_PADDING + stereo_h - 4.0; // 20 + 14 - 4 = 30
    let image_y = if is_person {
        let iy = y;
        y += 48.0;
        Some(iy)
    } else {
        None
    };
    y += 8.0;
    let label_y_off = y;
    y += calc_text_height(&el.label, LABEL_FONT_SIZE);
    let descr_y = if !el.descr.trim().is_empty() {
        let descr_h = calc_text_height(&el.descr, DESCR_FONT_SIZE);
        y += 20.0;
        let dy = y;
        y += descr_h;
        let _ = y;
        Some(dy)
    } else {
        None
    };
    let (w, h) = compute_shape_size(el);
    ShapeLayout {
        rect: Rect { x, y: y_pos, w, h },
        stereo_y: C4_SHAPE_PADDING,
        stereo_w,
        image_y,
        label_y: label_y_off,
        descr_y,
    }
}

fn boundary_label_y_offsets() -> (f64, f64) {
    // Calibrated from reference SVG (b0 rect at y=122):
    //   label "BankBoundary0" at y=130 → offset = 8
    //   type  "[ENTERPRISE]"  at y=152 → offset = 30
    // Mermaid calculates:
    //   label.Y = 0 + 8 = 8
    //   label.height = calculateTextHeight(label, 16px bold) = 17 (in the reference env)
    //   type.Y = 8 + 17 + 5 = 30
    // So boundary_header_total_y = type.Y + type.height = 30 + 16 = 46
    (8.0, 30.0)
}

fn boundary_header_total_y() -> f64 {
    // Y = type.Y + type.height = 30 + 16 = 46
    // (type.height = calculateTextHeight(14px) = 16 in the reference env)
    // Verified: Y_global + Y_b0 = 46 + 46 = 92
    //   _y = (starty_screen=10 + diagramMarginY=10 + Y_global=46) + diagramMarginY=10 + Y_b0=46 = 122 ✓
    46.0
}

struct Layout {
    shapes: HashMap<String, ShapeLayout>,
    boundaries: HashMap<String, BoundaryLayout>,
    svg_w: f64,
    svg_h: f64,
    title_x: f64,
}

fn compute_layout(diag: &C4Diagram) -> Layout {
    let mut shapes: HashMap<String, ShapeLayout> = HashMap::new();
    let mut boundaries: HashMap<String, BoundaryLayout> = HashMap::new();
    // Mermaid uses screenBounds.startx=50, starty=10
    let mut global_max_x = DIAGRAM_MARGIN_X; // = 50
    let mut global_max_y = DIAGRAM_MARGIN_Y; // = 10

    // Mermaid's algorithm has a virtual "global" boundary that wraps all user boundaries.
    // The "global" boundary occupies: startx=100, starty=66 (=10+10+46) after its header is
    // calculated (Y_global=46). Then user boundaries start at:
    //   _x = 100 + 50 = 150
    //   _y = 66 + 10 + 46 = 122
    // We simulate this by starting our bounds at the "global" boundary position.
    let global_stopy = DIAGRAM_MARGIN_Y * 2.0 + boundary_header_total_y(); // = 66
    let global_startx = DIAGRAM_MARGIN_X * 2.0; // = 100

    let mut global_bounds = Bounds::new(
        global_startx,
        global_startx,
        global_stopy,
        global_stopy,
        SCREEN_WIDTH,
    );

    let top_level_ids: Vec<String> = diag
        .boundaries
        .iter()
        .filter(|b| b.parent_id.is_none())
        .map(|b| b.id.clone())
        .collect();

    if !top_level_ids.is_empty() {
        draw_inside_boundary(
            &top_level_ids,
            &mut global_bounds,
            top_level_ids.len(),
            diag,
            &mut shapes,
            &mut boundaries,
            &mut global_max_x,
            &mut global_max_y,
        );
        // Mermaid's outer loop (processing "global" boundary) also bumps:
        //   screenBounds.stopy = max(currentBounds_global.stopy + c4ShapeMargin, screenBounds.stopy)
        //   globalBoundaryMaxY = max(globalBoundaryMaxY, screenBounds.stopy)
        // We simulate that here:
        let screen_stopy = (global_bounds.stopy + C4_SHAPE_MARGIN).max(DIAGRAM_MARGIN_Y);
        let screen_stopx = (global_bounds.stopx + C4_SHAPE_MARGIN).max(DIAGRAM_MARGIN_X);
        global_max_y = global_max_y.max(screen_stopy);
        global_max_x = global_max_x.max(screen_stopx);
    } else {
        // No boundaries: draw ungrouped elements starting from global position
        for el in diag.elements.iter().filter(|e| e.boundary_id.is_none()) {
            let (w, h) = compute_shape_size(el);
            let (x, y) = global_bounds.insert(w, h, C4_SHAPE_MARGIN);
            shapes.insert(el.id.clone(), build_shape_layout(el, x, y));
        }
        global_bounds.bump_last_margin(C4_SHAPE_MARGIN);
        // JS applies c4ShapeMargin twice: once in bumpLastMargin, once in
        // parentBounds.data.stopx = currentBounds.data.stopx + c4ShapeMargin
        global_max_x = global_bounds.stopx + C4_SHAPE_MARGIN;
        global_max_y = global_bounds.stopy + C4_SHAPE_MARGIN;
    }

    // Final SVG dimensions use screenBounds formula:
    // boxHeight = globalMaxY - screenBounds.starty = globalMaxY - 10
    // height = boxHeight + 2*diagramMarginY = globalMaxY - 10 + 20
    // boxWidth = globalMaxX - screenBounds.startx = globalMaxX - 50
    // width = boxWidth + 2*diagramMarginX = globalMaxX - 50 + 100
    let svg_h = global_max_y - DIAGRAM_MARGIN_Y + 2.0 * DIAGRAM_MARGIN_Y; // = globalMaxY + 10
    let svg_w = global_max_x - DIAGRAM_MARGIN_X + 2.0 * DIAGRAM_MARGIN_X; // = globalMaxX + 50
                                                                          // title: x = (box.stopx - box.startx)/2 - 4*diagramMarginX
                                                                          //           = (globalMaxX - 50)/2 - 200
    let title_x = (global_max_x - DIAGRAM_MARGIN_X) / 2.0 - 4.0 * DIAGRAM_MARGIN_X;
    Layout {
        shapes,
        boundaries,
        svg_w,
        svg_h,
        title_x,
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_inside_boundary(
    boundary_ids: &[String],
    parent_bounds: &mut Bounds,
    n_boundaries: usize,
    diag: &C4Diagram,
    shapes: &mut HashMap<String, ShapeLayout>,
    boundaries: &mut HashMap<String, BoundaryLayout>,
    global_max_x: &mut f64,
    global_max_y: &mut f64,
) {
    let width_limit = parent_bounds.width_limit / C4_BOUNDARY_IN_ROW.min(n_boundaries) as f64;
    let mut current_bounds = Bounds::new(
        parent_bounds.startx,
        parent_bounds.startx,
        parent_bounds.starty,
        parent_bounds.starty,
        width_limit,
    );

    for (i, bnd_id) in boundary_ids.iter().enumerate() {
        // Check boundary exists; skip if not found
        if diag.boundaries.iter().all(|b| b.id != *bnd_id) {
            continue;
        }
        let header_y = boundary_header_total_y();
        let (bx, by) = if i == 0 || i % C4_BOUNDARY_IN_ROW == 0 {
            (
                parent_bounds.startx + DIAGRAM_MARGIN_X,
                parent_bounds.stopy + DIAGRAM_MARGIN_Y + header_y,
            )
        } else {
            let x = if current_bounds.stopx != current_bounds.startx {
                current_bounds.stopx + DIAGRAM_MARGIN_X
            } else {
                current_bounds.startx
            };
            (x, current_bounds.starty)
        };
        current_bounds = Bounds::new(bx, bx, by, by, width_limit);

        for el_id in diag
            .elements
            .iter()
            .filter(|e| e.boundary_id.as_deref() == Some(bnd_id.as_str()))
            .map(|e| e.id.clone())
            .collect::<Vec<_>>()
        {
            if let Some(el) = diag.elements.iter().find(|e| e.id == el_id) {
                let (w, h) = compute_shape_size(el);
                let (ex, ey) = current_bounds.insert(w, h, C4_SHAPE_MARGIN);
                shapes.insert(el_id.clone(), build_shape_layout(el, ex, ey));
            }
        }
        current_bounds.bump_last_margin(C4_SHAPE_MARGIN);

        let sub_ids: Vec<String> = diag
            .boundaries
            .iter()
            .filter(|b| b.parent_id.as_deref() == Some(bnd_id.as_str()))
            .map(|b| b.id.clone())
            .collect();
        if !sub_ids.is_empty() {
            draw_inside_boundary(
                &sub_ids,
                &mut current_bounds,
                sub_ids.len(),
                diag,
                shapes,
                boundaries,
                global_max_x,
                global_max_y,
            );
        }

        let (label_y_off, type_y_off) = boundary_label_y_offsets();
        boundaries.insert(
            bnd_id.clone(),
            BoundaryLayout {
                rect: Rect {
                    x: current_bounds.startx,
                    y: current_bounds.starty,
                    w: current_bounds.stopx - current_bounds.startx,
                    h: current_bounds.stopy - current_bounds.starty,
                },
                label_y: label_y_off,
                type_y: type_y_off,
            },
        );

        parent_bounds.stopy = parent_bounds
            .stopy
            .max(current_bounds.stopy + C4_SHAPE_MARGIN);
        parent_bounds.stopx = parent_bounds
            .stopx
            .max(current_bounds.stopx + C4_SHAPE_MARGIN);
        *global_max_x = (*global_max_x).max(parent_bounds.stopx);
        *global_max_y = (*global_max_y).max(parent_bounds.stopy);
    }
}

pub fn render(diag: &C4Diagram, theme: Theme, _use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    // C4 uses specific fonts per element type (from defaultConfig.c4):
    //   personFontFamily / systemFontFamily: "Open Sans", sans-serif
    //   messageFontFamily: "trebuchet ms", verdana, arial, sans-serif
    // Use &quot; escaping since these appear inside style="" attributes.
    let ff = FF_SHAPE; // default for shape labels
    let svg_id = "mermaid-c4";
    let layout = compute_layout(diag);
    let Layout {
        shapes,
        boundaries,
        svg_w,
        svg_h,
        title_x,
    } = layout;
    let extra_vert = if diag.title.is_some() { 60.0 } else { 0.0 };
    let vb_y = -(DIAGRAM_MARGIN_Y + extra_vert);
    let vb_h = svg_h + extra_vert;

    let mut out = String::new();
    out.push_str(&svg_root(svg_id, svg_w, vb_y, vb_h));
    out.push_str("<g></g>");
    out.push_str(&symbol_computer(svg_id));
    out.push_str(&symbol_database(svg_id));
    out.push_str(&symbol_clock(svg_id));

    for el in &diag.elements {
        if let Some(shape) = shapes.get(&el.id) {
            out.push_str(&render_shape(el, shape, ff));
        }
    }
    for boundary in diag.boundaries.iter().rev() {
        if let Some(bl) = boundaries.get(&boundary.id) {
            out.push_str(&render_boundary(boundary, bl, ff));
        }
    }

    out.push_str(&all_markers(svg_id));

    if !diag.rels.is_empty() {
        out.push_str("<g>");
        let mut draw_as_line = true;
        for rel in &diag.rels {
            if let (Some(from_s), Some(to_s)) = (shapes.get(&rel.from), shapes.get(&rel.to)) {
                out.push_str(&render_rel(rel, from_s, to_s, svg_id, ff, draw_as_line));
                if draw_as_line {
                    draw_as_line = false;
                }
            }
        }
        out.push_str("</g>");
    }

    if let Some(ref title) = diag.title {
        out.push_str(&title_text(title_x, &esc(title), vars.text_color));
    }
    out.push_str("</svg>");
    out
}

fn render_shape(el: &C4Element, shape: &ShapeLayout, ff: &str) -> String {
    let type_name = el_type_name(&el.el_type);
    let is_db = matches!(
        el.el_type,
        C4ElementType::SystemDb
            | C4ElementType::SystemDbExt
            | C4ElementType::ContainerDb
            | C4ElementType::ContainerDbExt
            | C4ElementType::ComponentDb
            | C4ElementType::ComponentDbExt
    );
    let (fill, stroke) = element_colors(&el.el_type);
    let cx = shape.rect.x + shape.rect.w / 2.0;
    let stereo_text = format!("<<{}>>", type_name);
    let stereo_x = cx - shape.stereo_w / 2.0;
    let stereo_y_abs = shape.rect.y + shape.stereo_y;
    let mut s = String::new();
    s.push_str("<g class=\"person-man\">");
    if is_db {
        let half = shape.rect.w / 2.0;
        let h = shape.rect.h;
        s.push_str(&shape_db_path(
            fill,
            stroke,
            shape.rect.x,
            shape.rect.y,
            half,
            h,
        ));
    } else {
        s.push_str(&shape_rect(
            shape.rect.x,
            shape.rect.y,
            fill,
            stroke,
            shape.rect.w,
            shape.rect.h,
        ));
    }
    s.push_str(&shape_stereo_text(
        SHAPE_TEXT_COLOR,
        ff,
        &fmt_int(shape.stereo_w),
        stereo_x,
        stereo_y_abs,
        &esc(&stereo_text),
    ));
    if let Some(img_y_off) = shape.image_y {
        s.push_str(&shape_person_image(
            cx,
            shape.rect.y + img_y_off,
            PERSON_PNG,
        ));
    }
    let label_y_abs = shape.rect.y + shape.label_y;
    s.push_str(&shape_label_text(
        cx,
        label_y_abs,
        SHAPE_TEXT_COLOR,
        ff,
        &esc(&el.label),
    ));
    if let Some(descr_y_off) = shape.descr_y {
        s.push_str(&shape_descr_text(
            cx,
            shape.rect.y + descr_y_off,
            SHAPE_TEXT_COLOR,
            ff,
            &esc(&el.descr),
        ));
    }
    s.push_str("</g>");
    s
}

fn render_boundary(boundary: &super::parser::C4Boundary, bl: &BoundaryLayout, ff: &str) -> String {
    let cx = bl.rect.x + bl.rect.w / 2.0;
    let mut s = String::new();
    s.push_str("<g>");
    s.push_str(&boundary_rect(bl.rect.x, bl.rect.y, bl.rect.w, bl.rect.h));
    s.push_str(&boundary_label_text(
        cx,
        bl.rect.y + bl.label_y,
        BOUNDARY_TEXT_COLOR,
        ff,
        &esc(&boundary.label),
    ));
    s.push_str(&boundary_type_text(
        cx,
        bl.rect.y + bl.type_y,
        BOUNDARY_TEXT_COLOR,
        ff,
        &esc(&boundary.boundary_type.to_uppercase()),
    ));
    s.push_str("</g>");
    s
}

fn render_rel(
    rel: &C4Rel,
    from: &ShapeLayout,
    to: &ShapeLayout,
    svg_id: &str,
    _ff: &str,
    is_line: bool,
) -> String {
    let fn_box = |s: &ShapeLayout| NodeBox {
        x: s.rect.x,
        y: s.rect.y,
        w: s.rect.w,
        h: s.rect.h,
    };
    let from_node = fn_box(from);
    let to_node = fn_box(to);
    let to_center = Pt {
        x: to_node.x + to_node.w / 2.0,
        y: to_node.y + to_node.h / 2.0,
    };
    let from_center = Pt {
        x: from_node.x + from_node.w / 2.0,
        y: from_node.y + from_node.h / 2.0,
    };
    let sp = get_intersect_point(&from_node, &to_center);
    let ep = get_intersect_point(&to_node, &from_center);
    let is_bi = rel.rel_type == C4RelType::BiRel;
    let mut s = String::new();
    // JS: label position = geometric midpoint of startPoint/endPoint (for both line and bezier)
    // svgDraw.js: x = min(startX,endX) + |endX-startX|/2 = (startX+endX)/2
    let geom_mid_x = (sp.x + ep.x) / 2.0;
    let geom_mid_y = (sp.y + ep.y) / 2.0;

    if is_line {
        let bi = if is_bi {
            format!(" marker-start=\"url(#{}-arrowend)\"", svg_id)
        } else {
            String::new()
        };
        s.push_str(&rel_line(sp.x, sp.y, ep.x, ep.y, svg_id, &bi));
    } else {
        let ctrl_x = sp.x + (ep.x - sp.x) / 2.0 - (ep.x - sp.x) / 4.0;
        let ctrl_y = sp.y + (ep.y - sp.y) / 2.0;
        let bi = if is_bi {
            format!(" marker-start=\"url(#{}-arrowend)\"", svg_id)
        } else {
            String::new()
        };
        s.push_str(&rel_curve(
            sp.x, sp.y, ctrl_x, ctrl_y, ep.x, ep.y, svg_id, &bi,
        ));
    }
    // JS byTspan: text.attr('x', x + width/2) where x = geometric midpoint
    // → text center x = geom_mid_x + label_width/2
    let label_w = calc_text_width(&rel.label, MSG_FONT_SIZE, false);
    let techn_text = format!("[{}]", rel.techn);
    let techn_w = calc_text_width(&techn_text, MSG_FONT_SIZE, false);
    if !rel.label.is_empty() {
        let tx = geom_mid_x + label_w / 2.0;
        s.push_str(&rel_label(tx, geom_mid_y, FF_MSG_LABEL, &esc(&rel.label)));
    }
    if !rel.techn.is_empty() {
        // techn width = max(label.width, techn.width); y += messageFontSize + 5
        let max_w = label_w.max(techn_w);
        let tx = geom_mid_x + max_w / 2.0;
        let ty = geom_mid_y + MSG_FONT_SIZE + 5.0;
        s.push_str(&rel_techn_label(tx, ty, FF_TECHN_LABEL, &rel.techn));
    }
    s
}

struct Pt {
    x: f64,
    y: f64,
}
struct NodeBox {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn get_intersect_point(node: &NodeBox, end_pt: &Pt) -> Pt {
    let (x1, y1, x2, y2) = (node.x, node.y, end_pt.x, end_pt.y);
    let (cx, cy) = (x1 + node.w / 2.0, y1 + node.h / 2.0);
    let (dx, dy) = ((x1 - x2).abs(), (y1 - y2).abs());
    let tan_dyx = dy / dx.max(EPS);
    let from_dyx = node.h / node.w.max(EPS);
    if (y1 - y2).abs() < EPS && x1 < x2 {
        return Pt {
            x: x1 + node.w,
            y: cy,
        };
    }
    if (y1 - y2).abs() < EPS && x1 > x2 {
        return Pt { x: x1, y: cy };
    }
    if (x1 - x2).abs() < EPS && y1 < y2 {
        return Pt {
            x: cx,
            y: y1 + node.h,
        };
    }
    if (x1 - x2).abs() < EPS && y1 > y2 {
        return Pt { x: cx, y: y1 };
    }
    if x1 > x2 && y1 < y2 {
        if from_dyx >= tan_dyx {
            Pt {
                x: x1,
                y: cy + tan_dyx * node.w / 2.0,
            }
        } else {
            Pt {
                x: cx - dx / dy.max(EPS) * node.h / 2.0,
                y: y1 + node.h,
            }
        }
    } else if x1 < x2 && y1 < y2 {
        if from_dyx >= tan_dyx {
            Pt {
                x: x1 + node.w,
                y: cy + tan_dyx * node.w / 2.0,
            }
        } else {
            Pt {
                x: cx + dx / dy.max(EPS) * node.h / 2.0,
                y: y1 + node.h,
            }
        }
    } else if x1 < x2 && y1 > y2 {
        if from_dyx >= tan_dyx {
            Pt {
                x: x1 + node.w,
                y: cy - tan_dyx * node.w / 2.0,
            }
        } else {
            Pt {
                x: cx + node.h / 2.0 * dx / dy.max(EPS),
                y: y1,
            }
        }
    } else {
        if from_dyx >= tan_dyx {
            Pt {
                x: x1,
                y: cy - node.w / 2.0 * tan_dyx,
            }
        } else {
            Pt {
                x: cx - node.h / 2.0 * dx / dy.max(EPS),
                y: y1,
            }
        }
    }
}

fn element_colors(el_type: &C4ElementType) -> (&'static str, &'static str) {
    match el_type {
        C4ElementType::Person => ("#08427B", "#073B6F"),
        C4ElementType::PersonExt => ("#686868", "#8A8A8A"),
        C4ElementType::System => ("#1168BD", "#3C7FC0"),
        C4ElementType::SystemExt | C4ElementType::SystemDbExt => ("#999999", "#8A8A8A"),
        C4ElementType::SystemDb => ("#1168BD", "#3C7FC0"),
        C4ElementType::Container => ("#438DD5", "#3C7FC0"),
        C4ElementType::ContainerExt | C4ElementType::ContainerDbExt => ("#B3B3B3", "#A6A6A6"),
        C4ElementType::ContainerDb => ("#438DD5", "#3C7FC0"),
        C4ElementType::Component => ("#85BBF0", "#78A8D8"),
        C4ElementType::ComponentExt | C4ElementType::ComponentDbExt => ("#CCCCCC", "#BFBFBF"),
        C4ElementType::ComponentDb => ("#85BBF0", "#78A8D8"),
        C4ElementType::Node => ("#438DD5", "#3C7FC0"),
        C4ElementType::NodeExt => ("#B3B3B3", "#A6A6A6"),
    }
}

fn el_type_name(el_type: &C4ElementType) -> &'static str {
    match el_type {
        C4ElementType::Person => "person",
        C4ElementType::PersonExt => "external_person",
        C4ElementType::System => "system",
        C4ElementType::SystemExt => "external_system",
        C4ElementType::SystemDb => "system_db",
        C4ElementType::SystemDbExt => "external_system_db",
        C4ElementType::Container => "container",
        C4ElementType::ContainerExt => "external_container",
        C4ElementType::ContainerDb => "container_db",
        C4ElementType::ContainerDbExt => "external_container_db",
        C4ElementType::Component => "component",
        C4ElementType::ComponentExt => "external_component",
        C4ElementType::ComponentDb => "component_db",
        C4ElementType::ComponentDbExt => "external_component_db",
        C4ElementType::Node => "node",
        C4ElementType::NodeExt => "external_node",
    }
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    #[test]
    fn basic_render_produces_svg() {
        let input = "C4Context\n    title System Context\n    Person(customerA, \"Banking Customer A\", \"A customer\")\n    System(SystemAA, \"Internet Banking System\", \"Allows customers\")\n    Rel(customerA, SystemAA, \"Uses\")";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains("<svg"), "no <svg element");
        assert!(svg.contains("System Context"), "no title");
        assert!(svg.contains("Banking Customer A"), "no element label");
        assert!(svg.contains("Uses"), "no rel label");
    }

    #[test]
    fn person_renders_with_image() {
        let input = "C4Context\n    Person(user, \"User\", \"A user\")\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains("<image"), "no image for person");
        assert!(
            svg.contains("data:image/png;base64"),
            "no base64 person icon"
        );
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "C4Context\n      title System Context diagram for Internet Banking System\n      Enterprise_Boundary(b0, \"BankBoundary0\") {\n        Person(customerA, \"Banking Customer A\")\n        Person(customerB, \"Banking Customer B\")\n        System(SystemAA, \"Internet Banking System\")\n      }";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
