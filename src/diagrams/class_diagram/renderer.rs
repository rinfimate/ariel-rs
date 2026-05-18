// Mermaid class diagram renderer — faithful port of classRenderer-v3.ts
// Uses dagre for layout; SVG structure mirrors reference output.

use super::constants::*;
use super::parser::{ClassDiagram, ClassNode, ClassRelation, EndType, LineStyle};
#[allow(unused_imports)]
use super::templates;
use crate::text::measure;
use crate::theme::{Theme, ThemeVars};
use dagre_dgl_rs::graph::{EdgeLabel, Graph, GraphLabel, NodeLabel, Point};
use dagre_dgl_rs::layout::layout;

// ─── Public entry points ──────────────────────────────────────────────────────

pub fn render(diag: &ClassDiagram, theme: Theme, use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    render_inner(diag, &vars, use_foreign_object)
}

fn render_inner(diag: &ClassDiagram, vars: &ThemeVars, use_foreign_object: bool) -> String {
    let mut g = Graph::with_options(false, true, true);
    g.set_graph(GraphLabel {
        rankdir: Some(diag.direction.clone()),
        nodesep: Some(50.0),
        ranksep: Some(50.0),
        marginx: Some(8.0),
        marginy: Some(8.0),
        ..Default::default()
    });

    // Compute node sizes and add to graph
    let node_sizes: Vec<(String, f64, f64)> = diag
        .class_order
        .iter()
        .filter_map(|id| {
            let cls = diag.classes.get(id)?;
            let (w, h) = class_box_size(cls);
            Some((id.clone(), w, h))
        })
        .collect();

    for (id, w, h) in &node_sizes {
        g.set_node(
            id,
            NodeLabel {
                width: *w,
                height: *h,
                ..Default::default()
            },
        );
    }

    // Add edges — include label dimensions so dagre expands ranksep for labeled edges.
    // Edge labels in class diagrams are 24 px tall (one line); width is measured text.
    // Apply CONTENT_SCALE so dagre reserves the same space as the browser-rendered label.
    for (i, rel) in diag.relations.iter().enumerate() {
        let key = Some(format!("e{}", i));
        let (lbl_w, lbl_h) = if !rel.title.is_empty() {
            let (tw, _) = measure(&rel.title, FONT_SIZE);
            (tw * CONTENT_SCALE, 24.0)
        } else {
            (0.0, 0.0)
        };
        g.set_edge(
            &rel.id1,
            &rel.id2,
            EdgeLabel {
                minlen: Some(1),
                weight: Some(1.0),
                width: Some(lbl_w),
                height: Some(lbl_h),
                labelpos: Some("c".to_string()),
                ..Default::default()
            },
            key.as_deref(),
        );
    }

    layout(&mut g);

    let graph_w_dagre = g.graph().width.unwrap_or(200.0);
    let graph_h = g.graph().height.unwrap_or(200.0);

    // Mermaid's setupViewPortForSVG uses getBBox() on the full SVG, which includes cardinality
    // (edgeTerminal) labels.  In Mermaid's DOM structure, the terminal label foreignObject is a
    // direct child of the edgeTerminals group (no centering offset), so its CSS-styled width
    // (text.len() * 9 px) adds to the right edge of the bounding box.  We replicate that here
    // so our viewBox width matches the reference.
    //
    // Formula (mirrors Mermaid/browser getBBox result):
    //   content_right = max over all terminal labels of (terminal_cx + style_w_px)
    //   content_left  = marginx (leftmost node left edge ≈ marginx after dagre translate)
    //   viewBox_width = max(graph_w_dagre, content_right + marginx)
    let margin_x = 8.0_f64;
    let mut max_terminal_right: f64 = 0.0;
    if use_foreign_object {
        let terminal_marker_size: f64 = 10.0;
        for (i, rel) in diag.relations.iter().enumerate() {
            let edge_key = format!("e{}", i);
            let e = dagre_dgl_rs::graph::Edge::named(&rel.id1, &rel.id2, &edge_key);
            if let Some(lbl_data) = g.edge(&e) {
                let pts = lbl_data.points.clone().unwrap_or_default();
                if pts.len() >= 2 {
                    if !rel.title1.is_empty() {
                        let (cx, _) = calc_terminal_label_position(
                            terminal_marker_size,
                            TerminalPos::StartRight,
                            &pts,
                        );
                        let style_w = (rel.title1.len() * 9) as f64;
                        max_terminal_right = max_terminal_right.max(cx + style_w);
                    }
                    if !rel.title2.is_empty() {
                        let (cx, _) = calc_terminal_label_position(
                            terminal_marker_size,
                            TerminalPos::EndLeft,
                            &pts,
                        );
                        let style_w = (rel.title2.len() * 9) as f64;
                        max_terminal_right = max_terminal_right.max(cx + style_w);
                    }
                }
            }
        }
    }
    let graph_w = f64::max(graph_w_dagre, max_terminal_right + margin_x);

    let svg_id = "mermaid-svg";
    let css = build_css(svg_id, vars);

    let mut out = String::new();

    out.push_str(&format!(
        r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" class="classDiagram" style="max-width: {w}px;" viewBox="0 0 {w} {h}" role="graphics-document document" aria-roledescription="class">"#,
        id = svg_id,
        w = fmt(graph_w),
        h = fmt(graph_h),
    ));

    out.push_str("<style>");
    out.push_str(&css);
    out.push_str("</style>");

    out.push_str("<g>");
    out.push_str(&build_markers(svg_id));
    out.push_str("</g>");

    out.push_str(r#"<g class="root">"#);

    // clusters (none for basic class diagrams)
    out.push_str(r#"<g class="clusters"></g>"#);

    // edgePaths
    out.push_str(r#"<g class="edgePaths">"#);
    for (i, rel) in diag.relations.iter().enumerate() {
        let edge_key = format!("e{}", i);
        let e = dagre_dgl_rs::graph::Edge::named(&rel.id1, &rel.id2, &edge_key);
        if let Some(lbl) = g.edge(&e) {
            let pts = lbl.points.clone().unwrap_or_default();
            if pts.len() >= 2 {
                let edge_id = format!("{}-id_{}_{}_{}", svg_id, rel.id1, rel.id2, i + 1);
                let pts = trim_end(
                    &trim_start(&pts, start_trim(&rel.start)),
                    end_trim(&rel.end),
                );
                let path_d = edge_path(&pts);
                let is_dashed = rel.line_style == LineStyle::Dashed;
                let classes = if is_dashed {
                    " edge-thickness-normal edge-pattern-dashed relation"
                } else {
                    " edge-thickness-normal edge-pattern-solid relation"
                };
                let marker_start = marker_start_attr(svg_id, rel);
                let marker_end = marker_end_attr(svg_id, rel);
                out.push_str(&format!(
                    r#"<path d="{d}" id="{eid}" class="{cls}" style=";;;" data-edge="true" data-et="edge" data-id="{eid}" data-look="classic"{ms}{me}></path>"#,
                    d = path_d,
                    eid = edge_id,
                    cls = classes,
                    ms = marker_start,
                    me = marker_end,
                ));
            }
        }
    }
    out.push_str("</g>");

    // edgeLabels
    out.push_str(r#"<g class="edgeLabels">"#);
    for (i, rel) in diag.relations.iter().enumerate() {
        let edge_key = format!("e{}", i);
        let e = dagre_dgl_rs::graph::Edge::named(&rel.id1, &rel.id2, &edge_key);
        if let Some(lbl_data) = g.edge(&e) {
            let pts = lbl_data.points.clone().unwrap_or_default();
            let edge_id = format!("{}-id_{}_{}_{}", svg_id, rel.id1, rel.id2, i + 1);
            if !rel.title.is_empty() {
                let mid = midpoint(&pts);
                let (raw_fo_w, _) = measure(&rel.title, TITLE_FONT_SIZE);
                let fo_w = raw_fo_w * CONTENT_SCALE;
                if use_foreign_object {
                    out.push_str(&format!(
                        r#"<g class="edgeLabel" transform="translate({mx}, {my})"><g class="label" data-id="{eid}" transform="translate({ox}, -12)"><foreignObject width="{fw}" height="24"><div xmlns="http://www.w3.org/1999/xhtml" class="labelBkg" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;"><span class="edgeLabel "><p>{text}</p></span></div></foreignObject></g></g>"#,
                        mx = fmt(mid.0), my = fmt(mid.1),
                        eid = edge_id,
                        ox = fmt(-fo_w / 2.0),
                        fw = fmt(fo_w),
                        text = esc(&rel.title),
                    ));
                } else {
                    out.push_str(&format!(
                        r##"<g class="edgeLabel" transform="translate({mx}, {my})"><rect x="{ox}" y="-12" width="{fw}" height="24" fill="{pf}" stroke="none"></rect><text x="0" y="5" text-anchor="middle" font-family="{ff}" font-size="16" fill="#131300">{text}</text></g>"##,
                        mx = fmt(mid.0), my = fmt(mid.1),
                        ox = fmt(-fo_w / 2.0),
                        fw = fmt(fo_w),
                        pf = vars.primary_color,
                        ff = vars.font_family,
                        text = esc(&rel.title),
                    ));
                }
            } else {
                out.push_str(&format!(
                    r#"<g class="edgeLabel"><g class="label" data-id="{eid}" transform="translate(0, 0)"><foreignObject width="0" height="0"><div xmlns="http://www.w3.org/1999/xhtml" class="labelBkg" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;"><span class="edgeLabel "></span></div></foreignObject></g></g>"#,
                    eid = edge_id,
                ));
            }

            // Render start/end cardinality labels (title1 = near id1, title2 = near id2)
            // Faithfully ports Mermaid's calcTerminalLabelPosition algorithm from utils.ts
            // and positionEdgeLabel from edges.js:
            //   title1 → 'start_right' position (left of edge near source)
            //   title2 → 'end_left'   position (right of edge near target)
            if use_foreign_object {
                let render_card_label = |text: &str, cx: f64, cy: f64| -> String {
                    // CSS `.edgeTerminals{font-size:11px}` — measure at 11px with TEXT_SCALE
                    const TERMINAL_SCALE: f64 = 1.117;
                    let (fw_raw, _) = measure(text, 11.0);
                    let fw = fw_raw * TERMINAL_SCALE;
                    // width in style = char_count * 9px (Mermaid's setTerminalWidth)
                    let style_w = text.len() * 9;
                    format!(
                        r#"<g class="edgeTerminals" transform="translate({cx}, {cy})"><g class="inner" transform="translate(0, -8.25)"><foreignObject width="{fw}" height="16.5" style="width: {sw}px; height: 12px;"><div xmlns="http://www.w3.org/1999/xhtml" style="display: table-cell; white-space: nowrap; line-height: 1.5;"><span class="edgeLabel "><p>{text}</p></span></div></foreignObject></g></g>"#,
                        cx = fmt(cx),
                        cy = fmt(cy),
                        fw = fmt(fw),
                        sw = style_w,
                        text = esc(text),
                    )
                };
                // terminalMarkerSize: Mermaid passes `edge.arrowTypeStart ? 10 : 0`.
                // In Mermaid class-diagram rendering the arrowType strings are always set
                // (e.g. 'none', 'dependencyEnd', etc.) so arrowTypeStart/End are always
                // truthy JS strings — both terminals always receive terminalMarkerSize = 10.
                let terminal_marker_size: f64 = 10.0;

                if !rel.title1.is_empty() && pts.len() >= 2 {
                    // title1 near source → 'start_right' position
                    let (cx, cy) = calc_terminal_label_position(
                        terminal_marker_size,
                        TerminalPos::StartRight,
                        &pts,
                    );
                    out.push_str(&render_card_label(&rel.title1, cx, cy));
                }
                if !rel.title2.is_empty() && pts.len() >= 2 {
                    // title2 near target → 'end_left' position
                    // Apply render-time offset (+10x, +3y) so wider labels clear the edge line.
                    // This does NOT affect layout calculation (done separately above).
                    let (cx, cy) = calc_terminal_label_position(
                        terminal_marker_size,
                        TerminalPos::EndLeft,
                        &pts,
                    );
                    out.push_str(&render_card_label(&rel.title2, cx + 0.0, cy + 7.0));
                }
            }
        }
    }
    out.push_str("</g>");

    // nodes
    out.push_str(r#"<g class="nodes">"#);
    for (class_idx, id) in diag.class_order.iter().enumerate() {
        if let Some(cls) = diag.classes.get(id) {
            if let Some(n) = g.node_opt(id) {
                let cx = n.x.unwrap_or(0.0);
                let cy = n.y.unwrap_or(0.0);
                let w = n.width;
                let h = n.height;
                let dom_id = format!("{}-classId-{}-{}", svg_id, id, class_idx);
                out.push_str(&render_class_node(
                    cls,
                    cx,
                    cy,
                    w,
                    h,
                    vars,
                    &dom_id,
                    use_foreign_object,
                ));
            }
        }
    }
    out.push_str("</g>");

    out.push_str("</g>"); // root

    out.push_str(&format!(
        "<defs><filter id=\"{0}-drop-shadow\" height=\"130%\" width=\"130%\"><feDropShadow dx=\"4\" dy=\"4\" stdDeviation=\"0\" flood-opacity=\"0.06\" flood-color=\"#000000\"></feDropShadow></filter></defs>",
        svg_id
    ));
    out.push_str(&format!(
        "<defs><filter id=\"{0}-drop-shadow-small\" height=\"150%\" width=\"150%\"><feDropShadow dx=\"2\" dy=\"2\" stdDeviation=\"0\" flood-opacity=\"0.06\" flood-color=\"#000000\"></feDropShadow></filter></defs>",
        svg_id
    ));

    out.push_str("</svg>");
    out
}

// ─── Class box sizing ─────────────────────────────────────────────────────────

/// Returns the height of a non-empty section (n > 0 guaranteed).
/// Mermaid: (n+1)*24px per section when rows > 0.
fn section_h_nonzero(rows: usize) -> f64 {
    (rows as f64 + 1.0) * MEMBER_ROW_H
}

/// Compute the total width and height of a class box.
///
/// Width formula mirrors Mermaid's DOM-based layout (classBox.ts / textHelper):
///   • Annotation and label groups are **centred** at x=0.
///   • Member and method groups are **left-aligned** starting at x=0.
///   • After layout the shapeSvg bbox spans:
///       x_min = −max(ann_w, name_w) / 2
///       x_max = max(max(ann_w, name_w)/2, max_content_w)
///       bbox_w = x_max − x_min
///   • The enclosing rectangle adds H_PAD on each side:
///       hw = bbox_w / 2 + H_PAD   →   full_w = bbox_w + 2*H_PAD
fn class_box_size(cls: &ClassNode) -> (f64, f64) {
    // ── Centred items: class name + annotations ──────────────────────────────
    // Apply NAME_SCALE to the bold class name and CONTENT_SCALE to italic annotations,
    // matching browser foreignObject rendering widths derived from reference SVGs.
    let (raw_name_w, _) = measure(&cls.label, FONT_SIZE);
    let name_w = raw_name_w * NAME_SCALE;

    let mut max_centred_w: f64 = name_w;
    for ann in &cls.annotations {
        // Use actual guillemet characters (U+00AB, U+00BB) that Mermaid displays —
        // these are narrower than ASCII "<<>>" and match the reference foreignObject widths.
        let ann_text = format!("\u{00AB}{}\u{00BB}", ann);
        let (raw_w, _) = measure(&ann_text, FONT_SIZE);
        max_centred_w = max_centred_w.max(raw_w * CONTENT_SCALE);
    }

    // ── Left-aligned items: members + methods ────────────────────────────────
    // Apply CONTENT_SCALE to regular text (member/method display strings).
    let mut max_content_w: f64 = 0.0;
    for m in &cls.members {
        let (raw_w, _) = measure(&m.display_text(), FONT_SIZE);
        max_content_w = max_content_w.max(raw_w * CONTENT_SCALE);
    }
    for m in &cls.methods {
        let (raw_w, _) = measure(&m.display_text(), FONT_SIZE);
        max_content_w = max_content_w.max(raw_w * CONTENT_SCALE);
    }

    // ── shapeSvg bbox width ──────────────────────────────────────────────────
    let half_centred = max_centred_w / 2.0;
    let x_max = f64::max(half_centred, max_content_w);
    let bbox_w = x_max + half_centred; // x_max − (−half_centred)

    // ── Full box width = bbox_w + 2*H_PAD, with a minimum ───────────────────
    let w = (bbox_w + H_PAD * 2.0).max(MIN_BOX_W);

    // ── Height ───────────────────────────────────────────────────────────────
    //   annotations:      ann_rows * 24
    //   header:           48  (always)
    //   members section:  section_h(member_rows)  — 18 if empty, (n+1)*24 if non-empty
    //   methods section:  section_h(method_rows)
    //
    // Mermaid DOM observation: when annotations are present and the members section
    // is empty, the classBox.ts bounding-box calculation produces an extra 6px for
    // the empty members section (24 instead of 18) AND an extra 6px in the methods
    // section when methods are non-empty.  Together this adds 12px in that case.
    let ann_rows = cls.annotations.len();
    let member_rows = cls.members.len();
    let method_rows = cls.methods.len();

    // Section heights — derived from Mermaid classBox.ts DOM measurements.
    // When one section is empty and the other is not, Mermaid's GAP/2=6px floor on
    // membersGroupHeight shifts the layout, producing section sizes of 24px (not 18).
    //   (m=0, me=0): members=18, methods=18
    //   (m>0, me=0): members=(m+1)*24, methods=24       ← methods floor = 24
    //   (m=0, me>0): members=24, methods=(me+1)*24 + 6  ← members floor=24; +6 shift
    //   (m>0, me>0): members=(m+1)*24, methods=(me+1)*24
    let (members_h, methods_h) = match (member_rows, method_rows) {
        (0, 0) => (EMPTY_SECTION_H, EMPTY_SECTION_H),
        (m, 0) => (section_h_nonzero(m), MEMBER_ROW_H),
        (0, me) => (MEMBER_ROW_H, section_h_nonzero(me) + 6.0),
        (m, me) => (section_h_nonzero(m), section_h_nonzero(me)),
    };

    let h = ann_rows as f64 * ANNOTATION_H + HEADER_H + members_h + methods_h;

    (w, h)
}

// ─── Node rendering ───────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn render_class_node(
    cls: &ClassNode,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    vars: &ThemeVars,
    dom_id: &str,
    use_foreign_object: bool,
) -> String {
    let hw = w / 2.0;
    let hh = h / 2.0;
    let pb = vars.primary_border;
    let pf = vars.primary_color;

    let mut s = String::new();
    s.push_str(&format!(
        r#"<g class="node default " id="{did}" data-look="classic" transform="translate({cx}, {cy})">"#,
        did = dom_id, cx = fmt(cx), cy = fmt(cy),
    ));

    // Outer rectangle (filled, no stroke for shadow layer)
    s.push_str(&format!(
        r#"<g class="basic label-container outer-path"><path d="M{x1} {y1} L{x2} {y1} L{x2} {y2} L{x1} {y2}" stroke="none" stroke-width="0" fill="{pf}" style=""></path>"#,
        x1 = fmt(-hw), y1 = fmt(-hh), x2 = fmt(hw), y2 = fmt(hh), pf = pf,
    ));
    // Sketchy border path (matches Mermaid neo-classic look)
    s.push_str(&format!(
        r#"<path d="M{x1} {y1} C{cx1} {y1},{cx2} {y1},{x2} {y1} M{x2} {y1} C{x2} {cy1},{x2} {cy2},{x2} {y2} M{x2} {y2} C{cx3} {y2},{cx4} {y2},{x1} {y2} M{x1} {y2} C{x1} {cy3},{x1} {cy4},{x1} {y1}" stroke="{pb}" stroke-width="1.3" fill="none" stroke-dasharray="0 0" style=""></path></g>"#,
        x1 = fmt(-hw), y1 = fmt(-hh), x2 = fmt(hw), y2 = fmt(hh),
        cx1 = fmt(-hw * 0.6), cx2 = fmt(hw * 0.4),
        cx3 = fmt(hw * 0.5), cx4 = fmt(-hw * 0.2),
        cy1 = fmt(-hh * 0.6), cy2 = fmt(hh * 0.5),
        cy3 = fmt(hh * 0.5), cy4 = fmt(-hh * 0.1),
        pb = pb,
    ));

    // Y layout (all positions relative to node centre = 0, box spans -hh to +hh):
    //
    //   -hh ───────────────────── box top
    //        ann_rows * 24        annotation rows
    //   div1 ──────────────────── members divider  (= -hh + ann*24 + 48)
    //        section_h(members)   member rows       (0 rows → 18, n rows → (n+1)*24)
    //   div2 ──────────────────── methods divider  (= div1 + section_h(members))
    //        section_h(methods)   method rows
    //   +hh ───────────────────── box bottom
    //
    // Group positions:
    //   annotation_group_y = -hh (box top; row i centred at -hh + i*24 + 12)
    //   label_group_y = -hh + ann*24 + HEADER_H/2 (= centre of header section)
    //   members_group_y = div1 + MEMBER_ROW_H (first row centred at group_y + 0)
    //   methods_group_y = div2 + MEMBER_ROW_H (first row centred at group_y + 0)

    let ann_rows = cls.annotations.len();
    let member_rows = cls.members.len();

    let method_rows = cls.methods.len();

    let ann_top_y = -hh;
    let div1_y = ann_top_y + ann_rows as f64 * ANNOTATION_H + HEADER_H;

    // Members section height — must match class_box_size() exactly.
    let members_section_h = match (member_rows, method_rows) {
        (0, 0) => EMPTY_SECTION_H,
        (0, _) => MEMBER_ROW_H, // floor = 24
        (m, _) => section_h_nonzero(m),
    };
    let div2_y = div1_y + members_section_h;

    // ── Annotation group ────────────────────────────────────────────────────────
    // Vertically center the annotation+class_name block within the combined region.
    // region_h = ann_rows*24 + HEADER_H; content_h = (ann_rows+1)*24 (each row 24px).
    // vert_pad = (region_h - content_h) / 2  →  offsets annotation away from box top.
    let region_h = ann_rows as f64 * ANNOTATION_H + HEADER_H;
    let content_h = (ann_rows as f64 + 1.0) * ANNOTATION_H;
    let vert_pad = (region_h - content_h) / 2.0;
    let ann_group_y = if ann_rows > 0 {
        ann_top_y + vert_pad
    } else {
        ann_top_y + ann_rows as f64 * ANNOTATION_H + HEADER_H / 2.0
    };
    s.push_str(&format!(
        r#"<g class="annotation-group text" transform="translate(0, {})">"#,
        fmt(ann_group_y),
    ));
    for (i, ann) in cls.annotations.iter().enumerate() {
        // Row i centre is at ann_top_y + i*24 + 12 (absolute).
        // Relative to ann_group_y (= ann_top_y): offset = i*24 + 12.
        let ann_text = format!("\u{00AB}{}\u{00BB}", esc(ann));
        let row_centre_rel = i as f64 * ANNOTATION_H + ANNOTATION_H / 2.0;
        let (raw_ann_w, _) = measure(&format!("\u{00AB}{}\u{00BB}", ann), FONT_SIZE);
        let ann_w = raw_ann_w * CONTENT_SCALE;
        if use_foreign_object {
            s.push_str(&format!(
                r#"<g class="label" style="font-style: italic" transform="translate({ox}, {y})"><foreignObject width="{fw}" height="24"><div xmlns="http://www.w3.org/1999/xhtml" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;"><span class="nodeLabel markdown-node-label" style=""><p>{text}</p></span></div></foreignObject></g>"#,
                ox = fmt(-ann_w / 2.0),
                y  = fmt(row_centre_rel - ANNOTATION_H / 2.0),
                fw = fmt(ann_w),
                text = ann_text,
            ));
        } else {
            s.push_str(&format!(
                r#"<text x="0" y="{y}" text-anchor="middle" font-family="Arial,sans-serif" font-size="{fs}" fill="{pb}" font-style="italic">{text}</text>"#,
                y = fmt(row_centre_rel), fs = FONT_SIZE, pb = pb, text = ann_text,
            ));
        }
    }
    s.push_str("</g>");

    // ── Label group (class name) ─────────────────────────────────────────────────
    // Centred vertically in the header section.
    // header section runs from (ann_top_y + ann*24) to div1.
    // Centre of header = ann_top_y + ann*24 + HEADER_H/2.
    let header_centre_y = ann_top_y + ann_rows as f64 * ANNOTATION_H + HEADER_H / 2.0;
    let (raw_name_fo_w, _) = measure(&cls.label, TITLE_FONT_SIZE);
    let name_fo_w = raw_name_fo_w * NAME_SCALE;
    s.push_str(&format!(
        r#"<g class="label-group text" transform="translate({ox}, {gy})">"#,
        ox = fmt(-name_fo_w / 2.0),
        gy = fmt(header_centre_y),
    ));
    if use_foreign_object {
        s.push_str(&format!(
            r#"<g class="label" style="font-weight: bolder" transform="translate(0,-12)"><foreignObject width="{fw}" height="24"><div xmlns="http://www.w3.org/1999/xhtml" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 100px; text-align: center;"><span class="nodeLabel markdown-node-label" style=""><p>{text}</p></span></div></foreignObject></g>"#,
            fw = fmt(name_fo_w),
            text = esc(&cls.label),
        ));
    } else {
        s.push_str(&format!(
            r#"<text x="{hw}" y="5" text-anchor="middle" font-family="Arial,sans-serif" font-size="{fs}" fill="{pb}" font-weight="bold">{text}</text>"#,
            hw = fmt(name_fo_w / 2.0), fs = TITLE_FONT_SIZE, pb = pb,
            text = esc(&cls.label),
        ));
    }
    s.push_str("</g>");

    // ── Members group ────────────────────────────────────────────────────────────
    // members_group_y is the y of the group; row i centre = group_y + i*24.
    // First row centre = div1 + MEMBER_ROW_H (one full row-height below divider).
    let members_group_y = div1_y + MEMBER_ROW_H;
    s.push_str(&format!(
        r#"<g class="members-group text" transform="translate({ox}, {gy})">"#,
        ox = fmt(-hw + H_PAD),
        gy = fmt(members_group_y),
    ));
    for (i, m) in cls.members.iter().enumerate() {
        let text = m.display_text();
        let (raw_mem_w, _) = measure(&text, FONT_SIZE);
        let mem_fo_w = raw_mem_w * CONTENT_SCALE;
        // Row i centre at group_y + i*24; FO starts 12 above centre.
        let row_y = i as f64 * MEMBER_ROW_H;
        if use_foreign_object {
            s.push_str(&format!(
                r#"<g class="label" style="" transform="translate(0,{y})"><foreignObject width="{fw}" height="24"><div xmlns="http://www.w3.org/1999/xhtml" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 150px; text-align: center;"><span class="nodeLabel markdown-node-label" style=""><p>{text}</p></span></div></foreignObject></g>"#,
                y = fmt(row_y - 12.0),
                fw = fmt(mem_fo_w),
                text = esc(&text),
            ));
        } else {
            s.push_str(&format!(
                r#"<text x="0" y="{y}" font-family="Arial,sans-serif" font-size="{fs}" fill="{pb}">{text}</text>"#,
                y = fmt(row_y), fs = FONT_SIZE, pb = pb, text = esc(&text),
            ));
        }
    }
    s.push_str("</g>");

    // ── Methods group ─────────────────────────────────────────────────────────────
    let methods_group_y = div2_y + MEMBER_ROW_H;
    s.push_str(&format!(
        r#"<g class="methods-group text" transform="translate({ox}, {gy})">"#,
        ox = fmt(-hw + H_PAD),
        gy = fmt(methods_group_y),
    ));
    for (i, m) in cls.methods.iter().enumerate() {
        let text = m.display_text();
        let (raw_meth_w, _) = measure(&text, FONT_SIZE);
        let meth_fo_w = raw_meth_w * CONTENT_SCALE;
        let row_y = i as f64 * MEMBER_ROW_H;
        if use_foreign_object {
            s.push_str(&format!(
                r#"<g class="label" style="" transform="translate(0,{y})"><foreignObject width="{fw}" height="24"><div xmlns="http://www.w3.org/1999/xhtml" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;"><span class="nodeLabel markdown-node-label" style=""><p>{text}</p></span></div></foreignObject></g>"#,
                y = fmt(row_y - 12.0),
                fw = fmt(meth_fo_w),
                text = esc(&text),
            ));
        } else {
            s.push_str(&format!(
                r#"<text x="0" y="{y}" font-family="Arial,sans-serif" font-size="{fs}" fill="{pb}">{text}</text>"#,
                y = fmt(row_y), fs = FONT_SIZE, pb = pb, text = esc(&text),
            ));
        }
    }
    s.push_str("</g>");

    // ── Dividers ──────────────────────────────────────────────────────────────────
    // div1: between header and members section
    s.push_str(&format!(
        r#"<g class="divider" style=""><path d="M{x1} {y} C{cx1} {y},{cx2} {y},{x2} {y}" stroke="{pb}" stroke-width="1.3" fill="none" stroke-dasharray="0 0" style=""></path></g>"#,
        x1 = fmt(-hw), y = fmt(div1_y),
        cx1 = fmt(-hw * 0.4), cx2 = fmt(hw * 0.4),
        x2 = fmt(hw), pb = pb,
    ));
    // div2: between members and methods section
    s.push_str(&format!(
        r#"<g class="divider" style=""><path d="M{x1} {y} C{cx1} {y},{cx2} {y},{x2} {y}" stroke="{pb}" stroke-width="1.3" fill="none" stroke-dasharray="0 0" style=""></path></g>"#,
        x1 = fmt(-hw), y = fmt(div2_y),
        cx1 = fmt(-hw * 0.4), cx2 = fmt(hw * 0.4),
        x2 = fmt(hw), pb = pb,
    ));

    s.push_str("</g>"); // node
    s
}

// ─── Marker helpers ───────────────────────────────────────────────────────────

fn marker_start_attr(svg_id: &str, rel: &ClassRelation) -> String {
    match &rel.start {
        EndType::None => String::new(),
        EndType::Extension => format!(r#" marker-start="url(#{}_class-extensionStart)""#, svg_id),
        EndType::Composition => {
            format!(r#" marker-start="url(#{}_class-compositionStart)""#, svg_id)
        }
        EndType::Aggregation => {
            format!(r#" marker-start="url(#{}_class-aggregationStart)""#, svg_id)
        }
        EndType::Arrow => format!(r#" marker-start="url(#{}_class-dependencyStart)""#, svg_id),
    }
}

fn marker_end_attr(svg_id: &str, rel: &ClassRelation) -> String {
    match &rel.end {
        EndType::None => String::new(),
        EndType::Extension => format!(r#" marker-end="url(#{}_class-extensionEnd)""#, svg_id),
        EndType::Composition => format!(r#" marker-end="url(#{}_class-compositionEnd)""#, svg_id),
        EndType::Aggregation => format!(r#" marker-end="url(#{}_class-aggregationEnd)""#, svg_id),
        EndType::Arrow => format!(r#" marker-end="url(#{}_class-dependencyEnd)""#, svg_id),
    }
}

// ─── CSS ──────────────────────────────────────────────────────────────────────

fn build_css(id: &str, vars: &ThemeVars) -> String {
    let pb = vars.primary_border;
    let pf = vars.primary_color;
    let lc = vars.line_color;
    let ff = vars.font_family;
    let mut c = String::new();

    c.push_str(&format!(
        "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}"
    ));
    c.push_str("@keyframes edge-animation-frame{from{stroke-dashoffset:0;}}");
    c.push_str("@keyframes dash{to{stroke-dashoffset:0;}}");
    c.push_str(&format!("#{id} .edge-animation-slow{{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 50s linear infinite;stroke-linecap:round;}}"));
    c.push_str(&format!("#{id} .edge-animation-fast{{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 20s linear infinite;stroke-linecap:round;}}"));
    c.push_str(&format!("#{id} .error-icon{{fill:#552222;}}"));
    c.push_str(&format!(
        "#{id} .error-text{{fill:#552222;stroke:#552222;}}"
    ));
    c.push_str(&format!(
        "#{id} .edge-thickness-normal{{stroke-width:1px;}}"
    ));
    c.push_str(&format!(
        "#{id} .edge-thickness-thick{{stroke-width:3.5px;}}"
    ));
    c.push_str(&format!("#{id} .edge-pattern-solid{{stroke-dasharray:0;}}"));
    c.push_str(&format!(
        "#{id} .edge-thickness-invisible{{stroke-width:0;fill:none;}}"
    ));
    c.push_str(&format!(
        "#{id} .edge-pattern-dashed{{stroke-dasharray:3;}}"
    ));
    c.push_str(&format!(
        "#{id} .edge-pattern-dotted{{stroke-dasharray:2;}}"
    ));
    c.push_str(&format!("#{id} .marker{{fill:#333333;stroke:#333333;}}"));
    c.push_str(&format!("#{id} .marker.cross{{stroke:#333333;}}"));
    c.push_str(&format!("#{id} svg{{font-family:{ff};font-size:16px;}}"));
    c.push_str(&format!("#{id} p{{margin:0;}}"));
    c.push_str(&format!(
        "#{id} g.classGroup text{{fill:{pb};stroke:none;font-family:{ff};font-size:10px;}}"
    ));
    c.push_str(&format!(
        "#{id} g.classGroup text .title{{font-weight:bolder;}}"
    ));
    c.push_str(&format!("#{id} .cluster-label text{{fill:#333;}}"));
    c.push_str(&format!("#{id} .cluster-label span{{color:#333;}}"));
    c.push_str(&format!(
        "#{id} .cluster-label span p{{background-color:transparent;}}"
    ));
    c.push_str(&format!(
        "#{id} .cluster rect{{fill:#ffffde;stroke:#aaaa33;stroke-width:1px;}}"
    ));
    c.push_str(&format!("#{id} .cluster text{{fill:#333;}}"));
    c.push_str(&format!("#{id} .cluster span{{color:#333;}}"));
    c.push_str(&format!(
        "#{id} .nodeLabel,#{id} .edgeLabel{{color:#131300;}}"
    ));
    c.push_str(&format!(
        "#{id} .noteLabel .nodeLabel,#{id} .noteLabel .edgeLabel{{color:black;}}"
    ));
    c.push_str(&format!("#{id} .edgeLabel .label rect{{fill:{pf};}}"));
    c.push_str(&format!("#{id} .label text{{fill:#131300;}}"));
    c.push_str(&format!("#{id} .labelBkg{{background:{pf};}}"));
    c.push_str(&format!("#{id} .edgeLabel .label span{{background:{pf};}}"));
    c.push_str(&format!("#{id} .classTitle{{font-weight:bolder;}}"));
    c.push_str(&format!("#{id} .node rect,#{id} .node circle,#{id} .node ellipse,#{id} .node polygon,#{id} .node path{{fill:{pf};stroke:{pb};stroke-width:1;}}"));
    c.push_str(&format!("#{id} .divider{{stroke:{pb};stroke-width:1;}}"));
    c.push_str(&format!("#{id} g.clickable{{cursor:pointer;}}"));
    c.push_str(&format!(
        "#{id} g.classGroup rect{{fill:{pf};stroke:{pb};}}"
    ));
    c.push_str(&format!(
        "#{id} g.classGroup line{{stroke:{pb};stroke-width:1;}}"
    ));
    c.push_str(&format!(
        "#{id} .classLabel .box{{stroke:none;stroke-width:0;fill:{pf};opacity:0.5;}}"
    ));
    c.push_str(&format!(
        "#{id} .classLabel .label{{fill:{pb};font-size:10px;}}"
    ));
    c.push_str(&format!(
        "#{id} .relation{{stroke:{lc};stroke-width:1;fill:none;}}"
    ));
    c.push_str(&format!("#{id} .dashed-line{{stroke-dasharray:3;}}"));
    c.push_str(&format!("#{id} .dotted-line{{stroke-dasharray:1 2;}}"));
    c.push_str(&format!("#{id} [id$=\"-compositionStart\"],#{id} .composition{{fill:#333333!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-compositionEnd\"],#{id} .composition{{fill:#333333!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-dependencyStart\"],#{id} .dependency{{fill:#333333!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-dependencyEnd\"],#{id} .dependency{{fill:#333333!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-extensionStart\"],#{id} .extension{{fill:transparent!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-extensionEnd\"],#{id} .extension{{fill:transparent!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-aggregationStart\"],#{id} .aggregation{{fill:transparent!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-aggregationEnd\"],#{id} .aggregation{{fill:transparent!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-lollipopStart\"],#{id} .lollipop{{fill:{pf}!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-lollipopEnd\"],#{id} .lollipop{{fill:{pf}!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!(
        "#{id} .edgeTerminals{{font-size:11px;line-height:initial;}}"
    ));
    c.push_str(&format!(
        "#{id} .classTitleText{{text-anchor:middle;font-size:18px;fill:#333;}}"
    ));
    c.push_str(&format!("#{id} .edgeLabel[data-look=\"neo\"]{{background-color:rgba(232,232,232, 0.8);text-align:center;}}"));
    c.push_str(&format!(
        "#{id} .edgeLabel[data-look=\"neo\"] p{{background-color:rgba(232,232,232, 0.8);}}"
    ));
    c.push_str(&format!("#{id} .edgeLabel[data-look=\"neo\"] rect{{opacity:0.5;background-color:rgba(232,232,232, 0.8);fill:rgba(232,232,232, 0.8);}}"));
    c.push_str(&format!("#{id} .label-icon{{display:inline-block;height:1em;overflow:visible;vertical-align:-0.125em;}}"));
    c.push_str(&format!(
        "#{id} .node .label-icon path{{fill:currentColor;stroke:revert;stroke-width:revert;}}"
    ));
    c.push_str(&format!("#{id} .node .neo-node{{stroke:{pb};}}"));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node rect,#{id} [data-look=\"neo\"].cluster rect,#{id} [data-look=\"neo\"].node polygon{{stroke:{pb};filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node path{{stroke:{pb};stroke-width:1px;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node .outer-path{{filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node .neo-line path{{stroke:{pb};filter:none;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node circle{{stroke:{pb};filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node circle .state-start{{fill:#000000;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].icon-shape .icon{{fill:{pb};filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!("#{id} [data-look=\"neo\"].icon-shape .icon-neo path{{stroke:{pb};filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!("#{id} :root{{--mermaid-font-family:{ff};}}"));

    c
}

// ─── SVG Markers ─────────────────────────────────────────────────────────────

fn build_markers(id: &str) -> String {
    let mut m = String::new();
    // Aggregation (diamond, hollow)
    m.push_str(&format!(r#"<defs><marker id="{id}_class-aggregationStart" class="marker aggregation class" refX="18" refY="7" markerWidth="190" markerHeight="240" orient="auto"><path d="M 18,7 L9,13 L1,7 L9,1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-aggregationEnd" class="marker aggregation class" refX="1" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 18,7 L9,13 L1,7 L9,1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-aggregationStart-margin" class="marker aggregation class" refX="15" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><path d="M 18,7 L9,13 L1,7 L9,1 Z" style="stroke-width: 2;"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-aggregationEnd-margin" class="marker aggregation class" refX="1" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse"><path d="M 18,7 L9,13 L1,7 L9,1 Z" style="stroke-width: 2;"></path></marker></defs>"#));

    // Extension (inheritance triangle, hollow)
    m.push_str(&format!(r#"<defs><marker id="{id}_class-extensionStart" class="marker extension class" refX="18" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse"><path d="M 1,7 L18,13 V 1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-extensionEnd" class="marker extension class" refX="1" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 1,1 V 13 L18,7 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<marker id="{id}_class-extensionStart-margin" class="marker extension class" refX="18" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse" viewBox="0 0 20 14"><polygon points="10,7 18,13 18,1" style="stroke-width: 2; stroke-dasharray: 0;"></polygon></marker>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-extensionEnd-margin" class="marker extension class" refX="9" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse" viewBox="0 0 20 14"><polygon points="10,1 10,13 18,7" style="stroke-width: 2; stroke-dasharray: 0;"></polygon></marker></defs>"#));

    // Composition (diamond, filled)
    m.push_str(&format!(r#"<defs><marker id="{id}_class-compositionStart" class="marker composition class" refX="18" refY="7" markerWidth="190" markerHeight="240" orient="auto"><path d="M 18,7 L9,13 L1,7 L9,1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-compositionEnd" class="marker composition class" refX="1" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 18,7 L9,13 L1,7 L9,1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-compositionStart-margin" class="marker composition class" refX="15" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><path viewBox="0 0 15 15" d="M 18,7 L9,13 L1,7 L9,1 Z" style="stroke-width: 0;"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-compositionEnd-margin" class="marker composition class" refX="3.5" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse"><path d="M 18,7 L9,13 L1,7 L9,1 Z" style="stroke-width: 0;"></path></marker></defs>"#));

    // Dependency (open arrow)
    m.push_str(&format!(r#"<defs><marker id="{id}_class-dependencyStart" class="marker dependency class" refX="6" refY="7" markerWidth="190" markerHeight="240" orient="auto"><path d="M 5,7 L9,13 L1,7 L9,1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-dependencyEnd" class="marker dependency class" refX="13" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 18,7 L9,13 L14,7 L9,1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-dependencyStart-margin" class="marker dependency class" refX="4" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><path d="M 5,7 L9,13 L1,7 L9,1 Z" style="stroke-width: 0;"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-dependencyEnd-margin" class="marker dependency class" refX="16" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse"><path d="M 18,7 L9,13 L14,7 L9,1 Z" style="stroke-width: 0;"></path></marker></defs>"#));

    // Lollipop
    m.push_str(&format!(r#"<defs><marker id="{id}_class-lollipopStart" class="marker lollipop class" refX="13" refY="7" markerWidth="190" markerHeight="240" orient="auto"><circle fill="transparent" cx="7" cy="7" r="6"></circle></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-lollipopEnd" class="marker lollipop class" refX="1" refY="7" markerWidth="190" markerHeight="240" orient="auto"><circle fill="transparent" cx="7" cy="7" r="6"></circle></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-lollipopStart-margin" class="marker lollipop class" refX="13" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><circle fill="transparent" cx="7" cy="7" r="6" stroke-width="2"></circle></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-lollipopEnd-margin" class="marker lollipop class" refX="1" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><circle fill="transparent" cx="7" cy="7" r="6" stroke-width="2"></circle></marker></defs>"#));

    m
}

// ─── Edge path ────────────────────────────────────────────────────────────────

/// Arrowhead overhang = (tip_x - refX) for each End marker type.
/// The dagre edge endpoint lands on the node boundary; trimming pulls it back
/// so the arrowhead tip touches the boundary instead of being buried inside.
fn end_trim(end: &EndType) -> f64 {
    match end {
        EndType::Extension | EndType::Composition | EndType::Aggregation => 17.0,
        EndType::Arrow => 8.0,
        EndType::None => 0.0,
    }
}

fn start_trim(start: &EndType) -> f64 {
    match start {
        EndType::Extension | EndType::Composition | EndType::Aggregation => 17.0,
        EndType::Arrow => 8.0,
        EndType::None => 0.0,
    }
}

/// Trim `amount` units off the END of the last segment (toward source).
fn trim_end(pts: &[Point], amount: f64) -> Vec<Point> {
    if amount <= 0.0 || pts.len() < 2 {
        return pts.to_vec();
    }
    let mut result = pts.to_vec();
    let n = result.len();
    let last = result[n - 1].clone();
    let prev = result[n - 2].clone();
    let dx = last.x - prev.x;
    let dy = last.y - prev.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= amount {
        result.truncate(n - 1);
    } else {
        let frac = (len - amount) / len;
        result[n - 1] = Point {
            x: prev.x + dx * frac,
            y: prev.y + dy * frac,
        };
    }
    result
}

/// Trim `amount` units off the START of the first segment (toward target).
fn trim_start(pts: &[Point], amount: f64) -> Vec<Point> {
    if amount <= 0.0 || pts.len() < 2 {
        return pts.to_vec();
    }
    let mut result = pts.to_vec();
    let first = result[0].clone();
    let next = result[1].clone();
    let dx = next.x - first.x;
    let dy = next.y - first.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= amount {
        result.remove(0);
    } else {
        let frac = amount / len;
        result[0] = Point {
            x: first.x + dx * frac,
            y: first.y + dy * frac,
        };
    }
    result
}

fn edge_path(pts: &[Point]) -> String {
    let pairs: Vec<(f64, f64)> = pts.iter().map(|p| (p.x, p.y)).collect();
    crate::svg::curve_basis_path(&pairs)
}

fn midpoint(pts: &[Point]) -> (f64, f64) {
    if pts.is_empty() {
        return (0.0, 0.0);
    }
    let mid = pts.len() / 2;
    (pts[mid].x, pts[mid].y)
}

// ─── Terminal label positioning — faithful port of Mermaid utils.ts ──────────
//
// Mermaid uses calcTerminalLabelPosition(terminalMarkerSize, position, points)
// from packages/mermaid/src/utils.ts via positionEdgeLabel in edges.js.
//
// title1 (near source) → position 'start_right'
// title2 (near target) → position 'end_left'
//
// terminalMarkerSize = 10 when an arrow marker is present, 0 otherwise.

#[derive(Clone, Copy)]
enum TerminalPos {
    StartRight,
    EndLeft,
}

/// Port of utils.calcTerminalLabelPosition.
/// Returns (x, y) for the outer group transform of the terminal label.
fn calc_terminal_label_position(
    terminal_marker_size: f64,
    position: TerminalPos,
    points: &[Point],
) -> (f64, f64) {
    // For end positions, reverse the point list so we always traverse from source.
    let fwd: Vec<(f64, f64)> = points.iter().map(|p| (p.x, p.y)).collect();
    let rev: Vec<(f64, f64)> = fwd.iter().cloned().rev().collect();
    let pts_owned: Vec<(f64, f64)> = match position {
        TerminalPos::StartRight => fwd,
        TerminalPos::EndLeft => rev,
    };
    let pts_ref: &[(f64, f64)] = &pts_owned;

    let distance_to_cardinality_point = 25.0 + terminal_marker_size;
    // We need calculatePoint over Point slices — use a helper that accepts (f64,f64) tuples.
    let center = {
        let mut prev: Option<(f64, f64)> = None;
        let mut remaining = distance_to_cardinality_point;
        let mut result = pts_ref[pts_ref.len() - 1];
        for &p in pts_ref {
            if let Some(prev_p) = prev {
                let dx = p.0 - prev_p.0;
                let dy = p.1 - prev_p.1;
                let seg_len = (dx * dx + dy * dy).sqrt();
                if seg_len == 0.0 {
                    prev = Some(p);
                    continue;
                }
                if seg_len < remaining {
                    remaining -= seg_len;
                } else {
                    let ratio = remaining / seg_len;
                    result = (
                        (1.0 - ratio) * prev_p.0 + ratio * p.0,
                        (1.0 - ratio) * prev_p.1 + ratio * p.1,
                    );
                    break;
                }
            }
            prev = Some(p);
        }
        result
    };

    let d = 10.0 + terminal_marker_size * 0.5;
    let p0 = pts_ref[0];
    let angle = f64::atan2(p0.1 - center.1, p0.0 - center.0);

    let (x, y) = match position {
        TerminalPos::StartRight => {
            // sin(angle)*d + (p0.x + center.x)/2
            // -cos(angle)*d + (p0.y + center.y)/2
            let x = angle.sin() * d + (p0.0 + center.0) / 2.0;
            let y = -angle.cos() * d + (p0.1 + center.1) / 2.0;
            (x, y)
        }
        TerminalPos::EndLeft => {
            // Mermaid source: sin(angle)*d + (p0.x+center.x)/2 - 5
            //                 -cos(angle)*d + (p0.y+center.y)/2 - 5
            let x = angle.sin() * d + (p0.0 + center.0) / 2.0 - 5.0;
            let y = -angle.cos() * d + (p0.1 + center.1) / 2.0 - 5.0;
            (x, y)
        }
    };

    (x, y)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn fmt(v: f64) -> String {
    let s = format!("{:.7}", v);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

fn esc(s: &str) -> String {
    // First convert HTML named entities to Unicode, then XML-escape for SVG
    let s = crate::svg::html_entities_to_unicode(s);
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    const CLASS_BASIC: &str = "classDiagram\n    class Animal {\n        +String name\n        +int age\n        +makeSound() void\n    }\n    class Dog {\n        +String breed\n        +fetch() void\n    }\n    Animal <|-- Dog";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(CLASS_BASIC).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains("<svg"), "missing <svg tag");
        assert!(svg.contains("Animal"), "missing class name");
        assert!(svg.contains("Dog"), "missing class name");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(CLASS_BASIC).diagram;
        let svg = render(&diag, Theme::Dark, false);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn snapshot_default_theme() {
        let diag = parser::parse(CLASS_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(svg);
    }
}
