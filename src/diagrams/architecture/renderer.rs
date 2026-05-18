// Faithful Rust port of mermaid/src/diagrams/architecture/architectureRenderer.ts
//
// Matches mermaid's default output faithfully:
//   - Services rendered with actual SVG icon shapes (blue 80×80 boxes, white art)
//   - Groups rendered as dashed rounded-rect containers with optional icon
//   - Edges as 3-point paths with polygon arrowheads
//   - Coordinate system centered on content (viewBox with negative offsets)
//
// Output structure mirrors architectureRenderer.ts SVG output:
//   <g></g>
//   <g class="architecture-edges">
//   <g class="architecture-services">
//   <g class="architecture-groups">

use super::constants::*;
use super::parser::{ArchDiagram, ArchEdge, ArchGroup, ArchJunction, ArchService, Direction};
#[allow(unused_imports)]
use super::templates;
use crate::theme::Theme;
use std::collections::HashMap;

// ─── Position ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct Pos {
    x: f64,
    y: f64,
}

// ─── Position-based layout engine ────────────────────────────────────────────
//
// Stores absolute SVG centre coordinates rather than grid indices.
// This allows per-edge step sizes (cross-group edges use H_STEP + H_STEP_CROSS_EXTRA).

struct Layout {
    /// node_id -> SVG centre (x, y)
    pos: HashMap<String, Pos>,
}

impl Layout {
    fn new(services: &[ArchService], junctions: &[ArchJunction], edges: &[ArchEdge]) -> Self {
        let mut ids: Vec<String> = services.iter().map(|s| s.id.clone()).collect();
        ids.extend(junctions.iter().map(|j| j.id.clone()));

        // Build a map of node_id → group_id for cross-group detection.
        let node_group: HashMap<&str, Option<&str>> = services
            .iter()
            .map(|s| (s.id.as_str(), s.in_group.as_deref()))
            .chain(
                junctions
                    .iter()
                    .map(|j| (j.id.as_str(), j.in_group.as_deref())),
            )
            .collect();

        let mut pos: HashMap<String, Pos> = HashMap::new();

        // Seed the first node at the origin.
        if let Some(first) = ids.first() {
            pos.insert(first.clone(), Pos { x: 0.0, y: 0.0 });
        }

        // BFS: propagate positions along edge direction constraints.
        let mut changed = true;
        let mut iterations = 0;
        while changed && iterations < 200 {
            changed = false;
            iterations += 1;
            for edge in edges {
                let lhs_placed = pos.contains_key(&edge.lhs_id);
                let rhs_placed = pos.contains_key(&edge.rhs_id);

                if !lhs_placed && !rhs_placed {
                    if pos.is_empty() {
                        pos.insert(edge.lhs_id.clone(), Pos { x: 0.0, y: 0.0 });
                        changed = true;
                    }
                    continue;
                }

                // Determine if this edge crosses a group boundary.
                let lhs_grp = node_group.get(edge.lhs_id.as_str()).copied().flatten();
                let rhs_grp = node_group.get(edge.rhs_id.as_str()).copied().flatten();
                let cross_group = lhs_grp != rhs_grp;

                // Effective step distances for this edge.
                let h_step = if cross_group {
                    H_STEP + H_STEP_CROSS_EXTRA
                } else {
                    H_STEP
                };
                let v_step = V_STEP; // vertical step doesn't change with group boundary in reference

                if lhs_placed && !rhs_placed {
                    let lp = pos[&edge.lhs_id].clone();
                    let rp = offset_dir(&lp, &edge.lhs_dir, h_step, v_step);
                    pos.insert(edge.rhs_id.clone(), rp);
                    changed = true;
                } else if rhs_placed && !lhs_placed {
                    let rp = pos[&edge.rhs_id].clone();
                    let lp = offset_dir(&rp, &edge.rhs_dir, h_step, v_step);
                    pos.insert(edge.lhs_id.clone(), lp);
                    changed = true;
                }
            }
        }

        // Place any remaining unpositioned nodes below the rest.
        let max_y = pos.values().map(|p| p.y).fold(f64::MIN, f64::max);
        let start_y = if max_y == f64::MIN {
            0.0
        } else {
            max_y + V_STEP * 2.0
        };
        let mut x_off = 0.0;
        for id in &ids {
            if !pos.contains_key(id) {
                pos.insert(
                    id.clone(),
                    Pos {
                        x: x_off,
                        y: start_y,
                    },
                );
                x_off += H_STEP;
            }
        }

        Layout { pos }
    }

    fn node_centre(&self, id: &str) -> Option<Pos> {
        self.pos.get(id).cloned()
    }
}

/// Offset a position by one step in the given direction.
fn offset_dir(p: &Pos, dir: &Direction, h_step: f64, v_step: f64) -> Pos {
    match dir {
        Direction::L => Pos {
            x: p.x - h_step,
            y: p.y,
        },
        Direction::R => Pos {
            x: p.x + h_step,
            y: p.y,
        },
        Direction::T => Pos {
            x: p.x,
            y: p.y - v_step,
        },
        Direction::B => Pos {
            x: p.x,
            y: p.y + v_step,
        },
    }
}

/// The anchor point on a node's icon boundary in the given direction.
fn anchor(centre: &Pos, dir: &Direction) -> (f64, f64) {
    let h = ICON_SIZE / 2.0;
    match dir {
        Direction::L => (centre.x - h, centre.y),
        Direction::R => (centre.x + h, centre.y),
        Direction::T => (centre.x, centre.y - h),
        Direction::B => (centre.x, centre.y + h),
    }
}

// ─── Bounding box ─────────────────────────────────────────────────────────────

struct BBox {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// Bounding box for all content of a group (children icons + labels),
/// expanded by GROUP_PAD on all sides.
fn group_bbox(
    group_id: &str,
    layout: &Layout,
    services: &[ArchService],
    junctions: &[ArchJunction],
    groups: &[ArchGroup],
) -> Option<BBox> {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut any = false;

    for svc in services {
        if svc.in_group.as_deref() == Some(group_id) {
            if let Some(c) = layout.node_centre(&svc.id) {
                min_x = min_x.min(c.x - ICON_SIZE / 2.0);
                min_y = min_y.min(c.y - ICON_SIZE / 2.0);
                max_x = max_x.max(c.x + ICON_SIZE / 2.0);
                max_y = max_y.max(c.y + ICON_SIZE / 2.0 + LABEL_SPACE_IN_GROUP);
                any = true;
            }
        }
    }
    for jct in junctions {
        if jct.in_group.as_deref() == Some(group_id) {
            if let Some(c) = layout.node_centre(&jct.id) {
                min_x = min_x.min(c.x - 8.0);
                min_y = min_y.min(c.y - 8.0);
                max_x = max_x.max(c.x + 8.0);
                max_y = max_y.max(c.y + 8.0);
                any = true;
            }
        }
    }
    for child in groups {
        if child.in_group.as_deref() == Some(group_id) {
            if let Some(cb) = group_bbox(&child.id, layout, services, junctions, groups) {
                min_x = min_x.min(cb.x);
                min_y = min_y.min(cb.y);
                max_x = max_x.max(cb.x + cb.w);
                max_y = max_y.max(cb.y + cb.h);
                any = true;
            }
        }
    }

    if !any {
        return None;
    }

    Some(BBox {
        x: min_x - GROUP_PAD,
        y: min_y - GROUP_PAD,
        w: (max_x - min_x) + GROUP_PAD * 2.0,
        h: (max_y - min_y) + GROUP_PAD * 2.0,
    })
}

// ─── SVG icon library ─────────────────────────────────────────────────────────

/// Returns the SVG content (inside the 80×80 viewBox) for a named icon.
/// Blue (#087ebf) background with white line art.
fn icon_inner(name: &str) -> &'static str {
    match name.to_lowercase().as_str() {
        "internet" => concat!(
            "<rect width=\"80\" height=\"80\" style=\"fill: #087ebf; stroke-width: 0px;\"></rect>",
            "<circle cx=\"40\" cy=\"40\" r=\"22.5\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></circle>",
            "<line x1=\"40\" y1=\"17.5\" x2=\"40\" y2=\"62.5\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></line>",
            "<line x1=\"17.5\" y1=\"40\" x2=\"62.5\" y2=\"40\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></line>",
            "<path d=\"m39.99,17.51c-15.28,11.1-15.28,33.88,0,44.98\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></path>",
            "<path d=\"m40.01,17.51c15.28,11.1,15.28,33.88,0,44.98\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></path>",
            "<line x1=\"19.75\" y1=\"30.1\" x2=\"60.25\" y2=\"30.1\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></line>",
            "<line x1=\"19.75\" y1=\"49.9\" x2=\"60.25\" y2=\"49.9\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></line>",
        ),
        "database" => concat!(
            "<rect width=\"80\" height=\"80\" style=\"fill: #087ebf; stroke-width: 0px;\"></rect>",
            "<path id=\"IconifyId19e371fabfe869c2d0\" data-name=\"4\" d=\"m20,57.86c0,3.94,8.95,7.14,20,7.14s20-3.2,20-7.14\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></path>",
            "<path id=\"IconifyId19e371fabfe869c2d1\" data-name=\"3\" d=\"m20,45.95c0,3.94,8.95,7.14,20,7.14s20-3.2,20-7.14\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></path>",
            "<path id=\"IconifyId19e371fabfe869c2d2\" data-name=\"2\" d=\"m20,34.05c0,3.94,8.95,7.14,20,7.14s20-3.2,20-7.14\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></path>",
            "<ellipse id=\"IconifyId19e371fabfe869c2d3\" data-name=\"1\" cx=\"40\" cy=\"22.14\" rx=\"20\" ry=\"7.14\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></ellipse>",
            "<line x1=\"20\" y1=\"57.86\" x2=\"20\" y2=\"22.14\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></line>",
            "<line x1=\"60\" y1=\"57.86\" x2=\"60\" y2=\"22.14\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></line>",
        ),
        "server" => concat!(
            "<rect width=\"80\" height=\"80\" style=\"fill: #087ebf; stroke-width: 0px;\"></rect>",
            "<rect x=\"17.5\" y=\"17.5\" width=\"45\" height=\"45\" rx=\"2\" ry=\"2\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></rect>",
            "<line x1=\"17.5\" y1=\"32.5\" x2=\"62.5\" y2=\"32.5\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></line>",
            "<line x1=\"17.5\" y1=\"47.5\" x2=\"62.5\" y2=\"47.5\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></line>",
            "<g><path d=\"m56.25,25c0,.27-.45.5-1,.5h-10.5c-.55,0-1-.23-1-.5s.45-.5,1-.5h10.5c.55,0,1,.23,1,.5Z\" style=\"fill: #fff; stroke-width: 0px;\"></path><path d=\"m56.25,25c0,.27-.45.5-1,.5h-10.5c-.55,0-1-.23-1-.5s.45-.5,1-.5h10.5c.55,0,1,.23,1,.5Z\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10;\"></path></g>",
            "<g><path d=\"m56.25,40c0,.27-.45.5-1,.5h-10.5c-.55,0-1-.23-1-.5s.45-.5,1-.5h10.5c.55,0,1,.23,1,.5Z\" style=\"fill: #fff; stroke-width: 0px;\"></path><path d=\"m56.25,40c0,.27-.45.5-1,.5h-10.5c-.55,0-1-.23-1-.5s.45-.5,1-.5h10.5c.55,0,1,.23,1,.5Z\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10;\"></path></g>",
            "<g><path d=\"m56.25,55c0,.27-.45.5-1,.5h-10.5c-.55,0-1-.23-1-.5s.45-.5,1-.5h10.5c.55,0,1,.23,1,.5Z\" style=\"fill: #fff; stroke-width: 0px;\"></path><path d=\"m56.25,55c0,.27-.45.5-1,.5h-10.5c-.55,0-1-.23-1-.5s.45-.5,1-.5h10.5c.55,0,1,.23,1,.5Z\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10;\"></path></g>",
            "<g><circle cx=\"32.5\" cy=\"25\" r=\".75\" style=\"fill: #fff; stroke: #fff; stroke-miterlimit: 10;\"></circle><circle cx=\"27.5\" cy=\"25\" r=\".75\" style=\"fill: #fff; stroke: #fff; stroke-miterlimit: 10;\"></circle><circle cx=\"22.5\" cy=\"25\" r=\".75\" style=\"fill: #fff; stroke: #fff; stroke-miterlimit: 10;\"></circle></g>",
            "<g><circle cx=\"32.5\" cy=\"40\" r=\".75\" style=\"fill: #fff; stroke: #fff; stroke-miterlimit: 10;\"></circle><circle cx=\"27.5\" cy=\"40\" r=\".75\" style=\"fill: #fff; stroke: #fff; stroke-miterlimit: 10;\"></circle><circle cx=\"22.5\" cy=\"40\" r=\".75\" style=\"fill: #fff; stroke: #fff; stroke-miterlimit: 10;\"></circle></g>",
            "<g><circle cx=\"32.5\" cy=\"55\" r=\".75\" style=\"fill: #fff; stroke: #fff; stroke-miterlimit: 10;\"></circle><circle cx=\"27.5\" cy=\"55\" r=\".75\" style=\"fill: #fff; stroke: #fff; stroke-miterlimit: 10;\"></circle><circle cx=\"22.5\" cy=\"55\" r=\".75\" style=\"fill: #fff; stroke: #fff; stroke-miterlimit: 10;\"></circle></g>",
        ),
        "disk" => concat!(
            "<rect width=\"80\" height=\"80\" style=\"fill: #087ebf; stroke-width: 0px;\"></rect>",
            "<rect x=\"20\" y=\"15\" width=\"40\" height=\"50\" rx=\"1\" ry=\"1\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></rect>",
            "<ellipse cx=\"24\" cy=\"19.17\" rx=\".8\" ry=\".83\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></ellipse>",
            "<ellipse cx=\"56\" cy=\"19.17\" rx=\".8\" ry=\".83\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></ellipse>",
            "<ellipse cx=\"24\" cy=\"60.83\" rx=\".8\" ry=\".83\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></ellipse>",
            "<ellipse cx=\"56\" cy=\"60.83\" rx=\".8\" ry=\".83\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></ellipse>",
            "<ellipse cx=\"40\" cy=\"33.75\" rx=\"14\" ry=\"14.58\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></ellipse>",
            "<ellipse cx=\"40\" cy=\"33.75\" rx=\"4\" ry=\"4.17\" style=\"fill: #fff; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></ellipse>",
            "<path d=\"m37.51,42.52l-4.83,13.22c-.26.71-1.1,1.02-1.76.64l-4.18-2.42c-.66-.38-.81-1.26-.33-1.84l9.01-10.8c.88-1.05,2.56-.08,2.09,1.2Z\" style=\"fill: #fff; stroke-width: 0px;\"></path>",
        ),
        "cloud" => concat!(
            "<rect width=\"80\" height=\"80\" style=\"fill: #087ebf; stroke-width: 0px;\"></rect>",
            "<path d=\"m65,47.5c0,2.76-2.24,5-5,5H20c-2.76,0-5-2.24-5-5,0-1.87,1.03-3.51,2.56-4.36-.04-.21-.06-.42-.06-.64,0-2.6,2.48-4.74,5.65-4.97,1.65-4.51,6.34-7.76,11.85-7.76.86,0,1.69.08,2.5.23,2.09-1.57,4.69-2.5,7.5-2.5,6.1,0,11.19,4.38,12.28,10.17,2.14.56,3.72,2.51,3.72,4.83,0,.03,0,.07-.01.1,2.29.46,4.01,2.48,4.01,4.9Z\" style=\"fill: none; stroke: #fff; stroke-miterlimit: 10; stroke-width: 2px;\"></path>",
        ),
        _ => "<rect width=\"80\" height=\"80\" style=\"fill: #087ebf; stroke-width: 0px;\"></rect>",
    }
}

// ─── Main render entry ─────────────────────────────────────────────────────────

pub fn render(diag: &ArchDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let layout = Layout::new(&diag.services, &diag.junctions, &diag.edges);

    // Compute extents of all node centres
    let node_ids: Vec<&str> = diag
        .services
        .iter()
        .map(|s| s.id.as_str())
        .chain(diag.junctions.iter().map(|j| j.id.as_str()))
        .collect();

    // If no nodes, output a minimal SVG
    if node_ids.is_empty() {
        return r#"<svg id="mermaid-svg" width="100%" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" role="graphics-document document" aria-roledescription="architecture"><g></g><g class="architecture-edges"></g><g class="architecture-services"></g><g class="architecture-groups"></g></svg>"#.to_string();
    }

    // Node icon bounding boxes (without labels)
    let mut node_min_x = f64::MAX;
    let mut node_min_y = f64::MAX;
    let mut node_max_x = f64::MIN;
    let mut node_max_y = f64::MIN;
    for id in &node_ids {
        if let Some(c) = layout.node_centre(id) {
            node_min_x = node_min_x.min(c.x - ICON_SIZE / 2.0);
            node_min_y = node_min_y.min(c.y - ICON_SIZE / 2.0);
            node_max_x = node_max_x.max(c.x + ICON_SIZE / 2.0);
            node_max_y = node_max_y.max(c.y + ICON_SIZE / 2.0 + LABEL_SPACE);
        }
    }

    // Group bounding boxes may extend beyond node bounds
    let mut grp_min_x = node_min_x;
    let mut grp_min_y = node_min_y;
    let mut grp_max_x = node_max_x;
    let mut grp_max_y = node_max_y;
    for grp in &diag.groups {
        if grp.in_group.is_none() {
            if let Some(bb) = group_bbox(
                &grp.id,
                &layout,
                &diag.services,
                &diag.junctions,
                &diag.groups,
            ) {
                grp_min_x = grp_min_x.min(bb.x);
                grp_min_y = grp_min_y.min(bb.y);
                grp_max_x = grp_max_x.max(bb.x + bb.w);
                grp_max_y = grp_max_y.max(bb.y + bb.h);
            }
        }
    }

    // viewBox = content bbox expanded by OUTER_MARGIN on each side
    let vb_x = grp_min_x - OUTER_MARGIN;
    let vb_y = grp_min_y - OUTER_MARGIN;
    let vb_w = (grp_max_x - grp_min_x) + OUTER_MARGIN * 2.0;
    let vb_h = (grp_max_y - grp_min_y) + OUTER_MARGIN * 2.0;

    let mut out = String::new();

    // SVG header
    out.push_str(&format!(
        "<svg id=\"mermaid-svg\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" style=\"max-width: {}px;\" viewBox=\"{} {} {} {}\" role=\"graphics-document document\" aria-roledescription=\"architecture\">",
        fmt(vb_w), fmt(vb_x), fmt(vb_y), fmt(vb_w), fmt(vb_h),
    ));

    // CSS (matches mermaid's architectureRenderer CSS)
    out.push_str(&format!(
        concat!(
            "<style>",
            "#mermaid-svg{{font-family:{ff};font-size:16px;fill:#333;}}",
            "#mermaid-svg .edge{{stroke-width:3;stroke:#333333;fill:none;}}",
            "#mermaid-svg .arrow{{fill:#333333;}}",
            "#mermaid-svg .node-bkg{{fill:none;stroke:hsl(240, 60%, 86.2745098039%);stroke-width:2px;stroke-dasharray:8;}}",
            "#mermaid-svg .node-icon-text{{display:flex;align-items:center;}}",
            "#mermaid-svg p{{margin:0;}}",
            "</style>",
        ),
        ff = ff,
    ));

    // mermaid outputs an empty <g> first
    out.push_str("<g></g>");

    // Edges layer (BEFORE services in mermaid's output)
    out.push_str("<g class=\"architecture-edges\">");
    for edge in &diag.edges {
        render_edge(edge, &layout, &mut out);
    }
    out.push_str("</g>");

    // Services layer
    out.push_str("<g class=\"architecture-services\">");
    for svc in &diag.services {
        render_service(svc, &layout, &mut out);
    }
    out.push_str("</g>");

    // Groups layer (AFTER services in mermaid's output)
    out.push_str("<g class=\"architecture-groups\">");
    for grp in &diag.groups {
        if grp.in_group.is_none() {
            render_group_recursive(
                grp,
                &layout,
                &diag.services,
                &diag.junctions,
                &diag.groups,
                &mut out,
            );
        }
    }
    out.push_str("</g>");

    out.push_str("</svg>");
    out
}

// ─── Group rendering ───────────────────────────────────────────────────────────

fn render_group_recursive(
    grp: &ArchGroup,
    layout: &Layout,
    services: &[ArchService],
    junctions: &[ArchJunction],
    groups: &[ArchGroup],
    out: &mut String,
) {
    let bb = match group_bbox(&grp.id, layout, services, junctions, groups) {
        Some(b) => b,
        None => return,
    };

    // Dashed container rectangle
    out.push_str(&format!(
        "<rect id=\"mermaid-svg-group-{id}\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" class=\"node-bkg\"></rect>",
        id = esc(&grp.id),
        x = fmt(bb.x), y = fmt(bb.y), w = fmt(bb.w), h = fmt(bb.h),
    ));

    // Optional icon (30×30 at top-left) + label text to its right
    if let Some(icon) = &grp.icon {
        let icon_x = bb.x + 1.0;
        let icon_y = bb.y + 1.0;
        out.push_str(&format!(
            "<g><g transform=\"translate({ix}, {iy})\"><g><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"30\" height=\"30\" viewBox=\"0 0 80 80\"><g>{inner}</g></svg></g></g>",
            ix = fmt(icon_x), iy = fmt(icon_y),
            inner = icon_inner(icon),
        ));

        // Label text next to icon, using same tspan structure as services
        let label = grp.title.as_deref().unwrap_or(&grp.id);
        let label_tx = icon_x + 33.0; // icon 30px + 3px gap
        let label_ty = icon_y + 14.0; // vertically centred in 30px icon
        out.push_str(&format!(
            "<g dy=\"1em\" alignment-baseline=\"middle\" dominant-baseline=\"start\" text-anchor=\"start\" transform=\"translate({tx}, {ty})\"><g><rect class=\"background\" style=\"stroke: none\"></rect><text y=\"-10.1\" style=\"\"><tspan class=\"text-outer-tspan row\" x=\"0\" y=\"-0.1em\" dy=\"1.1em\"><tspan font-style=\"normal\" class=\"text-inner-tspan\" font-weight=\"normal\">{text}</tspan></tspan></text></g></g>",
            tx = fmt(label_tx), ty = fmt(label_ty),
            text = esc(label),
        ));
        out.push_str("</g>");
    } else {
        // No icon: plain label
        let label = grp.title.as_deref().unwrap_or(&grp.id);
        out.push_str(&format!(
            "<g dy=\"1em\" alignment-baseline=\"middle\" dominant-baseline=\"start\" text-anchor=\"start\" transform=\"translate({tx}, {ty})\"><g><rect class=\"background\" style=\"stroke: none\"></rect><text y=\"-10.1\" style=\"\"><tspan class=\"text-outer-tspan row\" x=\"0\" y=\"-0.1em\" dy=\"1.1em\"><tspan font-style=\"normal\" class=\"text-inner-tspan\" font-weight=\"normal\">{text}</tspan></tspan></text></g></g>",
            tx = fmt(bb.x + 10.0), ty = fmt(bb.y + 18.0),
            text = esc(label),
        ));
    }

    // Recurse into child groups
    for child in groups {
        if child.in_group.as_deref() == Some(&grp.id) {
            render_group_recursive(child, layout, services, junctions, groups, out);
        }
    }
}

// ─── Service rendering ─────────────────────────────────────────────────────────

fn render_service(svc: &ArchService, layout: &Layout, out: &mut String) {
    let centre = match layout.node_centre(&svc.id) {
        Some(c) => c,
        None => return,
    };

    let icon_left = centre.x - ICON_SIZE / 2.0;
    let icon_top = centre.y - ICON_SIZE / 2.0;
    let label = svc.title.as_deref().unwrap_or(&svc.id);

    out.push_str(&format!(
        "<g id=\"mermaid-svg-service-{id}\" class=\"architecture-service\" transform=\"translate({tx},{ty})\">",
        id = esc(&svc.id),
        tx = fmt(icon_left),
        ty = fmt(icon_top),
    ));

    // Label group below icon: translate(icon_half, icon_size) = translate(40, 80) in service-local
    out.push_str(&format!(
        "<g dy=\"1em\" alignment-baseline=\"middle\" dominant-baseline=\"middle\" text-anchor=\"middle\" transform=\"translate(40, 80)\"><g><rect class=\"background\" style=\"stroke: none\"></rect><text y=\"-10.1\" style=\"\"><tspan class=\"text-outer-tspan row\" x=\"0\" y=\"-0.1em\" dy=\"1.1em\"><tspan font-style=\"normal\" class=\"text-inner-tspan\" font-weight=\"normal\">{text}</tspan></tspan></text></g></g>",
        text = esc(label),
    ));

    // Icon SVG (80×80 with blue background and white art)
    out.push_str("<g><g>");
    let inner = svc.icon.as_deref().map(icon_inner).unwrap_or(
        "<rect width=\"80\" height=\"80\" style=\"fill: #087ebf; stroke-width: 0px;\"></rect>",
    );
    out.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"80\" height=\"80\" viewBox=\"0 0 80 80\"><g>{inner}</g></svg>",
        inner = inner,
    ));
    out.push_str("</g></g>");

    out.push_str("</g>");
}

// ─── Edge rendering ────────────────────────────────────────────────────────────

fn render_edge(edge: &ArchEdge, layout: &Layout, out: &mut String) {
    let lc = match layout.node_centre(&edge.lhs_id) {
        Some(c) => c,
        None => return,
    };
    let rc = match layout.node_centre(&edge.rhs_id) {
        Some(c) => c,
        None => return,
    };

    let (sx, sy) = anchor(&lc, &edge.lhs_dir);
    let (tx, ty) = anchor(&rc, &edge.rhs_dir);

    // 3-point path: start → midpoint → end (matches mermaid output format)
    let mid_x = (sx + tx) / 2.0;
    let mid_y = (sy + ty) / 2.0;
    let path = format!(
        "M {},{} L {},{} L{},{} ",
        fmt(sx),
        fmt(sy),
        fmt(mid_x),
        fmt(mid_y),
        fmt(tx),
        fmt(ty)
    );

    out.push_str("<g>");
    out.push_str(&format!(
        "<path d=\"{path}\" class=\"edge\" id=\"mermaid-svg-L_{lhs}_{rhs}_0\"></path>",
        path = path,
        lhs = esc(&edge.lhs_id),
        rhs = esc(&edge.rhs_id),
    ));

    // Arrow at rhs (into rhs node)
    if edge.rhs_into {
        let (pts, t) = arrow_at(&rc, &edge.rhs_dir, tx, ty);
        out.push_str(&format!(
            "<polygon points=\"{pts}\" transform=\"{t}\" class=\"arrow\"></polygon>",
            pts = pts,
            t = t,
        ));
    }

    // Arrow at lhs (into lhs node) — for bidirectional edges
    if edge.lhs_into {
        let (pts, t) = arrow_at(&lc, &edge.lhs_dir, sx, sy);
        out.push_str(&format!(
            "<polygon points=\"{pts}\" transform=\"{t}\" class=\"arrow\"></polygon>",
            pts = pts,
            t = t,
        ));
    }

    // Edge label
    if let Some(label) = &edge.title {
        let lx = (sx + tx) / 2.0;
        let ly = (sy + ty) / 2.0 - 6.0;
        out.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"14\" fill=\"#333333\">{text}</text>",
            x = fmt(lx), y = fmt(ly),
            text = esc(label),
        ));
    }

    out.push_str("</g>");
}

/// Returns (polygon_points, transform_string) for an arrowhead at a node edge.
///
/// The arrow polygon points toward the node interior (tip 2px past the icon edge).
/// Polygon shapes and translate formulas are derived from the mermaid reference output.
///
/// Arrow sizes: 13.33... (= 40/3) wide, 13.33... tall.
/// The transform positions the tip 2px inside the icon boundary.
///
/// `dir` is the port direction on the node (e.g. L = arrow enters from left → rightward arrow).
fn arrow_at(centre: &Pos, dir: &Direction, _anchor_x: f64, _anchor_y: f64) -> (String, String) {
    let a = 40.0_f64 / 3.0; // 13.333...
    let ha = a / 2.0; //  6.666...
    let (cx, cy) = (centre.x, centre.y);
    let h = ICON_SIZE / 2.0; // 40

    match dir {
        // Arrow enters at LEFT edge → rightward triangle (tip at right)
        // points: (a, ha), (0, a), (0, 0)
        // translate so that tip_x = left_edge + 2, tip_y = cy
        Direction::L => {
            let left = cx - h;
            let pts = format!("{},{}  0,{}  0,0", fmt(a), fmt(ha), fmt(a));
            // tip at (translate_x + a, translate_y + ha)
            // => translate_x = left + 2 - a = left - (a - 2)
            // => translate_y = cy - ha
            let t = format!("translate({},{})", fmt(left - (a - 2.0)), fmt(cy - ha));
            (pts, t)
        }
        // Arrow enters at RIGHT edge → leftward triangle (tip at left)
        // points: (0, ha), (a, a), (a, 0)
        // translate so that tip_x = right_edge - 2
        Direction::R => {
            let right = cx + h;
            let pts = format!("0,{}  {},{}  {},0", fmt(ha), fmt(a), fmt(a), fmt(a));
            // tip at (translate_x, translate_y + ha)
            // => translate_x = right - 2
            // => translate_y = cy - ha
            let t = format!("translate({},{})", fmt(right - 2.0), fmt(cy - ha));
            (pts, t)
        }
        // Arrow enters at TOP edge → downward triangle (tip at bottom)
        // points: (0, 0), (a, 0), (ha, a)
        // translate so that tip_y = top_edge + 2
        Direction::T => {
            let top = cy - h;
            let pts = format!("0,0  {},0  {},{}", fmt(a), fmt(ha), fmt(a));
            // tip at (translate_x + ha, translate_y + a)
            // => translate_y = top + 2 - a = top - (a - 2)
            // => translate_x = cx - ha
            let t = format!("translate({},{})", fmt(cx - ha), fmt(top - (a - 2.0)));
            (pts, t)
        }
        // Arrow enters at BOTTOM edge → upward triangle (tip at top)
        // points: (0, a), (a, a), (ha, 0)
        // translate so that tip_y = bottom_edge - 2
        Direction::B => {
            let bottom = cy + h;
            let pts = format!("0,{}  {},{}  {},0", fmt(a), fmt(a), fmt(a), fmt(ha));
            // tip at (translate_x + ha, translate_y)
            // => translate_y = bottom - 2
            // => translate_x = cx - ha
            let t = format!("translate({},{})", fmt(cx - ha), fmt(bottom - 2.0));
            (pts, t)
        }
    }
}

// ─── Number formatting ─────────────────────────────────────────────────────────

fn fmt(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e12 {
        return format!("{}", v as i64);
    }
    // Use enough precision then strip trailing zeros
    let s = format!("{:.13}", v);
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

    const ARCH_BASIC: &str = "architecture-beta\n    service api(internet)[API]\n    service db(database)[Database]\n    api:R --> L:db";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(ARCH_BASIC).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "missing <svg tag");
        assert!(svg.contains("API"), "missing service label");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(ARCH_BASIC).diagram;
        let svg = render(&diag, Theme::Dark);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    #[ignore = "platform-specific float precision — run locally"]
    fn snapshot_default_theme() {
        let diag = parser::parse(ARCH_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(svg);
    }
}
