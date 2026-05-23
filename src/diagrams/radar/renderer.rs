// Faithful Rust port of mermaid/src/diagrams/radar/renderer.ts
//
// Architecture mirrors Mermaid JS exactly:
//   drawFrame   – sets viewBox, creates a centered <g transform="translate(cx, cy)">
//   drawGraticule – concentric circles or polygons (no cx/cy needed — g is centred)
//   drawAxes    – radial spokes + axis labels at axisLabelFactor * radius
//   drawCurves  – data polygons or Catmull-Rom paths, classed radarCurve-N
//   drawLegend  – per-curve <g transform="translate(...)"> with rect + text
//   title       – <text class="radarTitle"> at y = -(height/2 + marginTop)
//
// Key differences from the old renderer fixed here:
//   1. Square viewBox (total = chart + margins), chart centered via g transform
//   2. CSS classes used for all styling (no inline stroke/fill)
//   3. Catmull-Rom tension used directly, not divided by 3
//   4. axisScaleFactor / axisLabelFactor respected
//   5. Legend coordinates match Mermaid JS formula exactly
//   6. No data-point circles (Mermaid JS does not render them)
//   7. HSL-based curve colours from theme cScale variables

use super::constants::*;
use super::parser::{GraticuleType, RadarDiagram};
use super::templates::{self, centered_group_open, esc, fmt};
use crate::theme::Theme;
use std::f64::consts::PI;

pub fn render(diag: &RadarDiagram, theme: Theme) -> String {
    let vars = theme.resolve();

    // ── Dimensions (mirror Mermaid JS drawFrame) ───────────────────────────
    // config.width = CHART_WIDTH, config.height = CHART_HEIGHT
    // totalWidth  = config.width  + config.marginLeft + config.marginRight
    // totalHeight = config.height + config.marginTop  + config.marginBottom
    let total_w = SVG_WIDTH; // 700
    let total_h = SVG_HEIGHT; // 700

    // Center point in absolute SVG coords
    let cx = MARGIN_LEFT + CHART_WIDTH / 2.0; // 50 + 300 = 350
    let cy = MARGIN_TOP + CHART_HEIGHT / 2.0; // 50 + 300 = 350

    // Radius = min(width, height) / 2
    let radius = (CHART_WIDTH.min(CHART_HEIGHT) / 2.0).max(1.0); // 300

    let n_axes = diag.axes.len();

    // ── max / min values ───────────────────────────────────────────────────
    let data_max = diag
        .curves
        .iter()
        .flat_map(|c| c.entries.iter().copied())
        .fold(f64::NEG_INFINITY, f64::max);
    let max_val = diag.options.max.unwrap_or(if data_max > f64::NEG_INFINITY {
        data_max
    } else {
        1.0
    });
    let min_val = diag.options.min;
    let ticks = diag.options.ticks.max(1);

    // ── Curve colors ───────────────────────────────────────────────────────
    let c_scale = theme_c_scale(theme);

    // ── SVG root ───────────────────────────────────────────────────────────
    let mut out = String::new();
    out.push_str(&templates::svg_root(&fmt(total_w), &fmt(total_h)));

    // ── Centered group (all drawing relative to chart centre) ──────────────
    out.push_str(&centered_group_open(&fmt(cx), &fmt(cy)));

    // ── drawGraticule ──────────────────────────────────────────────────────
    // Mermaid JS: for i in 0..ticks => r = radius * (i+1) / ticks
    // Graticule rings use a neutral grey (#DEDEDE), not the theme line_color.
    // This matches Mermaid's radarGraticuleColor which is fixed across themes.
    let graticule_color = GRATICULE_COLOR;

    for i in 0..ticks {
        let r = radius * (i as f64 + 1.0) / ticks as f64;
        if diag.options.graticule == GraticuleType::Circle {
            out.push_str(&templates::graticule_circle(
                &fmt(r),
                graticule_color,
                &fmt(GRATICULE_OPACITY),
                &fmt(GRATICULE_STROKE_WIDTH),
            ));
        } else if n_axes >= 3 {
            let pts = polygon_points(r, n_axes);
            out.push_str(&templates::graticule_polygon(
                &pts,
                graticule_color,
                &fmt(GRATICULE_OPACITY),
                &fmt(GRATICULE_STROKE_WIDTH),
            ));
        }
    }

    // ── drawAxes ───────────────────────────────────────────────────────────
    for (i, axis) in diag.axes.iter().enumerate() {
        let angle = axis_angle(i, n_axes);
        // Spoke end at axisScaleFactor * radius
        let spoke_x = diag.options.axis_scale_factor * radius * angle.cos();
        let spoke_y = diag.options.axis_scale_factor * radius * angle.sin();
        out.push_str(&templates::axis_line(
            &fmt(spoke_x),
            &fmt(spoke_y),
            AXIS_LINE_COLOR,
        ));

        // Label at axisLabelFactor * radius
        let label_x = diag.options.axis_label_factor * radius * angle.cos();
        let label_y = diag.options.axis_label_factor * radius * angle.sin();
        out.push_str(&templates::axis_label(
            &fmt(label_x),
            &fmt(label_y),
            &esc(&axis.label),
            vars.text_color,
            &fmt(AXIS_LABEL_FONT_SIZE),
        ));
    }

    // ── drawCurves ─────────────────────────────────────────────────────────
    // Mermaid JS skips curves where entries.length != numAxes
    for (ci, curve) in diag.curves.iter().enumerate() {
        if curve.entries.len() != n_axes {
            continue;
        }

        let color = c_scale[ci % c_scale.len()];

        // Points relative to centre (the g transform moves us there)
        let points: Vec<(f64, f64)> = curve
            .entries
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let angle = axis_angle(i, n_axes);
                let r = relative_radius(v, min_val, max_val, radius);
                (r * angle.cos(), r * angle.sin())
            })
            .collect();

        if diag.options.graticule == GraticuleType::Circle {
            let d = closed_round_curve(&points, CURVE_TENSION);
            out.push_str(&templates::curve_path(
                &d,
                ci,
                color,
                CURVE_OPACITY,
                CURVE_STROKE_WIDTH,
            ));
        } else {
            let pts = points
                .iter()
                .map(|(x, y)| format!("{},{}", fmt(*x), fmt(*y)))
                .collect::<Vec<_>>()
                .join(" ");
            out.push_str(&templates::curve_polygon(
                &pts,
                ci,
                color,
                CURVE_OPACITY,
                CURVE_STROKE_WIDTH,
            ));
        }
    }

    // ── drawLegend ─────────────────────────────────────────────────────────
    // Mermaid JS:
    //   legendX = (width/2 + marginRight) * 3/4
    //   legendY = -(height/2 + marginTop) * 3/4
    //   lineHeight = 20
    if diag.options.show_legend && !diag.curves.is_empty() {
        let legend_x = (CHART_WIDTH / 2.0 + MARGIN_RIGHT) * 0.75;
        let legend_y = -(CHART_HEIGHT / 2.0 + MARGIN_TOP) * 0.75;
        for (ci, curve) in diag.curves.iter().enumerate() {
            let color = c_scale[ci % c_scale.len()];
            let item_y = legend_y + ci as f64 * LEGEND_LINE_HEIGHT;
            out.push_str(&templates::legend_group_open(&fmt(legend_x), &fmt(item_y)));
            out.push_str(&templates::legend_rect(ci, color, CURVE_OPACITY));
            out.push_str(&templates::legend_label(
                &esc(&curve.label),
                &fmt(LEGEND_FONT_SIZE),
                vars.text_color,
            ));
            out.push_str("</g>");
        }
    }

    // ── Title ──────────────────────────────────────────────────────────────
    // Mermaid JS: y = -(config.height / 2 + config.marginTop)
    if let Some(t) = &diag.title {
        let title_y = -(CHART_HEIGHT / 2.0 + MARGIN_TOP);
        out.push_str(&templates::title_text(
            &fmt(title_y),
            &esc(t),
            vars.title_color,
            "16",
        ));
    }

    // Close the centred group and outer wrapper group
    out.push_str("</g></g>");
    out.push_str("</svg>");
    out
}

// ─── Curve colour generation ──────────────────────────────────────────────────

/// cScale palette for the default and dark themes (12 entries).
/// Values taken from Mermaid JS theme variable output in the reference SVG:
///   cScale0  = hsl(240,100%,76.27%)  (blue-ish)
///   cScale1  = hsl(60,100%,73.53%)   (yellow)
///   etc.
fn theme_c_scale(theme: Theme) -> &'static [&'static str] {
    match theme {
        Theme::Dark => &[
            "#1f2020", "#0b0000", "#4d1037", "#3f5258", "#4f2f1b", "#6e0a0a", "#3b0048", "#995a01",
            "#154706", "#161722", "#00296f", "#01629c",
        ],
        Theme::Forest => &[
            "hsl(78.1578947368, 58.4615384615%, 64.5098039216%)",
            "hsl(98.961038961, 100%, 74.9019607843%)",
            "hsl(78.1578947368, 58.4615384615%, 74.5098039216%)",
            "hsl(108.1578947368, 58.4615384615%, 64.5098039216%)",
            "hsl(138.1578947368, 58.4615384615%, 74.5098039216%)",
            "hsl(168.1578947368, 58.4615384615%, 74.5098039216%)",
            "hsl(198.1578947368, 58.4615384615%, 74.5098039216%)",
            "hsl(228.1578947368, 58.4615384615%, 74.5098039216%)",
            "hsl(288.1578947368, 58.4615384615%, 74.5098039216%)",
            "hsl(348.1578947368, 58.4615384615%, 74.5098039216%)",
            "hsl(18.1578947368, 58.4615384615%, 74.5098039216%)",
            "hsl(48.1578947368, 58.4615384615%, 74.5098039216%)",
        ],
        Theme::Neutral => &[
            "#555", "#F4F4F4", "#555", "#BBB", "#999", "#777", "#AAA", "#888", "#666", "#CCC",
            "#444", "#DDD",
        ],
        _ => &[
            "hsl(240,100%,76.2745098039%)",
            "hsl(60,100%,73.5294117647%)",
            "hsl(80,100%,76.2745098039%)",
            "hsl(270,100%,76.2745098039%)",
            "hsl(300,100%,76.2745098039%)",
            "hsl(330,100%,76.2745098039%)",
            "hsl(0,100%,76.2745098039%)",
            "hsl(30,100%,76.2745098039%)",
            "hsl(90,100%,76.2745098039%)",
            "hsl(150,100%,76.2745098039%)",
            "hsl(180,100%,76.2745098039%)",
            "hsl(210,100%,76.2745098039%)",
        ],
    }
}

// ─── Math helpers ─────────────────────────────────────────────────────────────

/// Angle for axis i of n_axes, starting at -π/2 (top), going clockwise.
/// Mirrors Mermaid JS: angle = 2 * i * PI / numAxes - PI/2
fn axis_angle(i: usize, n: usize) -> f64 {
    if n == 0 {
        return -PI / 2.0;
    }
    2.0 * PI * (i as f64) / (n as f64) - PI / 2.0
}

/// Normalise value to a radius in [0, max_radius].
/// Port of relativeRadius() from renderer.ts.
fn relative_radius(value: f64, min_val: f64, max_val: f64, max_radius: f64) -> f64 {
    let range = max_val - min_val;
    if range <= 0.0 {
        return 0.0;
    }
    let clamped = value.max(min_val).min(max_val);
    max_radius * (clamped - min_val) / range
}

/// Generates a space-separated SVG polygon points string for n-gon, centred at origin.
fn polygon_points(r: f64, n: usize) -> String {
    (0..n)
        .map(|i| {
            let a = axis_angle(i, n);
            format!("{},{}", fmt(r * a.cos()), fmt(r * a.sin()))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Closed Catmull-Rom spline through the points.
/// Faithful port of closedRoundCurve() from Mermaid JS renderer.ts.
/// NOTE: tension is used directly (NOT divided by 3 — that was our previous bug).
fn closed_round_curve(pts: &[(f64, f64)], tension: f64) -> String {
    let n = pts.len();
    if n < 2 {
        return String::new();
    }

    // M to first point
    let mut d = format!("M{},{}", fmt(pts[0].0), fmt(pts[0].1));

    for i in 0..n {
        let p0 = pts[(i + n - 1) % n];
        let p1 = pts[i];
        let p2 = pts[(i + 1) % n];
        let p3 = pts[(i + 2) % n];

        // Control points — Mermaid JS formula (tension, not tension/3)
        let cp1x = p1.0 + (p2.0 - p0.0) * tension;
        let cp1y = p1.1 + (p2.1 - p0.1) * tension;
        let cp2x = p2.0 - (p3.0 - p1.0) * tension;
        let cp2y = p2.1 - (p3.1 - p1.1) * tension;

        d.push_str(&format!(
            " C{},{} {},{} {},{}",
            fmt(cp1x),
            fmt(cp1y),
            fmt(cp2x),
            fmt(cp2y),
            fmt(p2.0),
            fmt(p2.1),
        ));
    }
    d.push_str(" Z");
    d
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    const RADAR_BASIC: &str = "radar-beta\n    title Skills\n    axis A[\"Coding\"], B[\"Design\"], C[\"Communication\"], D[\"Testing\"]\n    curve Team1 { A: 80, B: 60, C: 70, D: 85 }\n    curve Team2 { A: 70, B: 80, C: 65, D: 75 }";

    const RADAR_LIVE_EDITOR: &str = "radar-beta\n  axis m[\"Math\"], s[\"Science\"], e[\"English\"]\n  axis h[\"History\"], g[\"Geography\"], a[\"Art\"]\n  curve a[\"Alice\"]{85, 90, 80, 70, 75, 90}\n  curve b[\"Bob\"]{70, 75, 85, 80, 90, 85}\n\n  max 100\n  min 0";

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
    fn radar_has_correct_viewbox() {
        let diag = parser::parse(RADAR_BASIC).diagram;
        let svg = render(&diag, Theme::Default);
        // 700 = 600 chart + 50 marginLeft + 50 marginRight
        assert!(
            svg.contains("viewBox=\"0 0 700 700\""),
            "expected viewBox 700x700, got: {}",
            &svg[..svg.find('>').unwrap_or(200)]
        );
    }

    #[test]
    fn radar_has_centered_group() {
        let diag = parser::parse(RADAR_BASIC).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(
            svg.contains("translate(350, 350)"),
            "expected translate(350, 350) in: {}",
            &svg[..300.min(svg.len())]
        );
    }

    #[test]
    fn radar_uses_css_classes() {
        let diag = parser::parse(RADAR_BASIC).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(
            svg.contains("radarGraticule"),
            "missing radarGraticule class"
        );
        assert!(svg.contains("radarAxisLine"), "missing radarAxisLine class");
        assert!(
            svg.contains("radarAxisLabel"),
            "missing radarAxisLabel class"
        );
        assert!(svg.contains("radarCurve-0"), "missing radarCurve-0 class");
        assert!(
            svg.contains("radarLegendBox-0"),
            "missing radarLegendBox-0 class"
        );
        assert!(
            svg.contains("radarLegendText"),
            "missing radarLegendText class"
        );
    }

    #[test]
    fn radar_live_editor_has_title() {
        // The live editor input uses YAML frontmatter title — parser must strip it.
        let input = "---\ntitle: \"Grades\"\n---\nradar-beta\n  axis m[\"Math\"], s[\"Science\"], e[\"English\"]\n  axis h[\"History\"], g[\"Geography\"], a[\"Art\"]\n  curve a[\"Alice\"]{85, 90, 80, 70, 75, 90}\n  curve b[\"Bob\"]{70, 75, 85, 80, 90, 85}\n\n  max 100\n  min 0";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("radarTitle"), "missing radarTitle class");
    }

    #[test]
    fn snapshot_default_theme() {
        let diag = parser::parse(RADAR_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }

    #[test]
    fn snapshot_live_editor() {
        let diag = parser::parse(RADAR_LIVE_EDITOR).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
