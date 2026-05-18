// Faithful Rust port of Mermaid's requirementRenderer.ts.
// Uses dagre for layout (TB direction), like erRenderer and classRenderer.
// Layout constants tuned to match Mermaid reference output.

use super::constants::*;
use super::parser::{Element, Requirement, RequirementDiagram};
use super::templates;
use crate::text_browser_metrics::measure_browser;
use crate::theme::Theme;
use dagre_dgl_rs::graph::{EdgeLabel, Graph, GraphLabel, NodeLabel};
use dagre_dgl_rs::layout::layout;

struct NodeGeom {
    id: String,
    width: f64,
    height: f64,
}

fn tmw(s: &str) -> f64 {
    let (w, _) = measure_browser(s, FONT_SIZE);
    // Add correct space advance width (measure_browser stores 0 for space).
    // +6px safety margin prevents last letter clipping in browser rendering.
    let n_spaces = s.chars().filter(|&c| c == ' ').count() as f64;
    w + n_spaces * SPACE_W_16 + TEXT_SAFETY_MARGIN
}

fn req_geom(req: &Requirement) -> NodeGeom {
    let max_w = [
        tmw(&format!("<<{}>>", req.req_type.display())),
        tmw(&req.name),
        tmw(&format!("ID: {}", req.id)),
        tmw(&format!("Text: {}", req.text)),
        tmw(&format!("Risk: {}", req.risk.display())),
        tmw(&format!("Verification: {}", req.verify_method.display())),
    ]
    .iter()
    .cloned()
    .fold(0.0_f64, f64::max);
    let n_body = 4usize; // ID, Text, Risk, Verification
    NodeGeom {
        id: req.name.clone(),
        width: (max_w + PAD_X * 2.0).max(MIN_WIDTH),
        height: HEADER_H + (n_body as f64 + 0.5) * ROW_H + PAD_Y,
    }
}

fn elem_geom(elem: &Element) -> NodeGeom {
    let mut tw = vec![tmw("<<Element>>"), tmw(&elem.name)];
    if !elem.elem_type.is_empty() {
        tw.push(tmw(&format!("Type: {}", elem.elem_type)));
    }
    if !elem.doc_ref.is_empty() {
        tw.push(tmw(&format!("DocRef: {}", elem.doc_ref)));
    }
    let max_w = tw.iter().cloned().fold(0.0_f64, f64::max);
    let body_rows = tw.len().saturating_sub(2);
    NodeGeom {
        id: elem.name.clone(),
        width: (max_w + PAD_X * 2.0).max(MIN_WIDTH),
        height: HEADER_H + (body_rows as f64 + 0.5) * ROW_H + PAD_Y,
    }
}

fn xe(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_req(req: &Requirement, geom: &NodeGeom, cx: f64, cy: f64) -> String {
    let (w, h) = (geom.width, geom.height);
    let (hw, hh) = (w / 2.0, h / 2.0);
    // Divider separates header (type + name) from body (data items)
    let sep_y = -hh + HEADER_H;
    let mut o = templates::node_group_open(cx, cy);
    // Fill path (background) with border stroke
    o += &templates::node_box_path(-hw, -hh, hw, hh, BOX_STROKE, BOX_FILL);
    // Divider line
    o += &templates::node_divider(-hw, hw, sep_y, BOX_STROKE);
    // Header: <<type>> using foreignObject (resvg-invisible, matches reference)
    let type_str = format!("&lt;&lt;{}&gt;&gt;", req.req_type.display());
    let type_w = tmw(&format!("<<{}>>", req.req_type.display()));
    let name_w = tmw(&req.name);
    let type_lx = -(type_w / 2.0);
    let name_lx = -(name_w / 2.0);
    let type_ly = -hh + PAD_Y + ROW_H / 2.0 - 12.0; // top of foreignObject (center-12)
    let name_ly = -hh + PAD_Y + ROW_H * 1.5 - 12.0;
    o += &templates::label_fo(type_lx, type_ly, type_w, &type_str);
    o += &templates::label_fo_bold(name_lx, name_ly, name_w, &xe(&req.name));
    // Body items using foreignObject
    let items = [
        format!("ID: {}", req.id),
        format!("Text: {}", req.text),
        format!("Risk: {}", req.risk.display()),
        format!("Verification: {}", req.verify_method.display()),
    ];
    let mut ry = sep_y + PAD_Y + ROW_H / 2.0 - 12.0; // top of foreignObject
    for item in &items {
        let iw = tmw(item);
        let ix = -hw + PAD_X;
        o += &templates::label_fo_body(ix, ry, iw, &xe(item));
        ry += ROW_H;
    }
    o + "</g>"
}

fn render_elem(elem: &Element, geom: &NodeGeom, cx: f64, cy: f64) -> String {
    let (w, h) = (geom.width, geom.height);
    let (hw, hh) = (w / 2.0, h / 2.0);
    let sep_y = -hh + HEADER_H;
    let mut o = templates::node_group_open(cx, cy);
    // Fill path (background) with border stroke
    o += &templates::node_box_path(-hw, -hh, hw, hh, ELEM_STROKE, ELEM_FILL);
    // Divider line
    o += &templates::node_divider(-hw, hw, sep_y, ELEM_STROKE);
    // Header: <<Element>> and name using foreignObject
    let elem_w = tmw("<<Element>>");
    let name_w = tmw(&elem.name);
    let type_lx = -(elem_w / 2.0);
    let name_lx = -(name_w / 2.0);
    let type_ly = -hh + PAD_Y + ROW_H / 2.0 - 12.0;
    let name_ly = -hh + PAD_Y + ROW_H * 1.5 - 12.0;
    o += &templates::label_fo(type_lx, type_ly, elem_w, "&lt;&lt;Element&gt;&gt;");
    o += &templates::label_fo_bold(name_lx, name_ly, name_w, &xe(&elem.name));
    // Body items using foreignObject
    let mut body: Vec<String> = vec![];
    if !elem.elem_type.is_empty() {
        body.push(format!("Type: {}", elem.elem_type));
    }
    if !elem.doc_ref.is_empty() {
        body.push(format!("DocRef: {}", elem.doc_ref));
    }
    let mut ry = sep_y + PAD_Y + ROW_H / 2.0 - 12.0;
    for item in &body {
        let iw = tmw(item);
        let ix = -hw + PAD_X;
        o += &templates::label_fo_body(ix, ry, iw, &xe(item));
        ry += ROW_H;
    }
    o + "</g>"
}

fn pts_path(pts: &[(f64, f64)]) -> String {
    if pts.is_empty() {
        return String::new();
    }
    let mut d = format!("M{:.1},{:.1}", pts[0].0, pts[0].1);
    for p in &pts[1..] {
        d += &format!("L{:.1},{:.1}", p.0, p.1);
    }
    d
}

fn midpt(pts: &[(f64, f64)]) -> (f64, f64) {
    if pts.len() <= 1 {
        return pts.first().copied().unwrap_or_default();
    }
    // Compute true geometric midpoint of the polyline
    let total_len: f64 = pts
        .windows(2)
        .map(|s| {
            let dx = s[1].0 - s[0].0;
            let dy = s[1].1 - s[0].1;
            (dx * dx + dy * dy).sqrt()
        })
        .sum();
    let half = total_len / 2.0;
    let mut acc = 0.0_f64;
    for s in pts.windows(2) {
        let dx = s[1].0 - s[0].0;
        let dy = s[1].1 - s[0].1;
        let seg_len = (dx * dx + dy * dy).sqrt();
        if acc + seg_len >= half {
            let t = (half - acc) / seg_len;
            return (s[0].0 + t * dx, s[0].1 + t * dy);
        }
        acc += seg_len;
    }
    *pts.last().unwrap()
}

fn fallback_pts(g: &Graph, v: &str, w: &str) -> Vec<(f64, f64)> {
    match (g.node_opt(v), g.node_opt(w)) {
        (Some(a), Some(b)) => match (a.x, a.y, b.x, b.y) {
            (Some(ax), Some(ay), Some(bx), Some(by)) => vec![(ax, ay), (bx, by)],
            _ => vec![],
        },
        _ => vec![],
    }
}

fn render_relation(rel: &super::parser::Relation, pts: &[(f64, f64)], sid: &str) -> String {
    if pts.len() < 2 {
        return String::new();
    }
    let d = pts_path(pts);
    let dash = if rel.rel_type.is_contains() {
        "0"
    } else {
        "10,7"
    };
    let marker_end = templates::marker_end_attr(sid);
    let marker_start = if rel.rel_type.is_contains() {
        templates::marker_start_attr(sid)
    } else {
        String::new()
    };
    let path = templates::relation_path(&d, REL_COLOR, dash, &marker_start, &marker_end);
    let lhtml = format!("&lt;&lt;{}&gt;&gt;", rel.rel_type.display());
    let (mx, my) = midpt(pts);
    // Edge label using foreignObject (like reference). Mermaid uses 16px text width.
    let (lw, _) = measure_browser(&format!("<<{}>>", rel.rel_type.display()), FONT_SIZE);
    let lbl_inner_x = -(lw / 2.0);
    let lbl = templates::edge_label_fo(mx, my, lbl_inner_x, lw, &lhtml);
    format!("{path}{lbl}")
}

fn markers(sid: &str) -> String {
    let mut o = templates::marker_arrow_end(sid, REL_COLOR);
    o += &templates::marker_contains_start(sid, REL_COLOR);
    o
}

fn css(sid: &str) -> String {
    format!(
        "#{sid}{{font-family:Arial,sans-serif;font-size:{fs}px;fill:{FONT_COLOR};}}\
        #{sid} p{{margin:0;}}\
        #{sid} .node rect{{stroke-width:1.3;}}\
        #{sid} .relationshipLine{{fill:none;stroke-width:1.5;}}\
        #{sid} .labelBkg{{background-color:rgba(232,232,232,0.8);text-align:center;}}\
        #{sid} .edgeLabel{{background-color:rgba(232,232,232,0.8);text-align:center;}}\
        #{sid} .edgeLabel p{{background-color:rgba(232,232,232,0.8);}}",
        fs = FONT_SIZE as i64
    )
}

pub fn render(diag: &RequirementDiagram, theme: Theme) -> String {
    let _vars = theme.resolve();
    let sid = "mermaid-req-svg";
    // Guard: empty diagram
    if diag.requirements.is_empty() && diag.elements.is_empty() {
        return templates::empty_svg(sid);
    }
    let rg: Vec<NodeGeom> = diag.requirements.iter().map(req_geom).collect();
    let eg: Vec<NodeGeom> = diag.elements.iter().map(elem_geom).collect();

    let mut g = Graph::with_options(false, true, false);
    g.set_graph(GraphLabel {
        rankdir: Some("TB".to_string()),
        nodesep: Some(NODE_SEP),
        ranksep: Some(RANK_SEP),
        marginx: Some(MARGIN_X),
        marginy: Some(MARGIN_Y),
        ..Default::default()
    });
    for geom in &rg {
        g.set_node(
            &geom.id,
            NodeLabel {
                width: geom.width,
                height: geom.height,
                ..Default::default()
            },
        );
    }
    for geom in &eg {
        g.set_node(
            &geom.id,
            NodeLabel {
                width: geom.width,
                height: geom.height,
                ..Default::default()
            },
        );
    }
    let label_fs = FONT_SIZE - 4.0; // 12px edge label font size
    for (i, rel) in diag.relations.iter().enumerate() {
        let (lw, _) = measure_browser(&format!("<<{}>>", rel.rel_type.display()), label_fs);
        g.set_edge(
            &rel.src,
            &rel.dst,
            EdgeLabel {
                minlen: Some(1),
                weight: Some(1.0),
                width: Some(lw + 8.0),
                height: Some(18.0),
                labelpos: Some("c".to_string()),
                labeloffset: Some(10.0),
                ..Default::default()
            },
            Some(&format!("rel{i}")),
        );
    }
    layout(&mut g);

    let (gw, gh) = (
        g.graph().width.unwrap_or(600.0),
        g.graph().height.unwrap_or(400.0),
    );
    let mut svg = templates::svg_root(sid, gw, gh, &css(sid));
    svg += &markers(sid);
    svg += "<g class=\"req-root\"><g class=\"req-relationships\">";
    for (i, rel) in diag.relations.iter().enumerate() {
        let ename = format!("rel{i}");
        let pts: Vec<(f64, f64)> = {
            let lab = g.edge_label_named(&rel.src, &rel.dst, &ename);
            if let Some(l) = lab {
                l.points
                    .as_ref()
                    .map(|p| p.iter().map(|q| (q.x, q.y)).collect())
                    .unwrap_or_else(|| fallback_pts(&g, &rel.src, &rel.dst))
            } else {
                g.edge_vw(&rel.src, &rel.dst)
                    .and_then(|l| {
                        l.points
                            .as_ref()
                            .map(|p| p.iter().map(|q| (q.x, q.y)).collect())
                    })
                    .unwrap_or_else(|| fallback_pts(&g, &rel.src, &rel.dst))
            }
        };
        svg += &render_relation(rel, &pts, sid);
    }
    svg += "</g><g class=\"req-nodes\">";
    for (i, req) in diag.requirements.iter().enumerate() {
        let geom = &rg[i];
        let (cx, cy) = g
            .node_opt(&geom.id)
            .and_then(|n| n.x.zip(n.y))
            .unwrap_or((0.0, 0.0));
        svg += &render_req(req, geom, cx, cy);
    }
    for (i, elem) in diag.elements.iter().enumerate() {
        let geom = &eg[i];
        let (cx, cy) = g
            .node_opt(&geom.id)
            .and_then(|n| n.x.zip(n.y))
            .unwrap_or((0.0, 0.0));
        svg += &render_elem(elem, geom, cx, cy);
    }
    svg + "</g></g></svg>"
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    #[test]
    fn basic_render() {
        let input = "requirementDiagram\n    requirement test_req {\n        id: 1\n        text: the test text.\n        risk: high\n        verifymethod: test\n    }\n    element test_entity {\n        type: simulation\n    }\n    test_entity - satisfies -> test_req";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        assert!(svg.contains("<svg"), "no svg");
        assert!(svg.contains("test_req"), "no req");
        assert!(svg.contains("test_entity"), "no elem");
        assert!(svg.contains("satisfies"), "no rel");
    }

    #[test]
    fn empty_renders() {
        let svg = render(
            &parser::parse("requirementDiagram").diagram,
            crate::theme::Theme::Default,
        );
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "requirementDiagram\n    requirement test_req {\n    id: 1\n    text: the test text.\n    risk: high\n    verifymethod: test\n    }\n    element test_entity {\n    type: simulation\n    }\n    test_entity - satisfies -> test_req";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(svg);
    }
}
