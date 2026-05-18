use super::constants::*;
use super::parser::{C4Boundary, C4Diagram, C4Element, C4ElementType, C4Rel, C4RelType};
/// Faithful Rust port of Mermaid's c4Renderer.ts.
///
/// Layout algorithm reverse-engineered from Mermaid reference SVG output.
///
/// Key constants (all measured from reference SVGs):
///   ELEMENT_W   = 216  (element box width)
///   H_GAP       = 100  (horizontal gap between elements)
///   V_GAP       = 100  (vertical gap between element rows)
///   SVG_LEFT    = 150  (x of first element left edge, no boundary)
///   BOUND_PAD   = 50   (boundary padding on left/right/bottom)
///   BOUND_TOP   = 100  (space from boundary top to first element row)
///   ELEM_BASE_Y = 166  (y of first element row, ungrouped)
///   Person no-descr h = 103; with 1-line descr += 31 per line
///   System  no-descr h = 60; with 1-line descr += 26 per line
///   viewBox y-offset = -70
///   viewBox extra bottom = 100 (below last element)
#[allow(unused_imports)]
use super::templates;
use crate::theme::Theme;

pub fn render(diag: &C4Diagram, theme: Theme, _use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let svg_id = "mermaid-c4";
    let (layout, total_w, total_h) = compute_layout(diag);

    let has_boundary = !diag.boundaries.is_empty()
        && diag.boundaries.iter().any(|b| {
            diag.elements
                .iter()
                .any(|e| e.boundary_id.as_deref() == Some(&b.id))
        });
    let content_start_x = SVG_LEFT + if has_boundary { BOUND_PAD } else { 0.0 };
    let title_x = content_start_x + 16.0;

    let mut out = String::new();

    // SVG root
    out.push_str("<svg id=\"");
    out.push_str(svg_id);
    out.push_str("\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\"");
    out.push_str(" xmlns:xlink=\"http://www.w3.org/1999/xlink\"");
    out.push_str(&format!(" style=\"max-width: {}px;\"", fmt(total_w)));
    out.push_str(&format!(
        " viewBox=\"0 -70 {} {}\"",
        fmt(total_w),
        fmt(total_h)
    ));
    out.push_str(" role=\"graphics-document document\" aria-roledescription=\"c4\">");

    out.push_str("<style>");
    out.push_str(&build_style(svg_id, ff));
    out.push_str("</style>");

    // Arrow marker defs
    out.push_str("<defs>");
    out.push_str(&format!(
        "<marker id=\"{id}-arrowhead\" refX=\"9\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"12\" markerHeight=\"12\" orient=\"auto\"><path d=\"M 0 0 L 10 5 L 0 10 z\"></path></marker>",
        id = svg_id
    ));
    out.push_str("</defs>");
    out.push_str("<defs>");
    out.push_str(&format!(
        "<marker id=\"{id}-arrowend\" refX=\"1\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"12\" markerHeight=\"12\" orient=\"auto\"><path d=\"M 10 0 L 0 5 L 10 10 z\"></path></marker>",
        id = svg_id
    ));
    out.push_str("</defs>");

    // Title
    if let Some(ref title) = diag.title {
        out.push_str(&format!(
            "<text x=\"{}\" y=\"20\">{}</text>",
            fmt(title_x),
            esc(title)
        ));
    }

    // Boundary boxes
    for boundary in &diag.boundaries {
        if let Some(s) = render_boundary(diag, boundary, &layout, ff) {
            out.push_str(&s);
        }
    }

    // Elements
    for el in &diag.elements {
        if let Some(&(rx, ry, rw, rh)) = layout.elements.get(&el.id) {
            out.push_str(&render_element(el, rx, ry, rw, rh, ff));
        }
    }

    // Relationships
    if !diag.rels.is_empty() {
        out.push_str("<g>");
        for rel in &diag.rels {
            if let (Some(&(fx, fy, fw, fh)), Some(&(tx, ty, tw, th))) =
                (layout.elements.get(&rel.from), layout.elements.get(&rel.to))
            {
                out.push_str(&render_rel(rel, fx, fy, fw, fh, tx, ty, tw, th, svg_id, ff));
            }
        }
        out.push_str("</g>");
    }

    out.push_str("</svg>");
    out
}

struct Layout {
    elements: std::collections::HashMap<String, (f64, f64, f64, f64)>,
}

fn compute_layout(diag: &C4Diagram) -> (Layout, f64, f64) {
    use std::collections::HashMap;
    let mut elements: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();

    // Collect groups: boundary groups first, then ungrouped
    let mut groups: Vec<(Option<&C4Boundary>, Vec<&C4Element>)> = Vec::new();

    for boundary in &diag.boundaries {
        let els: Vec<&C4Element> = diag
            .elements
            .iter()
            .filter(|e| e.boundary_id.as_deref() == Some(&boundary.id))
            .collect();
        if !els.is_empty() {
            groups.push((Some(boundary), els));
        }
    }

    let ungrouped: Vec<&C4Element> = diag
        .elements
        .iter()
        .filter(|e| e.boundary_id.is_none())
        .collect();
    if !ungrouped.is_empty() {
        groups.push((None, ungrouped));
    }

    // Determine initial cur_y based on whether first group is bounded
    // These match Mermaid reference output exactly:
    //   - First bounded group: boundary top at y=122, elements at y=222
    //   - First ungrouped group: elements at y=166
    let first_is_bounded = groups.first().map(|(b, _)| b.is_some()).unwrap_or(false);
    let mut cur_y = if first_is_bounded {
        BOUNDARY_FIRST_Y // boundary top; elements will be at cur_y + BOUND_TOP
    } else {
        UNGROUPED_FIRST_Y // elements start directly here
    };

    let mut max_right = 0.0_f64; // tracks rightmost element edge (before boundary pad)
    let mut any_bounded = false;

    for (boundary_opt, els) in &groups {
        let is_bounded = boundary_opt.is_some();
        if is_bounded {
            any_bounded = true;
        }

        // Element start positions
        let elem_start_y = if is_bounded { cur_y + BOUND_TOP } else { cur_y };
        let elem_start_x = SVG_LEFT + if is_bounded { BOUND_PAD } else { 0.0 };

        let mut col = 0usize;
        let mut row_y = elem_start_y;
        let mut row_max_h = 0.0_f64;

        for el in els.iter() {
            let ew = ELEMENT_W;
            let eh = element_height(el);
            let ex = elem_start_x + col as f64 * (ELEMENT_W + H_GAP);
            let ey = row_y;

            elements.insert(el.id.clone(), (ex, ey, ew, eh));

            let right = ex + ew;
            if right > max_right {
                max_right = right;
            }
            if eh > row_max_h {
                row_max_h = eh;
            }

            col += 1;
            if col >= COLS {
                col = 0;
                row_y += row_max_h + V_GAP;
                row_max_h = 0.0;
            }
        }

        // Advance cur_y past the last (possibly partial) row
        if col > 0 {
            row_y += row_max_h; // add height of partial last row
        }
        // row_y is now the bottom of the last row of elements
        let group_bottom = row_y;

        if is_bounded {
            // Next group starts after boundary bottom + gap
            cur_y = group_bottom + BOUND_PAD + V_GAP;
        } else {
            cur_y = group_bottom + V_GAP;
        }
    }

    // SVG width: include boundary padding on right for bounded groups
    let svg_w = max_right + if any_bounded { BOUND_PAD } else { 0.0 } + SVG_LEFT;

    // SVG height: the effective bottom is the element bottom + BOUND_PAD (for bounded groups).
    // Verified from reference SVGs:
    //   c4_basic: element bottom=485, boundary bottom=535, svg_h=535+100+70=705
    //   c4_relations: element bottom=486, no boundary, svg_h=486+100+70=656
    //
    // We compute per-element effective bottom, then take max.
    let max_effective_bottom = elements
        .iter()
        .map(|(id, &(_, y, _, h))| {
            let el_bottom = y + h;
            // Check if this element is in a bounded group
            let in_boundary = diag
                .elements
                .iter()
                .find(|e| e.id == *id)
                .and_then(|e| e.boundary_id.as_ref())
                .is_some();
            el_bottom + if in_boundary { BOUND_PAD } else { 0.0 }
        })
        .fold(0.0_f64, f64::max);
    let svg_h = max_effective_bottom + 100.0 + 70.0;

    (Layout { elements }, svg_w, svg_h)
}

fn element_height(el: &C4Element) -> f64 {
    let is_person = matches!(el.el_type, C4ElementType::Person | C4ElementType::PersonExt);
    let has_descr = !el.descr.trim().is_empty();
    if is_person {
        PERSON_BASE_H + if has_descr { PERSON_DESCR_LINE_H } else { 0.0 }
    } else {
        SYSTEM_BASE_H + if has_descr { SYSTEM_DESCR_LINE_H } else { 0.0 }
    }
}

fn render_boundary(
    diag: &C4Diagram,
    boundary: &C4Boundary,
    layout: &Layout,
    ff: &str,
) -> Option<String> {
    let els: Vec<&C4Element> = diag
        .elements
        .iter()
        .filter(|e| e.boundary_id.as_deref() == Some(&boundary.id))
        .collect();
    if els.is_empty() {
        return None;
    }

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for el in &els {
        if let Some(&(ex, ey, ew, eh)) = layout.elements.get(&el.id) {
            if ex < min_x {
                min_x = ex;
            }
            if ey < min_y {
                min_y = ey;
            }
            if ex + ew > max_x {
                max_x = ex + ew;
            }
            if ey + eh > max_y {
                max_y = ey + eh;
            }
        }
    }

    let rx = min_x - BOUND_PAD;
    let ry = min_y - BOUND_TOP;
    let rw = max_x - min_x + BOUND_PAD * 2.0;
    let rh = max_y - min_y + BOUND_TOP + BOUND_PAD;
    let cx = rx + rw / 2.0;
    let label_y = ry + 8.0;
    let type_y = ry + 30.0;
    let btype_upper = boundary.boundary_type.to_uppercase();

    let mut s = String::new();
    s.push_str("<g>");

    // Rect
    s.push_str(&format!(
        "<rect x=\"{}\" y=\"{}\" fill=\"none\" stroke=\"#444444\" width=\"{}\" height=\"{}\" rx=\"2.5\" ry=\"2.5\" stroke-width=\"1\" stroke-dasharray=\"7.0,7.0\"></rect>",
        fmt(rx), fmt(ry), fmt(rw), fmt(rh)
    ));

    // Name label
    s.push_str(&center_text_bold16(
        cx,
        label_y,
        "#444444",
        &boundary.label,
        ff,
    ));

    // Type label
    s.push_str(&center_text_normal14(
        cx,
        type_y,
        "#444444",
        &format!("[{}]", btype_upper),
        ff,
    ));

    s.push_str("</g>");
    Some(s)
}

fn render_element(el: &C4Element, ex: f64, ey: f64, ew: f64, eh: f64, ff: &str) -> String {
    let is_person = matches!(el.el_type, C4ElementType::Person | C4ElementType::PersonExt);
    let is_ext = matches!(
        el.el_type,
        C4ElementType::PersonExt
            | C4ElementType::SystemExt
            | C4ElementType::SystemDbExt
            | C4ElementType::ContainerExt
            | C4ElementType::ContainerDbExt
            | C4ElementType::ComponentExt
            | C4ElementType::ComponentDbExt
            | C4ElementType::NodeExt
    );

    let (fill, stroke) = if is_person {
        ("#08427B", "#073B6F")
    } else if is_ext {
        ("#999999", "#8A8A8A")
    } else {
        ("#1168BD", "#3C7FC0")
    };

    let stereotype = stereotype_text(&el.el_type);
    let cx = ex + ew / 2.0;

    let mut s = String::new();
    s.push_str("<g class=\"person-man\">");

    // Background rect
    s.push_str(&format!(
        "<rect x=\"{}\" y=\"{}\" fill=\"{}\" stroke=\"{}\" width=\"{}\" height=\"{}\" rx=\"2.5\" ry=\"2.5\" stroke-width=\"0.5\"></rect>",
        fmt(ex), fmt(ey), fill, stroke, fmt(ew), fmt(eh)
    ));

    // Stereotype text at ey+20
    let stereo_y = ey + 20.0;
    let stereo_len = estimate_stereo_len(stereotype);
    let stereo_x = cx - stereo_len as f64 / 2.0;
    s.push_str(&format!(
        "<text fill=\"#FFFFFF\" font-family=\"{ff}\" font-size=\"12\" font-style=\"italic\" lengthAdjust=\"spacing\" textLength=\"{}\" x=\"{}\" y=\"{}\">{}</text>",
        stereo_len, fmt(stereo_x), fmt(stereo_y), esc(stereotype)
    ));

    if is_person {
        // Person: embedded 48x48 PNG icon
        let img_x = cx - 24.0;
        let img_y = stereo_y + 10.0;
        s.push_str(&format!(
            "<image width=\"48\" height=\"48\" x=\"{}\" y=\"{}\" xlink:href=\"data:image/png;base64,{}\"></image>",
            fmt(img_x), fmt(img_y), PERSON_PNG
        ));

        // Name label: below image (img_y + 48 + 8)
        let label_y = img_y + 56.0;
        s.push_str(&center_text_bold16(cx, label_y, "#FFFFFF", &el.label, ff));

        // Description
        if !el.descr.trim().is_empty() {
            let descr_y = label_y + 37.0;
            s.push_str(&center_text_normal14(cx, descr_y, "#FFFFFF", &el.descr, ff));
        }
    } else {
        // System box: label at ey+38
        let label_y = ey + 38.0;
        s.push_str(&center_text_bold16(cx, label_y, "#FFFFFF", &el.label, ff));

        if !el.descr.trim().is_empty() {
            let descr_y = label_y + 37.0;
            s.push_str(&center_text_normal14(cx, descr_y, "#FFFFFF", &el.descr, ff));
        }
    }

    s.push_str("</g>");
    s
}

fn center_text_bold16(cx: f64, y: f64, fill: &str, text: &str, ff: &str) -> String {
    format!(
        "<text x=\"{}\" y=\"{}\" dominant-baseline=\"middle\" fill=\"{}\" style=\"text-anchor: middle; font-size: 16px; font-weight: bold; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">{}</tspan></text>",
        fmt(cx), fmt(y), fill, esc(text)
    )
}

fn center_text_normal14(cx: f64, y: f64, fill: &str, text: &str, ff: &str) -> String {
    format!(
        "<text x=\"{}\" y=\"{}\" dominant-baseline=\"middle\" fill=\"{}\" style=\"text-anchor: middle; font-size: 14px; font-weight: normal; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">{}</tspan></text>",
        fmt(cx), fmt(y), fill, esc(text)
    )
}

fn estimate_stereo_len(s: &str) -> u32 {
    match s {
        "<<person>>" => 50,
        "<<system>>" => 52,
        "<<external_system>>" => 101,
        "<<database>>" => 66,
        "<<container>>" => 68,
        "<<external_container>>" => 113,
        "<<component>>" => 72,
        "<<external_component>>" => 116,
        "<<external_database>>" => 110,
        "<<node>>" => 44,
        "<<external_node>>" => 89,
        _ => (s.len() as f64 * 5.5) as u32,
    }
}

#[allow(clippy::too_many_arguments)]
fn render_rel(
    rel: &C4Rel,
    fx: f64,
    fy: f64,
    fw: f64,
    fh: f64,
    tx: f64,
    ty: f64,
    tw: f64,
    th: f64,
    svg_id: &str,
    ff: &str,
) -> String {
    let is_bi = rel.rel_type == C4RelType::BiRel;

    let f_cx = fx + fw / 2.0;
    let f_cy = fy + fh / 2.0;
    let t_cx = tx + tw / 2.0;
    let t_cy = ty + th / 2.0;

    let (sx, sy) = edge_intersection(f_cx, f_cy, t_cx, t_cy, fx, fy, fw, fh);
    let (ex, ey) = edge_intersection(t_cx, t_cy, f_cx, f_cy, tx, ty, tw, th);

    let mid_x = (sx + ex) / 2.0;
    let mid_y = (sy + ey) / 2.0;

    let mut s = String::new();

    let dx_abs = (ex - sx).abs();
    let dy_abs = (ey - sy).abs();
    let is_straight = dx_abs.min(dy_abs) < 20.0;

    // Control point for the quadratic bezier (curved edges only)
    let qx = mid_x;
    let qy = mid_y + 74.0;

    // Label position:
    //   straight lines → above the midpoint (−12px)
    //   curved paths   → at the bezier midpoint (t=0.5: 0.25*P0 + 0.5*Q + 0.25*P2)
    let (lbl_x, lbl_y) = if is_straight {
        (mid_x, mid_y - 12.0)
    } else {
        // Quadratic bezier at t=0.5
        (
            0.25 * sx + 0.5 * qx + 0.25 * ex,
            0.25 * sy + 0.5 * qy + 0.25 * ey,
        )
    };

    if is_straight {
        let ms = if is_bi {
            format!(" marker-start=\"url(#{id}-arrowend)\"", id = svg_id)
        } else {
            String::new()
        };
        s.push_str(&format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke-width=\"1\" stroke=\"#444444\" marker-end=\"url(#{}-arrowhead)\"{}  style=\"fill: none;\"></line>",
            fmt(sx), fmt(sy), fmt(ex), fmt(ey), svg_id, ms
        ));
    } else {
        s.push_str(&format!(
            "<path fill=\"none\" stroke-width=\"1\" stroke=\"#444444\" d=\"M{},{} Q{},{} {},{}\" marker-end=\"url(#{}-arrowhead)\"></path>",
            fmt(sx), fmt(sy), fmt(qx), fmt(qy), fmt(ex), fmt(ey), svg_id
        ));
    }

    if !rel.label.is_empty() {
        s.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" dominant-baseline=\"middle\" fill=\"#444444\" style=\"text-anchor: middle; font-size: 12px; font-weight: normal; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">{}</tspan></text>",
            fmt(lbl_x), fmt(lbl_y), esc(&rel.label)
        ));
    }

    if !rel.techn.is_empty() {
        s.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" dominant-baseline=\"middle\" fill=\"#444444\" style=\"text-anchor: middle; font-size: 10px; font-style: italic; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">[{}]</tspan></text>",
            fmt(lbl_x), fmt(lbl_y + 14.0), esc(&rel.techn)
        ));
    }

    s
}

#[allow(clippy::too_many_arguments)]
fn edge_intersection(
    cx: f64,
    cy: f64,
    ox: f64,
    oy: f64,
    bx: f64,
    by: f64,
    bw: f64,
    bh: f64,
) -> (f64, f64) {
    let dx = ox - cx;
    let dy = oy - cy;
    if dx.abs() < 1e-9 && dy.abs() < 1e-9 {
        return (cx, cy);
    }

    let mut best_t = f64::MAX;
    let mut best_x = cx;
    let mut best_y = cy;

    if dx > 0.0 {
        let t = (bx + bw - cx) / dx;
        let y = cy + t * dy;
        if t > 0.0 && y >= by && y <= by + bh && t < best_t {
            best_t = t;
            best_x = bx + bw;
            best_y = y;
        }
    }
    if dx < 0.0 {
        let t = (bx - cx) / dx;
        let y = cy + t * dy;
        if t > 0.0 && y >= by && y <= by + bh && t < best_t {
            best_t = t;
            best_x = bx;
            best_y = y;
        }
    }
    if dy > 0.0 {
        let t = (by + bh - cy) / dy;
        let x = cx + t * dx;
        if t > 0.0 && x >= bx && x <= bx + bw && t < best_t {
            best_t = t;
            best_x = x;
            best_y = by + bh;
        }
    }
    if dy < 0.0 {
        let t = (by - cy) / dy;
        let x = cx + t * dx;
        if t > 0.0 && x >= bx && x <= bx + bw && t < best_t {
            best_t = t;
            best_x = x;
            best_y = by;
        }
    }

    let _ = best_t;
    (best_x, best_y)
}

fn stereotype_text(el_type: &C4ElementType) -> &'static str {
    match el_type {
        C4ElementType::Person => "<<person>>",
        C4ElementType::PersonExt => "<<person>>",
        C4ElementType::System => "<<system>>",
        C4ElementType::SystemExt => "<<external_system>>",
        C4ElementType::SystemDb => "<<database>>",
        C4ElementType::SystemDbExt => "<<external_system>>",
        C4ElementType::Container => "<<container>>",
        C4ElementType::ContainerExt => "<<external_container>>",
        C4ElementType::ContainerDb => "<<database>>",
        C4ElementType::ContainerDbExt => "<<external_database>>",
        C4ElementType::Component => "<<component>>",
        C4ElementType::ComponentExt => "<<external_component>>",
        C4ElementType::ComponentDb => "<<database>>",
        C4ElementType::ComponentDbExt => "<<external_database>>",
        C4ElementType::Node => "<<node>>",
        C4ElementType::NodeExt => "<<external_node>>",
    }
}

fn build_style(id: &str, ff: &str) -> String {
    let mut c = String::new();
    c.push_str(&format!(
        "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}"
    ));
    c.push_str(&format!("#{id} p{{margin:0;}}"));
    c.push_str(&format!(
        "#{id} .person{{stroke:hsl(240, 60%, 86.2745098039%);fill:#ECECFF;}}"
    ));
    c.push_str(&format!("#{id} :root{{--mermaid-font-family:{ff};}}"));
    c
}

fn fmt(v: f64) -> String {
    let s = format!("{:.4}", v);
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
    #[ignore = "platform-specific float precision — run locally"]
    fn snapshot_default_theme() {
        let input = "C4Context\n      title System Context diagram for Internet Banking System\n      Enterprise_Boundary(b0, \"BankBoundary0\") {\n        Person(customerA, \"Banking Customer A\")\n        Person(customerB, \"Banking Customer B\")\n        System(SystemAA, \"Internet Banking System\")\n      }";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(svg);
    }
}
