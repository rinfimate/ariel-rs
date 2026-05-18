// Faithful Rust port of mermaid/src/diagrams/radar/renderer.ts
//
// Key functions ported:
//   drawFrame   – centres the chart, sets margins
//   drawGraticule – concentric circles or polygons for grid lines
//   drawAxes    – radial spokes + axis labels
//   drawCurves  – data polygons or Catmull-Rom paths
//   drawLegend  – colour legend in top-right
//   relativeRadius – normalise value to 0..radius
//   closedRoundCurve – Catmull-Rom closed spline

use super::constants::*;
use super::parser::{GraticuleType, RadarDiagram};
use super::templates;
use crate::text::measure;
use crate::theme::Theme;
use std::f64::consts::PI;

pub fn render(diag: &RadarDiagram, theme: Theme) -> String {
    let vars = theme.resolve();

    let has_legend = diag.options.show_legend && !diag.curves.is_empty();
    let right_margin = if has_legend {
        MARGIN_RIGHT + LEGEND_WIDTH + 20.0
    } else {
        MARGIN_RIGHT
    };

    // Inner chart area
    let inner_w = SVG_WIDTH - MARGIN_LEFT - right_margin;
    let inner_h = SVG_HEIGHT - MARGIN_TOP - MARGIN_BOTTOM;
    let radius = (inner_w.min(inner_h) / 2.0).max(1.0);

    // Chart centre in SVG coords
    let cx = MARGIN_LEFT + inner_w / 2.0;
    let cy = MARGIN_TOP + inner_h / 2.0;

    let n_axes = diag.axes.len();

    // Compute max value across all curves
    let data_max = diag
        .curves
        .iter()
        .flat_map(|c| c.entries.iter().copied())
        .fold(f64::NEG_INFINITY, f64::max);
    let max_val = diag
        .options
        .max
        .unwrap_or(if data_max > 0.0 { data_max } else { 1.0 });
    let min_val = diag.options.min;
    let ticks = diag.options.ticks.max(1);

    let mut out = String::new();

    out.push_str(&templates::svg_root(&fmt(SVG_WIDTH), &fmt(SVG_HEIGHT)));
    out.push_str(&templates::style_block(vars.font_family, vars.text_color));

    // Title
    if let Some(t) = &diag.title {
        out.push_str(&templates::title_text(
            &fmt(SVG_WIDTH / 2.0),
            vars.font_family,
            &fmt(TITLE_FONT),
            vars.title_color,
            &esc(t),
        ));
    }

    // ── drawGraticule ─────────────────────────────────────────────────────────
    out.push_str(r#"<g class="graticule">"#);
    for i in 1..=ticks {
        let r = radius * (i as f64 / ticks as f64);
        if diag.options.graticule == GraticuleType::Circle {
            out.push_str(&templates::graticule_circle(
                &fmt(cx),
                &fmt(cy),
                &fmt(r),
                vars.line_color,
            ));
        } else {
            // Polygon graticule
            if n_axes >= 3 {
                let pts = polygon_points(cx, cy, r, n_axes);
                out.push_str(&templates::graticule_polygon(&pts, vars.line_color));
            }
        }

        // Tick label (value at this ring)
        let tick_val = min_val + (max_val - min_val) * (i as f64 / ticks as f64);
        let tick_str = if tick_val == tick_val.floor() {
            format!("{:.0}", tick_val)
        } else {
            format!("{:.2}", tick_val)
        };
        out.push_str(&templates::graticule_tick_label(
            &fmt(cx),
            &fmt(cy - r - 2.0),
            vars.font_family,
            vars.text_color,
            &esc(&tick_str),
        ));
    }
    out.push_str("</g>");

    // ── drawAxes ──────────────────────────────────────────────────────────────
    out.push_str(r#"<g class="axes">"#);
    for (i, axis) in diag.axes.iter().enumerate() {
        let angle = axis_angle(i, n_axes);
        let ax = cx + radius * angle.cos();
        let ay = cy + radius * angle.sin();

        // Spoke line
        out.push_str(&templates::axis_spoke(
            &fmt(cx),
            &fmt(cy),
            &fmt(ax),
            &fmt(ay),
            vars.line_color,
        ));

        // Axis label: positioned just outside the radius tip
        let label_r = radius + AXIS_LABEL_RADIUS_OFFSET;
        let lx = cx + label_r * angle.cos();
        let ly = cy + label_r * angle.sin();

        // Text anchor depends on which side of the chart
        let anchor = if angle.cos() > AXIS_LABEL_ANCHOR_THRESHOLD {
            "start"
        } else if angle.cos() < -AXIS_LABEL_ANCHOR_THRESHOLD {
            "end"
        } else {
            "middle"
        };
        let display_label = &axis.label;
        let (_, lh) = measure(display_label, AXIS_LABEL_FONT);
        // Vertical adjustment: shift down half a line-height when above center
        let dy = if angle.sin() < -AXIS_LABEL_ANCHOR_THRESHOLD {
            -lh / 2.0
        } else if angle.sin() > AXIS_LABEL_ANCHOR_THRESHOLD {
            lh / 2.0
        } else {
            lh * 0.35
        };

        out.push_str(&templates::axis_label(
            &fmt(lx),
            &fmt(ly),
            &fmt(dy),
            anchor,
            vars.font_family,
            &fmt(AXIS_LABEL_FONT),
            vars.text_color,
            &esc(display_label),
        ));
    }
    out.push_str("</g>");

    // ── drawCurves ────────────────────────────────────────────────────────────
    out.push_str(r#"<g class="curves">"#);
    for (ci, curve) in diag.curves.iter().enumerate() {
        let color = CURVE_COLORS[ci % CURVE_COLORS.len()];

        // Map entries to (x, y) points
        let points: Vec<(f64, f64)> = curve
            .entries
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let r = relative_radius(v, min_val, max_val, radius);
                let angle = axis_angle(i, n_axes);
                (cx + r * angle.cos(), cy + r * angle.sin())
            })
            .collect();

        if points.len() < 3 {
            continue;
        }

        let path_d = if diag.options.graticule == GraticuleType::Circle {
            closed_round_curve(&points)
        } else {
            polygon_path(&points)
        };

        out.push_str(&templates::curve_path(&path_d, color));

        // Draw data points
        for (px, py) in &points {
            out.push_str(&templates::data_point(&fmt(*px), &fmt(*py), color));
        }
    }
    out.push_str("</g>");

    // ── drawLegend ────────────────────────────────────────────────────────────
    if has_legend {
        let lx = SVG_WIDTH - LEGEND_WIDTH - 10.0;
        let ly_start = MARGIN_TOP;
        out.push_str(r#"<g class="legend">"#);
        for (ci, curve) in diag.curves.iter().enumerate() {
            let color = CURVE_COLORS[ci % CURVE_COLORS.len()];
            let item_y = ly_start + ci as f64 * (LEGEND_BOX + 6.0);
            out.push_str(&templates::legend_rect(
                &fmt(lx),
                &fmt(item_y),
                &fmt(LEGEND_BOX),
                &fmt(LEGEND_BOX),
                color,
            ));
            out.push_str(&templates::legend_label(
                &fmt(lx + LEGEND_BOX + 6.0),
                &fmt(item_y + LEGEND_BOX / 2.0),
                vars.font_family,
                &fmt(LEGEND_FONT),
                vars.text_color,
                &esc(&curve.label),
            ));
        }
        out.push_str("</g>");
    }

    out.push_str("</svg>");
    out
}

// ─── Math helpers ─────────────────────────────────────────────────────────────

/// Angle for axis i of n_axes, starting at -π/2 (top), going clockwise.
/// Mirrors D3's scalePoint().range([0, 2π]) starting at top.
fn axis_angle(i: usize, n: usize) -> f64 {
    if n == 0 {
        return -PI / 2.0;
    }
    -PI / 2.0 + 2.0 * PI * (i as f64 / n as f64)
}

/// Normalise value to a radius in [0, max_radius].
/// Port of relativeRadius() from renderer.ts.
fn relative_radius(value: f64, min_val: f64, max_val: f64, max_radius: f64) -> f64 {
    let range = max_val - min_val;
    if range <= 0.0 {
        return 0.0;
    }
    let clamped = value.max(min_val).min(max_val);
    ((clamped - min_val) / range) * max_radius
}

/// Generates a comma-separated SVG polygon points string for n-gon at (cx,cy) r=r.
fn polygon_points(cx: f64, cy: f64, r: f64, n: usize) -> String {
    (0..n)
        .map(|i| {
            let a = axis_angle(i, n);
            format!("{},{}", fmt(cx + r * a.cos()), fmt(cy + r * a.sin()))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Closed polygon path through points.
fn polygon_path(pts: &[(f64, f64)]) -> String {
    if pts.is_empty() {
        return String::new();
    }
    let mut d = format!("M{},{}", fmt(pts[0].0), fmt(pts[0].1));
    for (x, y) in &pts[1..] {
        d.push_str(&format!("L{},{}", fmt(*x), fmt(*y)));
    }
    d.push('Z');
    d
}

/// Closed Catmull-Rom spline through the points.
/// Port of closedRoundCurve() from renderer.ts — uses cubic Bézier approximation
/// with tension α=0.5 (standard Catmull-Rom).
fn closed_round_curve(pts: &[(f64, f64)]) -> String {
    let n = pts.len();
    if n < 2 {
        return String::new();
    }

    let mut d = String::new();
    d.push_str(&format!("M{},{}", fmt(pts[0].0), fmt(pts[0].1)));

    for i in 0..n {
        let p0 = pts[(i + n - 1) % n];
        let p1 = pts[i];
        let p2 = pts[(i + 1) % n];
        let p3 = pts[(i + 2) % n];

        // Compute control points from Catmull-Rom formula
        let cp1x = p1.0 + (p2.0 - p0.0) * CATMULL_ROM_ALPHA / 3.0;
        let cp1y = p1.1 + (p2.1 - p0.1) * CATMULL_ROM_ALPHA / 3.0;
        let cp2x = p2.0 - (p3.0 - p1.0) * CATMULL_ROM_ALPHA / 3.0;
        let cp2y = p2.1 - (p3.1 - p1.1) * CATMULL_ROM_ALPHA / 3.0;

        d.push_str(&format!(
            "C{},{} {},{} {},{}",
            fmt(cp1x),
            fmt(cp1y),
            fmt(cp2x),
            fmt(cp2y),
            fmt(p2.0),
            fmt(p2.1),
        ));
    }
    d.push('Z');
    d
}

// ─── Utility ──────────────────────────────────────────────────────────────────

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

    const RADAR_BASIC: &str = "radar-beta\n    title Skills\n    axis A[\"Coding\"], B[\"Design\"], C[\"Communication\"], D[\"Testing\"]\n    curve Team1 { A: 80, B: 60, C: 70, D: 85 }\n    curve Team2 { A: 70, B: 80, C: 65, D: 75 }";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(RADAR_BASIC).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(RADAR_BASIC).diagram;
        let svg = render(&diag, Theme::Dark);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn snapshot_default_theme() {
        let diag = parser::parse(RADAR_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(svg);
    }
}
