use super::constants::*;
use super::parser::QuadrantDiagram;
use super::templates::{self, esc, fmt};
/// Faithful Rust port of Mermaid's quadrantBuilder.ts + quadrantRenderer.ts.
///
/// Layout algorithm is a direct translation of QuadrantBuilder.calculateSpace(),
/// getQuadrants(), getAxisLabels(), getQuadrantPoints(), getBorders(), getTitle().
///
/// Key behaviours (from quadrantBuilder.ts):
/// - When data points exist, x-axis is forced to "bottom" position.
/// - Quadrant label positioning: center when no points, top when points present.
/// - Axis labels: if both left+right x-axis texts are present, they are drawn at
///   quadrantHalfWidth/2 positions ("center" vertical alignment); otherwise at left.
/// - Y-axis labels are rotated -90 degrees.
/// - scaleLinear: x maps [0,1] -> [quadrantLeft, quadrantLeft+quadrantWidth]
///   y maps [0,1] -> [quadrantTop+quadrantHeight, quadrantTop] (inverted)
use crate::theme::Theme;

// ── Theme-resolved colour set ─────────────────────────────────────────────────

/// Colours resolved from a [`Theme`] for the quadrant diagram.
struct QuadrantColors {
    quadrant1_fill: &'static str,
    quadrant2_fill: &'static str,
    quadrant3_fill: &'static str,
    quadrant4_fill: &'static str,
    text_fill: &'static str,
    point_fill: &'static str,
    border_fill: &'static str,
}

// ── Internal layout data types ────────────────────────────────────────────────

struct SpaceData {
    #[allow(dead_code)]
    x_axis_space_top: f64,
    #[allow(dead_code)]
    x_axis_space_bottom: f64,
    #[allow(dead_code)]
    y_axis_space_left: f64,
    title_space_top: f64,
    quadrant_left: f64,
    quadrant_top: f64,
    quadrant_width: f64,
    quadrant_half_width: f64,
    quadrant_height: f64,
    quadrant_half_height: f64,
}

struct TextEl {
    text: String,
    x: f64,
    y: f64,
    fill: &'static str,
    font_size: f64,
    /// "left" | "center"
    vertical_pos: &'static str,
    /// "top" | "middle"
    horizontal_pos: &'static str,
    rotation: f64,
}

struct QuadrantEl {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    fill: &'static str,
    text: TextEl,
}

struct PointEl {
    x: f64,
    y: f64,
    radius: f64,
    fill: &'static str,
    stroke_color: &'static str,
    stroke_width: &'static str,
    text: TextEl,
}

struct LineEl {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    stroke_fill: &'static str,
    stroke_width: f64,
}

// ── calculate_space (port of QuadrantBuilder.calculateSpace) ──────────────────

fn calculate_space(
    x_axis_position: &str, // "top" | "bottom"
    show_x_axis: bool,
    show_y_axis: bool,
    show_title: bool,
) -> SpaceData {
    let x_axis_space_calc = X_AXIS_LABEL_PADDING * 2.0 + X_AXIS_LABEL_FONT_SIZE;
    let x_axis_space_top = if x_axis_position == "top" && show_x_axis {
        x_axis_space_calc
    } else {
        0.0
    };
    let x_axis_space_bottom = if x_axis_position == "bottom" && show_x_axis {
        x_axis_space_calc
    } else {
        0.0
    };

    let y_axis_space_calc = Y_AXIS_LABEL_PADDING * 2.0 + Y_AXIS_LABEL_FONT_SIZE;
    let y_axis_space_left = if show_y_axis { y_axis_space_calc } else { 0.0 };

    let title_space_calc = TITLE_FONT_SIZE + TITLE_PADDING * 2.0;
    let title_space_top = if show_title { title_space_calc } else { 0.0 };

    let quadrant_left = QUADRANT_PADDING + y_axis_space_left;
    let quadrant_top = QUADRANT_PADDING + x_axis_space_top + title_space_top;
    let quadrant_width = CHART_WIDTH - QUADRANT_PADDING * 2.0 - y_axis_space_left;
    let quadrant_height = CHART_HEIGHT
        - QUADRANT_PADDING * 2.0
        - x_axis_space_top
        - x_axis_space_bottom
        - title_space_top;
    let quadrant_half_width = quadrant_width / 2.0;
    let quadrant_half_height = quadrant_height / 2.0;

    SpaceData {
        x_axis_space_top,
        x_axis_space_bottom,
        y_axis_space_left,
        title_space_top,
        quadrant_left,
        quadrant_top,
        quadrant_width,
        quadrant_half_width,
        quadrant_height,
        quadrant_half_height,
    }
}

// ── get_quadrants (port of QuadrantBuilder.getQuadrants) ─────────────────────

fn get_quadrants(
    diag: &QuadrantDiagram,
    space: &SpaceData,
    colors: &QuadrantColors,
) -> Vec<QuadrantEl> {
    let has_points = !diag.points.is_empty();
    let ql = space.quadrant_left;
    let qt = space.quadrant_top;
    let qhw = space.quadrant_half_width;
    let qhh = space.quadrant_half_height;

    // quadrant-1: top-right, quadrant-2: top-left, quadrant-3: bottom-left, quadrant-4: bottom-right
    let defs = [
        (
            diag.quadrant1_text.as_str(),
            colors.quadrant1_fill,
            ql + qhw,
            qt,
        ),
        (diag.quadrant2_text.as_str(), colors.quadrant2_fill, ql, qt),
        (
            diag.quadrant3_text.as_str(),
            colors.quadrant3_fill,
            ql,
            qt + qhh,
        ),
        (
            diag.quadrant4_text.as_str(),
            colors.quadrant4_fill,
            ql + qhw,
            qt + qhh,
        ),
    ];

    defs.iter()
        .map(|(label, fill, x, y)| {
            let text_x = x + qhw / 2.0;
            let (text_y, horiz) = if has_points {
                (y + QUADRANT_TEXT_TOP_PADDING, "top")
            } else {
                (y + qhh / 2.0, "middle")
            };
            QuadrantEl {
                x: *x,
                y: *y,
                width: qhw,
                height: qhh,
                fill,
                text: TextEl {
                    text: (*label).to_string(),
                    x: text_x,
                    y: text_y,
                    fill: colors.text_fill,
                    font_size: QUADRANT_LABEL_FONT_SIZE,
                    vertical_pos: "center",
                    horizontal_pos: horiz,
                    rotation: 0.0,
                },
            }
        })
        .collect()
}

// ── get_axis_labels (port of QuadrantBuilder.getAxisLabels) ──────────────────

fn get_axis_labels(
    diag: &QuadrantDiagram,
    x_axis_position: &str,
    show_x_axis: bool,
    show_y_axis: bool,
    space: &SpaceData,
    colors: &QuadrantColors,
) -> Vec<TextEl> {
    let mut labels = Vec::new();
    let ql = space.quadrant_left;
    let qt = space.quadrant_top;
    let qhw = space.quadrant_half_width;
    let qhh = space.quadrant_half_height;
    let _qw = space.quadrant_width;
    let qh = space.quadrant_height;
    let ts_top = space.title_space_top;

    let draw_x_mid = !diag.x_axis_right_text.is_empty();
    let draw_y_mid = !diag.y_axis_top_text.is_empty();

    if !diag.x_axis_left_text.is_empty() && show_x_axis {
        let x = ql + if draw_x_mid { qhw / 2.0 } else { 0.0 };
        let y = if x_axis_position == "top" {
            X_AXIS_LABEL_PADDING + ts_top
        } else {
            X_AXIS_LABEL_PADDING + qt + qh + QUADRANT_PADDING
        };
        labels.push(TextEl {
            text: diag.x_axis_left_text.clone(),
            x,
            y,
            fill: colors.text_fill,
            font_size: X_AXIS_LABEL_FONT_SIZE,
            vertical_pos: if draw_x_mid { "center" } else { "left" },
            horizontal_pos: "top",
            rotation: 0.0,
        });
    }

    if !diag.x_axis_right_text.is_empty() && show_x_axis {
        let x = ql + qhw + if draw_x_mid { qhw / 2.0 } else { 0.0 };
        let y = if x_axis_position == "top" {
            X_AXIS_LABEL_PADDING + ts_top
        } else {
            X_AXIS_LABEL_PADDING + qt + qh + QUADRANT_PADDING
        };
        labels.push(TextEl {
            text: diag.x_axis_right_text.clone(),
            x,
            y,
            fill: colors.text_fill,
            font_size: X_AXIS_LABEL_FONT_SIZE,
            vertical_pos: if draw_x_mid { "center" } else { "left" },
            horizontal_pos: "top",
            rotation: 0.0,
        });
    }

    if !diag.y_axis_bottom_text.is_empty() && show_y_axis {
        let x = Y_AXIS_LABEL_PADDING;
        let y = qt + qh - if draw_y_mid { qhh / 2.0 } else { 0.0 };
        labels.push(TextEl {
            text: diag.y_axis_bottom_text.clone(),
            x,
            y,
            fill: colors.text_fill,
            font_size: Y_AXIS_LABEL_FONT_SIZE,
            vertical_pos: if draw_y_mid { "center" } else { "left" },
            horizontal_pos: "top",
            rotation: -90.0,
        });
    }

    if !diag.y_axis_top_text.is_empty() && show_y_axis {
        let x = Y_AXIS_LABEL_PADDING;
        let y = qt + qhh - if draw_y_mid { qhh / 2.0 } else { 0.0 };
        labels.push(TextEl {
            text: diag.y_axis_top_text.clone(),
            x,
            y,
            fill: colors.text_fill,
            font_size: Y_AXIS_LABEL_FONT_SIZE,
            vertical_pos: if draw_y_mid { "center" } else { "left" },
            horizontal_pos: "top",
            rotation: -90.0,
        });
    }

    labels
}

// ── get_quadrant_points (port of QuadrantBuilder.getQuadrantPoints) ───────────

fn get_quadrant_points(
    diag: &QuadrantDiagram,
    space: &SpaceData,
    colors: &QuadrantColors,
) -> Vec<PointEl> {
    let ql = space.quadrant_left;
    let qt = space.quadrant_top;
    let qw = space.quadrant_width;
    let qh = space.quadrant_height;

    let scale_x = |v: f64| ql + v * qw;
    let scale_y = |v: f64| (qh + qt) + v * (qt - (qh + qt));

    diag.points
        .iter()
        .map(|p| {
            let px = scale_x(p.x);
            let py = scale_y(p.y);
            PointEl {
                x: px,
                y: py,
                radius: POINT_RADIUS,
                fill: colors.point_fill,
                stroke_color: colors.point_fill,
                stroke_width: "0px",
                text: TextEl {
                    text: p.text.clone(),
                    x: px,
                    y: py + POINT_TEXT_PADDING,
                    fill: colors.text_fill,
                    font_size: POINT_LABEL_FONT_SIZE,
                    vertical_pos: "center",
                    horizontal_pos: "top",
                    rotation: 0.0,
                },
            }
        })
        .collect()
}

// ── get_borders (port of QuadrantBuilder.getBorders) ─────────────────────────

fn get_borders(space: &SpaceData, colors: &QuadrantColors) -> Vec<LineEl> {
    let hw = QUADRANT_EXTERNAL_BORDER_STROKE_WIDTH / 2.0;
    let ql = space.quadrant_left;
    let qt = space.quadrant_top;
    let qw = space.quadrant_width;
    let qhw = space.quadrant_half_width;
    let qh = space.quadrant_height;
    let qhh = space.quadrant_half_height;

    vec![
        LineEl {
            x1: ql - hw,
            y1: qt,
            x2: ql + qw + hw,
            y2: qt,
            stroke_fill: colors.border_fill,
            stroke_width: QUADRANT_EXTERNAL_BORDER_STROKE_WIDTH,
        },
        LineEl {
            x1: ql + qw,
            y1: qt + hw,
            x2: ql + qw,
            y2: qt + qh - hw,
            stroke_fill: colors.border_fill,
            stroke_width: QUADRANT_EXTERNAL_BORDER_STROKE_WIDTH,
        },
        LineEl {
            x1: ql - hw,
            y1: qt + qh,
            x2: ql + qw + hw,
            y2: qt + qh,
            stroke_fill: colors.border_fill,
            stroke_width: QUADRANT_EXTERNAL_BORDER_STROKE_WIDTH,
        },
        LineEl {
            x1: ql,
            y1: qt + hw,
            x2: ql,
            y2: qt + qh - hw,
            stroke_fill: colors.border_fill,
            stroke_width: QUADRANT_EXTERNAL_BORDER_STROKE_WIDTH,
        },
        LineEl {
            x1: ql + qhw,
            y1: qt + hw,
            x2: ql + qhw,
            y2: qt + qh - hw,
            stroke_fill: colors.border_fill,
            stroke_width: QUADRANT_INTERNAL_BORDER_STROKE_WIDTH,
        },
        LineEl {
            x1: ql + hw,
            y1: qt + qhh,
            x2: ql + qw - hw,
            y2: qt + qhh,
            stroke_fill: colors.border_fill,
            stroke_width: QUADRANT_INTERNAL_BORDER_STROKE_WIDTH,
        },
    ]
}

// ── get_title (port of QuadrantBuilder.getTitle) ──────────────────────────────

fn get_title(diag: &QuadrantDiagram, show_title: bool, colors: &QuadrantColors) -> Option<TextEl> {
    if show_title {
        Some(TextEl {
            text: diag.title.clone(),
            x: CHART_WIDTH / 2.0,
            y: TITLE_PADDING,
            fill: colors.text_fill,
            font_size: TITLE_FONT_SIZE,
            vertical_pos: "center",
            horizontal_pos: "top",
            rotation: 0.0,
        })
    } else {
        None
    }
}

// ── SVG rendering helpers ─────────────────────────────────────────────────────

fn dominant_baseline(horiz: &str) -> &'static str {
    if horiz == "top" {
        "hanging"
    } else {
        "middle"
    }
}

fn text_anchor(vert: &str) -> &'static str {
    if vert == "left" {
        "start"
    } else {
        "middle"
    }
}

fn render_text_el(el: &TextEl) -> String {
    let transform = if el.rotation != 0.0 {
        format!(
            "translate({}, {}) rotate({})",
            fmt(el.x),
            fmt(el.y),
            fmt(el.rotation)
        )
    } else {
        format!("translate({}, {})", fmt(el.x), fmt(el.y))
    };
    templates::text_el(
        el.fill,
        &fmt(el.font_size),
        dominant_baseline(el.horizontal_pos),
        text_anchor(el.vertical_pos),
        &transform,
        &esc(&el.text),
    )
}

// ── Main render entry point ───────────────────────────────────────────────────

pub fn render(diag: &QuadrantDiagram, theme: Theme) -> String {
    let vars = theme.resolve();

    // Quadrant fills: per-theme base colours matching Mermaid's quadrant cScale.
    // Each subsequent quadrant lightens the base by +5 per RGB channel.
    let (q1, q2, q3, q4) = match theme {
        crate::theme::Theme::Dark => ("#1f2020", "#242525", "#292a2a", "#2e2f2f"),
        crate::theme::Theme::Forest => ("#cde498", "#d2e99d", "#d7eea2", "#dcf3a7"),
        crate::theme::Theme::Neutral => ("#eeeeee", "#f3f3f3", "#f8f8f8", "#fdfdfd"),
        _ => ("#ECECFF", "#f1f1ff", "#f6f6ff", "#fbfbff"),
    };
    let colors = QuadrantColors {
        quadrant1_fill: q1,
        quadrant2_fill: q2,
        quadrant3_fill: q3,
        quadrant4_fill: q4,
        text_fill: vars.text_color,
        point_fill: vars.text_color,
        border_fill: vars.line_color,
    };

    let show_x_axis = !diag.x_axis_left_text.is_empty() || !diag.x_axis_right_text.is_empty();
    let show_y_axis = !diag.y_axis_top_text.is_empty() || !diag.y_axis_bottom_text.is_empty();
    let show_title = !diag.title.is_empty();

    let x_axis_position = if !diag.points.is_empty() {
        "bottom"
    } else {
        "top"
    };

    let space = calculate_space(x_axis_position, show_x_axis, show_y_axis, show_title);

    let quadrants = get_quadrants(diag, &space, &colors);
    let points = get_quadrant_points(diag, &space, &colors);
    let axis_labels = get_axis_labels(
        diag,
        x_axis_position,
        show_x_axis,
        show_y_axis,
        &space,
        &colors,
    );
    let borders = get_borders(&space, &colors);
    let title = get_title(diag, show_title, &colors);

    let id = "mermaid-quadrant";
    let width = CHART_WIDTH;
    let height = CHART_HEIGHT;

    let mut out = Vec::<String>::new();

    out.push(templates::svg_root(
        id,
        &fmt(width),
        &fmt(height),
        vars.font_family,
    ));
    out.push(r#"<g class="main">"#.to_string());

    out.push(r#"<g class="quadrants">"#.to_string());
    for q in &quadrants {
        out.push(templates::quadrant_group(
            &fmt(q.x),
            &fmt(q.y),
            &fmt(q.width),
            &fmt(q.height),
            q.fill,
            &render_text_el(&q.text),
        ));
    }
    out.push("</g>".to_string());

    out.push(r#"<g class="border">"#.to_string());
    for b in &borders {
        out.push(templates::border_line(
            &fmt(b.x1),
            &fmt(b.y1),
            &fmt(b.x2),
            &fmt(b.y2),
            b.stroke_fill,
            &fmt(b.stroke_width),
        ));
    }
    out.push("</g>".to_string());

    out.push(r#"<g class="data-points">"#.to_string());
    for p in &points {
        out.push(templates::data_point_group(
            &fmt(p.x),
            &fmt(p.y),
            &fmt(p.radius),
            p.fill,
            p.stroke_color,
            p.stroke_width,
            &render_text_el(&p.text),
        ));
    }
    out.push("</g>".to_string());

    out.push(r#"<g class="labels">"#.to_string());
    for l in &axis_labels {
        out.push(templates::label_group(&render_text_el(l)));
    }
    out.push("</g>".to_string());

    out.push(r#"<g class="title">"#.to_string());
    if let Some(t) = &title {
        out.push(render_text_el(t));
    }
    out.push("</g>".to_string());

    out.push("</g>".to_string()); // close .main
    out.push("</svg>".to_string());

    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::quadrant::parser;

    fn sample_diagram() -> QuadrantDiagram {
        parser::parse(
            r#"quadrantChart
    title Reach and engagement of campaigns
    x-axis Influence --> High Influence
    y-axis Low Reach --> High Reach
    quadrant-1 We should expand
    quadrant-2 Need to promote
    quadrant-3 Re-evaluate
    quadrant-4 May be improved
    Campaign A: [0.3, 0.6]
    Campaign B: [0.45, 0.23]"#,
        )
        .diagram
    }

    #[test]
    fn produces_svg() {
        let diag = sample_diagram();
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "expected <svg tag");
        assert!(svg.contains("quadrantChart"), "expected aria description");
        assert!(svg.contains("<rect"), "expected quadrant rects");
        assert!(svg.contains("<circle"), "expected data point circles");
        assert!(svg.contains("Campaign A"), "expected point label");
        assert!(svg.contains("Campaign B"), "expected point label");
    }

    #[test]
    fn title_renders() {
        let diag = sample_diagram();
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("Reach and engagement"), "expected title");
    }

    #[test]
    fn axis_labels_render() {
        let diag = sample_diagram();
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("Influence"), "expected x-axis label");
        assert!(
            svg.contains("High Influence"),
            "expected x-axis right label"
        );
        assert!(svg.contains("Low Reach"), "expected y-axis label");
        assert!(svg.contains("High Reach"), "expected y-axis top label");
    }

    #[test]
    fn quadrant_labels_render() {
        let diag = sample_diagram();
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("We should expand"));
        assert!(svg.contains("Need to promote"));
        assert!(svg.contains("Re-evaluate"));
        assert!(svg.contains("May be improved"));
    }

    #[test]
    fn no_points_no_circles() {
        let diag = parser::parse("quadrantChart\n    quadrant-1 Q1\n    quadrant-2 Q2").diagram;
        let svg = render(&diag, Theme::Default);
        assert!(!svg.contains("<circle"), "no circles without points");
    }

    #[test]
    fn six_border_lines() {
        let diag = sample_diagram();
        let svg = render(&diag, Theme::Default);
        let count = svg.matches("<line").count();
        assert_eq!(count, 6, "expected 6 border lines, got {count}");
    }

    #[test]
    fn x_axis_bottom_when_points() {
        let diag = sample_diagram();
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("Influence"));
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "quadrantChart\n    title Reach and engagement of campaigns\n    x-axis Low Reach --> High Reach\n    y-axis Low Engagement --> High Engagement\n    quadrant-1 We should expand\n    quadrant-2 Need to promote\n    quadrant-3 Re-evaluate\n    quadrant-4 May be improved\n    Campaign A: [0.3, 0.6]\n    Campaign B: [0.45, 0.23]\n    Campaign C: [0.57, 0.69]\n    Campaign D: [0.78, 0.34]\n    Campaign E: [0.40, 0.34]\n    Campaign F: [0.35, 0.78]";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
