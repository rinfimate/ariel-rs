use super::constants::*;
use super::templates::{self, esc, fmt, fmt_floor};
// Faithful Rust port of mermaid/src/diagrams/venn/vennRenderer.ts
//
// Layout: port of @upsetjs/venn.js greedyLayout + normalizeSolution + scaleSolution.
// Intersection paths: port of intersectionAreaPath / arcsToPath.
// Colors: palette from themeVariables.venn1–venn8, overridden by style directives.

use super::parser::VennDiagram;
use crate::theme::Theme;
use std::collections::HashMap;
use std::f64::consts::PI;

// ─── Geometry primitives ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct Circle {
    x: f64,
    y: f64,
    radius: f64,
    setid: String,
}

fn dist(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt()
}

/// Area of a circular cap of height `width` in a circle of radius `r`.
fn circle_area(r: f64, width: f64) -> f64 {
    r * r * (1.0 - width / r).acos() - (r - width) * (width * (2.0 * r - width)).sqrt()
}

/// Overlap area between two circles of radii r1, r2 with centre distance d.
fn circle_overlap(r1: f64, r2: f64, d: f64) -> f64 {
    if d >= r1 + r2 {
        return 0.0;
    }
    if d <= (r1 - r2).abs() {
        return PI * r1.min(r2).powi(2);
    }
    let w1 = r1 - (d * d - r2 * r2 + r1 * r1) / (2.0 * d);
    let w2 = r2 - (d * d - r1 * r1 + r2 * r2) / (2.0 * d);
    circle_area(r1, w1) + circle_area(r2, w2)
}

/// Find d such that circleOverlap(r1, r2, d) == target_overlap (bisection).
fn distance_from_intersect_area(r1: f64, r2: f64, overlap: f64) -> f64 {
    if overlap < SMALL {
        return r1 + r2;
    }
    let max_overlap = PI * r1.min(r2).powi(2);
    if overlap + SMALL >= max_overlap {
        return (r1 - r2).abs();
    }
    // bisect on [0, r1+r2]
    let f = |d: f64| circle_overlap(r1, r2, d) - overlap;
    let mut a = 0.0_f64;
    let mut b = r1 + r2;
    for _ in 0..100 {
        let mid = (a + b) / 2.0;
        if f(mid) > 0.0 {
            a = mid;
        } else {
            b = mid;
        }
        if (b - a).abs() < 1e-10 {
            break;
        }
    }
    (a + b) / 2.0
}

/// Two intersection points of two circles (empty vec if no intersection).
fn circle_circle_intersection(
    x1: f64,
    y1: f64,
    r1: f64,
    x2: f64,
    y2: f64,
    r2: f64,
) -> Vec<(f64, f64)> {
    let d = dist(x1, y1, x2, y2);
    if d >= r1 + r2 || d <= (r1 - r2).abs() {
        return vec![];
    }
    let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
    let h = (r1 * r1 - a * a).max(0.0).sqrt();
    let x0 = x1 + a * (x2 - x1) / d;
    let y0 = y1 + a * (y2 - y1) / d;
    let rx = -(y2 - y1) * (h / d);
    let ry = -(x2 - x1) * (h / d);
    vec![(x0 + rx, y0 - ry), (x0 - rx, y0 + ry)]
}

// ─── venn.js layout port ──────────────────────────────────────────────────────

/// Greedy layout: place circles one at a time to minimise a loss function.
fn greedy_layout(sets_data: &[(String, f64)], pairs: &[(usize, usize, f64)]) -> Vec<Circle> {
    let n = sets_data.len();
    if n == 0 {
        return vec![];
    }

    let mut circles: Vec<Circle> = sets_data
        .iter()
        .map(|(id, size)| Circle {
            x: 1e10,
            y: 1e10,
            radius: (*size / PI).sqrt(),
            setid: id.clone(),
        })
        .collect();

    if n == 1 {
        circles[0].x = 0.0;
        circles[0].y = 0.0;
        return circles;
    }

    // Build set-overlap list per set
    let mut set_overlaps: Vec<Vec<(usize, f64)>> = vec![vec![]; n];
    for &(li, ri, size) in pairs {
        let l_size = circles[li].radius * circles[li].radius * PI;
        let r_size = circles[ri].radius * circles[ri].radius * PI;
        let weight = if size + 1e-10 >= l_size.min(r_size) {
            0.0
        } else {
            1.0
        };
        if weight > 0.0 {
            set_overlaps[li].push((ri, size));
            set_overlaps[ri].push((li, size));
        }
    }

    // Sort sets by total overlap (most overlapped first)
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        let sa: f64 = set_overlaps[a].iter().map(|(_, s)| s).sum();
        let sb: f64 = set_overlaps[b].iter().map(|(_, s)| s).sum();
        sb.partial_cmp(&sa).unwrap()
    });

    // Place first circle at origin
    let first = order[0];
    circles[first].x = 0.0;
    circles[first].y = 0.0;

    let mut placed = vec![false; n];
    placed[first] = true;

    for &idx in &order[1..] {
        let mut best_loss = f64::INFINITY;
        let mut best_x = 0.0_f64;
        let mut best_y = 0.0_f64;

        let placed_indices: Vec<usize> = (0..n).filter(|&i| placed[i]).collect();
        let r_idx = circles[idx].radius;

        // Candidate positions: along the line between circle pairs + current
        let mut candidates: Vec<(f64, f64)> = Vec::new();
        for &pi in &placed_indices {
            let r_pi = circles[pi].radius;
            let pair_size = pairs
                .iter()
                .find(|&&(li, ri, _)| (li == idx && ri == pi) || (li == pi && ri == idx))
                .map(|&(_, _, s)| s)
                .unwrap_or(0.0);
            let desired_d = distance_from_intersect_area(r_pi, r_idx, pair_size);

            // Place along various angles
            let n_angles = if placed_indices.len() == 1 { 4 } else { 12 };
            for k in 0..n_angles {
                let angle = 2.0 * PI * k as f64 / n_angles as f64;
                candidates.push((
                    circles[pi].x + desired_d * angle.cos(),
                    circles[pi].y + desired_d * angle.sin(),
                ));
            }

            // Also try positions at intersection of distance circles from pi and pj
            for &pj in &placed_indices {
                if pj == pi {
                    continue;
                }
                let pair_size_j = pairs
                    .iter()
                    .find(|&&(li, ri, _)| (li == idx && ri == pj) || (li == pj && ri == idx))
                    .map(|&(_, _, s)| s)
                    .unwrap_or(0.0);
                let desired_d_j =
                    distance_from_intersect_area(circles[pj].radius, r_idx, pair_size_j);
                let pts = circle_circle_intersection(
                    circles[pi].x,
                    circles[pi].y,
                    desired_d,
                    circles[pj].x,
                    circles[pj].y,
                    desired_d_j,
                );
                candidates.extend(pts);
            }
        }

        if candidates.is_empty() {
            candidates.push((0.0, 0.0));
        }

        for (cx, cy) in candidates {
            let mut loss = 0.0;
            for &pi in &placed_indices {
                let pair_size = pairs
                    .iter()
                    .find(|&&(li, ri, _)| (li == idx && ri == pi) || (li == pi && ri == idx))
                    .map(|&(_, _, s)| s)
                    .unwrap_or(0.0);
                let d = dist(cx, cy, circles[pi].x, circles[pi].y);
                let actual_overlap = circle_overlap(r_idx, circles[pi].radius, d);
                let diff = actual_overlap - pair_size;
                loss += diff * diff;
            }
            if loss < best_loss {
                best_loss = loss;
                best_x = cx;
                best_y = cy;
            }
        }

        circles[idx].x = best_x;
        circles[idx].y = best_y;
        placed[idx] = true;
    }

    circles
}

/// Rotate circles so the largest pair aligns to `orientation` (default π/2).
fn orientate_circles(circles: &mut [Circle], orientation: f64) {
    if circles.len() < 2 {
        return;
    }
    // Shift so largest circle is at origin
    let lx = circles[0].x;
    let ly = circles[0].y;
    for c in circles.iter_mut() {
        c.x -= lx;
        c.y -= ly;
    }

    if circles.len() == 2 {
        let d = dist(circles[0].x, circles[0].y, circles[1].x, circles[1].y);
        let r_diff = (circles[1].radius - circles[0].radius).abs();
        if d < r_diff {
            circles[1].x = circles[0].x + circles[0].radius - circles[1].radius - 1e-10;
            circles[1].y = circles[0].y;
        }
    }

    // Rotate so second-largest aligns to orientation
    let rotation = circles[1].x.atan2(circles[1].y) - orientation;
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();
    for c in circles.iter_mut() {
        let x = c.x;
        let y = c.y;
        c.x = cos_r * x - sin_r * y;
        c.y = sin_r * x + cos_r * y;
    }

    // Flip if third circle is on wrong side
    if circles.len() > 2 {
        let mut angle = circles[2].x.atan2(circles[2].y) - orientation;
        while angle < 0.0 {
            angle += 2.0 * PI;
        }
        while angle > 2.0 * PI {
            angle -= 2.0 * PI;
        }
        if angle > PI {
            let slope = circles[1].y / (1e-10 + circles[1].x);
            for c in circles.iter_mut() {
                let d = (c.x + slope * c.y) / (1.0 + slope * slope);
                c.x = 2.0 * d - c.x;
                c.y = 2.0 * d * slope - c.y;
            }
        }
    }
}

fn get_bounding_box(circles: &[Circle]) -> ([f64; 2], [f64; 2]) {
    let x_min = circles
        .iter()
        .fold(f64::INFINITY, |acc, c| acc.min(c.x - c.radius));
    let x_max = circles
        .iter()
        .fold(f64::NEG_INFINITY, |acc, c| acc.max(c.x + c.radius));
    let y_min = circles
        .iter()
        .fold(f64::INFINITY, |acc, c| acc.min(c.y - c.radius));
    let y_max = circles
        .iter()
        .fold(f64::NEG_INFINITY, |acc, c| acc.max(c.y + c.radius));
    ([x_min, x_max], [y_min, y_max])
}

/// Scale solution to fit within width×height with given padding.
fn scale_solution(circles: &mut [Circle], width: f64, height: f64, padding: f64) {
    let w = width - 2.0 * padding;
    let h = height - 2.0 * padding;
    let ([x_min, x_max], [y_min, y_max]) = get_bounding_box(circles);

    if (x_max - x_min).abs() < 1e-10 || (y_max - y_min).abs() < 1e-10 {
        return;
    }

    let x_scaling = w / (x_max - x_min);
    let y_scaling = h / (y_max - y_min);
    let scaling = x_scaling.min(y_scaling);

    let x_offset = (w - (x_max - x_min) * scaling) / 2.0;
    let y_offset = (h - (y_max - y_min) * scaling) / 2.0;

    for c in circles.iter_mut() {
        c.radius *= scaling;
        c.x = padding + x_offset + (c.x - x_min) * scaling;
        c.y = padding + y_offset + (c.y - y_min) * scaling;
    }
}

// ─── Intersection path (arc-arc paths) ───────────────────────────────────────

#[derive(Debug, Clone)]
struct Arc {
    circle_x: f64,
    circle_y: f64,
    circle_r: f64,
    p1x: f64,
    p1y: f64,
    p2x: f64,
    p2y: f64,
    large: bool,
    sweep: bool,
}

/// Compute the intersection path arcs for the given set of circles.
fn intersection_area_arcs(circles: &[&Circle]) -> Vec<Arc> {
    if circles.is_empty() {
        return vec![];
    }
    if circles.len() == 1 {
        let c = circles[0];
        return vec![Arc {
            circle_x: c.x,
            circle_y: c.y,
            circle_r: c.radius,
            p1x: c.x,
            p1y: c.y + c.radius,
            p2x: c.x - 1e-10,
            p2y: c.y + c.radius,
            large: true,
            sweep: true,
        }];
    }

    // Compute all pairwise intersection points
    #[derive(Clone, Debug)]
    struct IntPoint {
        x: f64,
        y: f64,
        parent_index: Vec<usize>,
    }

    let mut all_points: Vec<IntPoint> = Vec::new();
    for i in 0..circles.len() {
        for j in (i + 1)..circles.len() {
            let pts = circle_circle_intersection(
                circles[i].x,
                circles[i].y,
                circles[i].radius,
                circles[j].x,
                circles[j].y,
                circles[j].radius,
            );
            for (px, py) in pts {
                all_points.push(IntPoint {
                    x: px,
                    y: py,
                    parent_index: vec![i, j],
                });
            }
        }
    }

    // Keep only points inside all circles
    let inner_points: Vec<IntPoint> = all_points
        .into_iter()
        .filter(|p| {
            circles
                .iter()
                .all(|c| dist(p.x, p.y, c.x, c.y) < c.radius + 1e-10)
        })
        .collect();

    if inner_points.len() < 2 {
        // Either fully contained or disjoint
        // Find smallest circle
        let smallest = circles
            .iter()
            .min_by(|a, b| a.radius.partial_cmp(&b.radius).unwrap())
            .unwrap();
        // Check if all circles contain smallest's centre
        let all_contain = circles.iter().all(|c| {
            dist(smallest.x, smallest.y, c.x, c.y) <= (smallest.radius - c.radius).abs() + 1e-10
        });
        if !all_contain {
            return vec![];
        }
        return vec![Arc {
            circle_x: smallest.x,
            circle_y: smallest.y,
            circle_r: smallest.radius,
            p1x: smallest.x,
            p1y: smallest.y + smallest.radius,
            p2x: smallest.x - 1e-10,
            p2y: smallest.y + smallest.radius,
            large: true,
            sweep: true,
        }];
    }

    // Sort inner_points by angle around their centroid
    let cx_mean = inner_points.iter().map(|p| p.x).sum::<f64>() / inner_points.len() as f64;
    let cy_mean = inner_points.iter().map(|p| p.y).sum::<f64>() / inner_points.len() as f64;

    let mut sorted_points = inner_points.clone();
    sorted_points.sort_by(|a, b| {
        let ang_a = (a.x - cx_mean).atan2(a.y - cy_mean);
        let ang_b = (b.x - cx_mean).atan2(b.y - cy_mean);
        ang_b.partial_cmp(&ang_a).unwrap()
    });

    let n = sorted_points.len();
    let mut arcs: Vec<Arc> = Vec::new();

    // Iterate pairs (p2, p1) = (sorted_points[n-1], sorted_points[0]), ..., wrapping
    for idx in 0..n {
        let p1 = &sorted_points[idx];
        let p2 = &sorted_points[(idx + n - 1) % n];
        let mid_x = (p1.x + p2.x) / 2.0;
        let mid_y = (p1.y + p2.y) / 2.0;

        // Find the best (smallest width) shared circle arc
        let mut best_arc: Option<Arc> = None;
        let mut best_width = f64::INFINITY;

        for &j in &p1.parent_index {
            if !p2.parent_index.contains(&j) {
                continue;
            }
            let c = circles[j];
            let a1 = (p1.x - c.x).atan2(p1.y - c.y);
            let a2 = (p2.x - c.x).atan2(p2.y - c.y);
            let mut angle_diff = a2 - a1;
            if angle_diff < 0.0 {
                angle_diff += 2.0 * PI;
            }
            let a_mid = a2 - angle_diff / 2.0;
            let mid_on_circle_x = c.x + c.radius * a_mid.sin();
            let mid_on_circle_y = c.y + c.radius * a_mid.cos();
            let mut width = dist(mid_x, mid_y, mid_on_circle_x, mid_on_circle_y);
            if width > c.radius * 2.0 {
                width = c.radius * 2.0;
            }
            if width < best_width {
                best_width = width;
                best_arc = Some(Arc {
                    circle_x: c.x,
                    circle_y: c.y,
                    circle_r: c.radius,
                    p1x: p1.x,
                    p1y: p1.y,
                    p2x: p2.x,
                    p2y: p2.y,
                    large: width > c.radius,
                    sweep: true,
                });
            }
        }

        if let Some(arc) = best_arc {
            arcs.push(arc);
        }
    }

    arcs
}

/// Convert arcs to SVG path string (port of arcsToPath).
fn arcs_to_path(arcs: &[Arc]) -> String {
    if arcs.is_empty() {
        return "M 0 0".to_string();
    }
    if arcs.len() == 1 {
        // Full circle path
        let a = &arcs[0];
        return circle_path(a.circle_x, a.circle_y, a.circle_r);
    }
    let first = &arcs[0];
    let mut parts = vec![format!("\nM {} {}", fmt(first.p2x), fmt(first.p2y))];
    for arc in arcs {
        let r = fmt(arc.circle_r);
        let large = if arc.large { 1 } else { 0 };
        let sweep = if arc.sweep { 1 } else { 0 };
        parts.push(format!(
            "\nA {} {} 0 {} {} {} {}",
            r,
            r,
            large,
            sweep,
            fmt(arc.p1x),
            fmt(arc.p1y)
        ));
    }
    parts.join(" ")
}

/// Port of venn.js circlePath(x, y, r) — uses relative moves.
fn circle_path(cx: f64, cy: f64, r: f64) -> String {
    format!(
        "\nM {} {}\nm {} 0\na {r} {r} 0 1 0 {} 0\na {r} {r} 0 1 0 {} 0",
        fmt(cx),
        fmt(cy),
        fmt(-r),
        fmt(r * 2.0),
        fmt(-r * 2.0),
        r = fmt(r),
    )
}

// ─── Text centre computation ──────────────────────────────────────────────────

/// Compute the approximate visual centre for a set of circles (for label placement).
/// Uses gradient ascent to find the point that maximises the minimum distance to all
/// circle boundaries, mirroring the Nelder-Mead optimisation in vennRenderer.ts.
fn compute_text_centre(interior: &[&Circle], exterior: &[&Circle]) -> (f64, f64) {
    if interior.is_empty() {
        return (0.0, 0.0);
    }

    // Initial candidates: circle centres + intersection points
    let mut candidates: Vec<(f64, f64)> = interior.iter().map(|c| (c.x, c.y)).collect();

    for c in interior {
        candidates.push((c.x + c.radius / 2.0, c.y));
        candidates.push((c.x - c.radius / 2.0, c.y));
        candidates.push((c.x, c.y + c.radius / 2.0));
        candidates.push((c.x, c.y - c.radius / 2.0));
    }

    // Add all pairwise circle-circle intersection points inside interior
    for i in 0..interior.len() {
        for j in (i + 1)..interior.len() {
            let pts = circle_circle_intersection(
                interior[i].x,
                interior[i].y,
                interior[i].radius,
                interior[j].x,
                interior[j].y,
                interior[j].radius,
            );
            for (px, py) in pts {
                candidates.push((px, py));
            }
        }
    }

    let margin = |px: f64, py: f64| circle_margin_single(px, py, interior, exterior);

    // Find best candidate
    let mut best = candidates[0];
    let mut best_m = margin(best.0, best.1);
    for (px, py) in &candidates {
        let m = margin(*px, *py);
        if m > best_m {
            best_m = m;
            best = (*px, *py);
        }
    }

    // If best margin is negative (no valid point found), fall back to centroid of intersection
    // points that are inside all interior circles
    if best_m < 0.0 {
        let valid: Vec<(f64, f64)> = candidates
            .iter()
            .filter(|(px, py)| {
                interior
                    .iter()
                    .all(|c| dist(*px, *py, c.x, c.y) <= c.radius + 1e-6)
            })
            .copied()
            .collect();
        if !valid.is_empty() {
            let mx = valid.iter().map(|(x, _)| x).sum::<f64>() / valid.len() as f64;
            let my = valid.iter().map(|(_, y)| y).sum::<f64>() / valid.len() as f64;
            return (mx, my);
        }
        let mx = interior.iter().map(|c| c.x).sum::<f64>() / interior.len() as f64;
        let my = interior.iter().map(|c| c.y).sum::<f64>() / interior.len() as f64;
        return (mx, my);
    }

    // Gradient ascent refinement
    let step_init = interior.iter().map(|c| c.radius).fold(0.0_f64, f64::max) * 0.1;
    let mut step = step_init;
    let mut pos = best;
    for _ in 0..500 {
        let m0 = margin(pos.0, pos.1);
        let mut moved = false;
        for &(dx, dy) in &[(step, 0.0), (-step, 0.0), (0.0, step), (0.0, -step)] {
            let nx = pos.0 + dx;
            let ny = pos.1 + dy;
            let m = margin(nx, ny);
            if m > m0 {
                pos = (nx, ny);
                moved = true;
                break;
            }
        }
        if !moved {
            step *= 0.5;
            if step < 1e-10 {
                break;
            }
        }
    }

    pos
}

// ─── Style helpers ────────────────────────────────────────────────────────────

/// Build a map from sorted-set-key (sets joined by "|") → style map.
fn build_style_map(diag: &VennDiagram) -> HashMap<String, HashMap<String, String>> {
    let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
    for entry in &diag.style_entries {
        let key = entry.targets.join("|");
        let existing = map.entry(key).or_default();
        for (k, v) in &entry.styles {
            existing.insert(k.clone(), v.clone());
        }
    }
    map
}

fn sets_key(sets: &[String]) -> String {
    let mut s = sets.to_vec();
    s.sort();
    s.join("|")
}

/// Darken a CSS named color or hex color by ~30% (for set label text).
fn darken_color(color: &str) -> String {
    // Handle hex colors
    if color.starts_with('#') && color.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&color[1..3], 16),
            u8::from_str_radix(&color[3..5], 16),
            u8::from_str_radix(&color[5..7], 16),
        ) {
            let factor = 0.6_f64;
            return format!(
                "#{:02X}{:02X}{:02X}",
                (r as f64 * factor) as u8,
                (g as f64 * factor) as u8,
                (b as f64 * factor) as u8,
            );
        }
    }
    // Named colors — return as-is (darken is only used for label readability,
    // the reference JS uses darken(baseColor, 30) which works on hex)
    color.to_string()
}

// ─── Main render ──────────────────────────────────────────────────────────────

pub fn render(diag: &VennDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let scale2 = SVG_WIDTH / REFERENCE_WIDTH;

    let title_height = if diag.title.is_some() {
        48.0 * scale2 // matches Mermaid reference translate offset
    } else {
        0.0
    };
    let chart_h = SVG_HEIGHT - title_height;

    // Build subset data: sets (size 10 each) + pairwise unions (size 10/4 each by default)
    let n = diag.sets.len();

    // Collect sizes: default size per the JS parser is 10 / len^2
    // For individual sets: size = 10
    let sets_data: Vec<(String, f64)> = diag.sets.iter().map(|s| (s.id.clone(), 10.0)).collect();

    // Build pairs for greedy layout (only 2-set intersections matter for layout)
    let mut pairs: Vec<(usize, usize, f64)> = Vec::new();
    let set_index_map: HashMap<String, usize> = diag
        .sets
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.clone(), i))
        .collect();

    for inter in &diag.intersections {
        if inter.sets.len() == 2 {
            if let (Some(&li), Some(&ri)) = (
                set_index_map.get(&inter.sets[0]),
                set_index_map.get(&inter.sets[1]),
            ) {
                // Default size = 10 / 4 = 2.5
                pairs.push((li, ri, 2.5));
            }
        }
    }

    // If no explicit pairs, add all pairwise combinations with default size
    if pairs.is_empty() && n >= 2 {
        for i in 0..n {
            for j in (i + 1)..n {
                pairs.push((i, j, 2.5));
            }
        }
    }

    // Run layout
    let mut circles = greedy_layout(&sets_data, &pairs);

    // Sort by radius descending (as normalizeSolution does)
    circles.sort_by(|a, b| b.radius.partial_cmp(&a.radius).unwrap());

    // Orientate
    orientate_circles(&mut circles, PI / 2.0);

    // Scale
    scale_solution(&mut circles, SVG_WIDTH, chart_h, 15.0);

    // Rebuild lookup by setid
    let circle_map: HashMap<String, usize> = circles
        .iter()
        .enumerate()
        .map(|(i, c)| (c.setid.clone(), i))
        .collect();

    let style_map = build_style_map(diag);

    // Theme colors (venn1..venn8)
    let theme_colors = vars.venn_colors;
    let default_text_color = vars.text_color;

    // Circle ordering for index (venn-set-0, venn-set-1, ...) follows the order
    // in diag.sets (not the sorted circles order)
    // Build set order: for each set in diag.sets, find its circle
    let mut out = String::new();

    out.push_str(&templates::svg_root(
        SVG_ID,
        &fmt(SVG_WIDTH),
        &fmt(SVG_HEIGHT),
    ));
    out.push_str(&templates::style_block_venn(vars.font_family));

    // Title
    if let Some(t) = &diag.title {
        let title_y = 32.0 * scale2; // matches Mermaid reference y position
        out.push_str(&templates::title_text_venn(
            &fmt(title_y),
            1.0, // title font is 32px regardless of scale (matches Mermaid reference)
            vars.venn_title_text_color,
            &esc(t),
        ));
    }

    // Main group translated by titleHeight
    out.push_str(&templates::main_group_open(&fmt(title_height)));

    // ── venn-circle groups ───────────────────────────────────────────────────
    for (set_i, set) in diag.sets.iter().enumerate() {
        let Some(&ci) = circle_map.get(&set.id) else {
            continue;
        };
        let c = &circles[ci];

        let skey = set.id.clone();
        let custom_style = style_map.get(&skey);

        let base_color = custom_style
            .and_then(|s| s.get("fill"))
            .map(|s| s.as_str())
            .unwrap_or_else(|| theme_colors[set_i % theme_colors.len()]);

        let fill_opacity = custom_style
            .and_then(|s| s.get("fill-opacity"))
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.1);

        let stroke_color = custom_style
            .and_then(|s| s.get("stroke"))
            .map(|s| s.as_str())
            .unwrap_or(base_color);

        let stroke_width = custom_style
            .and_then(|s| s.get("stroke-width"))
            .cloned()
            .unwrap_or_else(|| fmt(5.0 * scale2));

        let text_color = custom_style
            .and_then(|s| s.get("color"))
            .map(|s| s.as_str())
            .unwrap_or_else(|| "");

        let label_color = if text_color.is_empty() {
            darken_color(base_color)
        } else {
            text_color.to_string()
        };

        let path_d = circle_path(c.x, c.y, c.radius);
        let label = set.label.as_deref().unwrap_or(&set.id);
        let font_size = 48.0 * scale2;

        // Text position: use computeTextCentre for single set
        // For single circle label, reference uses circle's centre x and a y above
        // (the label text is placed at the centre according to wrapText, but
        //  in the reference the label appears at the circle centre since it's the
        //  only interior region)
        // Looking at the reference: A at (276, 307), B at (523, 307), C at (399, 93)
        // Circle A centre: (319.16, 283.01) → label at (276, 307)? That's not the centre.
        // Actually in the reference, the text positions come from textCentres computed by
        // computeTextCentres which uses computeTextCentre with interior=[circle] exterior=[other circles]
        // The label for A is displaced away from the intersection area.
        // For simplicity we use the computed text centres.

        // Actually, looking at reference carefully:
        // A centre=(319.16, 283.01), label at x=276, y=307 → below and left of circle centre
        // B centre=(480.84, 283.01), label at x=523, y=307 → below and right
        // C centre=(400.00, 143.00), label at x=399, y=93 → above circle centre

        // The text centre for each set (excluding area inside other circles) is computed by
        // computeTextCentre(interior=[circle_A], exterior=[circle_B, circle_C])
        // This is the nelderMead optimization. We'll approximate it analytically.

        let interior = vec![c];
        let exterior: Vec<&Circle> = circles.iter().filter(|cc| cc.setid != set.id).collect();
        let (tx, ty) = compute_text_centre_set(&interior, &exterior);

        out.push_str(&templates::venn_circle_group_open(set_i % 8, &esc(&set.id)));
        out.push_str(&templates::venn_circle_path(
            &path_d,
            &fmt(fill_opacity),
            base_color,
            stroke_color,
            &stroke_width,
        ));
        out.push_str(&templates::venn_set_label_open(
            &fmt_floor(tx),
            &fmt_floor(ty),
            &label_color,
            &fmt(font_size),
        ));
        out.push_str(&templates::venn_label_tspan(
            &fmt_floor(tx),
            &fmt_floor(ty),
            &esc(label),
        ));
        out.push_str("</text>");
        out.push_str("</g>");
    }

    // ── venn-intersection groups ──────────────────────────────────────────────
    // All subsets with 2+ sets (unions from the diagram)
    // We need to process ALL subsets in the layout order (as the venn.js does)
    // The reference SVG shows intersections in order: A_B, B_C, A_C, A_B_C

    // Build all intersection entries from the diagram
    for inter in &diag.intersections {
        let set_ids = &inter.sets; // already sorted
        let skey = sets_key(set_ids);
        let custom_style = style_map.get(&skey);

        // Gather circles for this intersection
        let int_circles: Vec<&Circle> = set_ids
            .iter()
            .filter_map(|id| circle_map.get(id).map(|&i| &circles[i]))
            .collect();

        if int_circles.is_empty() {
            continue;
        }

        // Compute intersection path
        let arcs = intersection_area_arcs(&int_circles);
        let path_d = arcs_to_path(&arcs);

        // Fill opacity and color from custom style
        let fill_opacity = if custom_style.and_then(|s| s.get("fill")).is_some() {
            "1"
        } else {
            "0"
        };
        let fill_color = custom_style
            .and_then(|s| s.get("fill"))
            .map(|s| s.as_str())
            .unwrap_or("transparent");

        // Text color for intersection label
        let text_color = custom_style
            .and_then(|s| s.get("color"))
            .map(|s| s.as_str())
            .unwrap_or(default_text_color);

        // Text centre

        let exterior: Vec<&Circle> = circles
            .iter()
            .filter(|c| !set_ids.contains(&c.setid))
            .collect();
        let (tx, ty) = compute_text_centre(&int_circles, &exterior);

        let data_venn = set_ids.join("_");
        let font_size = 48.0 * scale2;

        out.push_str(&templates::venn_intersection_group_open(&esc(&data_venn)));
        out.push_str(&templates::venn_intersection_path(
            &path_d,
            fill_opacity,
            fill_color,
        ));

        if let Some(label) = &inter.label {
            out.push_str(&templates::venn_set_label_open(
                &fmt_floor(tx),
                &fmt_floor(ty),
                text_color,
                &fmt(font_size),
            ));
            out.push_str(&templates::venn_label_tspan(
                &fmt_floor(tx),
                &fmt_floor(ty),
                &esc(label),
            ));
            out.push_str("</text>");
        }
        out.push_str("</g>");
    }

    out.push_str("</g>"); // translate group
    out.push_str("</svg>");
    out
}

/// Margin function: minimum clearance from all circle boundaries at a test point.
/// Positive = point is in the allowed region (inside all interior, outside all exterior).
fn circle_margin_single(px: f64, py: f64, interior: &[&Circle], exterior: &[&Circle]) -> f64 {
    let mut m = f64::INFINITY;
    for c in interior {
        let d = c.radius - dist(px, py, c.x, c.y);
        if d < m {
            m = d;
        }
    }
    for e in exterior {
        let d = dist(px, py, e.x, e.y) - e.radius;
        if d < m {
            m = d;
        }
    }
    m
}

/// Compute text centre for a single set circle given the other (exterior) circles.
/// Approximates the Nelder-Mead result using a grid search followed by gradient ascent.
fn compute_text_centre_set(interior: &[&Circle], exterior: &[&Circle]) -> (f64, f64) {
    if interior.is_empty() {
        return (0.0, 0.0);
    }
    let c = interior[0];
    if exterior.is_empty() {
        return (c.x, c.y);
    }

    // Initial candidate: circle centre + a set of points on rings inside the circle
    let mut candidates: Vec<(f64, f64)> = vec![
        (c.x, c.y),
        (c.x + c.radius / 2.0, c.y),
        (c.x - c.radius / 2.0, c.y),
        (c.x, c.y + c.radius / 2.0),
        (c.x, c.y - c.radius / 2.0),
    ];
    for k in 0..24 {
        let angle = 2.0 * PI * k as f64 / 24.0;
        for &frac in &[0.25_f64, 0.5, 0.75] {
            candidates.push((
                c.x + c.radius * frac * angle.cos(),
                c.y + c.radius * frac * angle.sin(),
            ));
        }
    }

    let margin = |px: f64, py: f64| circle_margin_single(px, py, interior, exterior);

    // Find best candidate
    let mut best = (c.x, c.y);
    let mut best_m = margin(c.x, c.y);
    for (px, py) in candidates {
        let m = margin(px, py);
        if m > best_m {
            best_m = m;
            best = (px, py);
        }
    }

    // Gradient ascent refinement (500 iterations, mirrors nelderMead maxIterations=500)
    let mut step = c.radius * 0.1;
    let mut pos = best;
    for _ in 0..500 {
        let m0 = margin(pos.0, pos.1);
        let mut moved = false;
        for &(dx, dy) in &[(step, 0.0), (-step, 0.0), (0.0, step), (0.0, -step)] {
            let nx = pos.0 + dx;
            let ny = pos.1 + dy;
            let m = margin(nx, ny);
            if m > m0 {
                pos = (nx, ny);
                moved = true;
                break;
            }
        }
        if !moved {
            step *= 0.5;
            if step < 1e-10 {
                break;
            }
        }
    }

    pos
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    const VENN_BASIC: &str = "vennDiagram\n    title Sets\n    set A\n    set B\n    A&B";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(VENN_BASIC).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(VENN_BASIC).diagram;
        let svg = render(&diag, Theme::Dark);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn snapshot_default_theme() {
        let diag = parser::parse(VENN_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
