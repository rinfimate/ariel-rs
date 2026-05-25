use super::constants::*;
use super::parser::PieDiagram;
use super::templates::{self, esc, fmt, fmt_value};
use crate::text_browser_metrics::measure_browser;
use crate::theme::Theme;
/// Faithful Rust port of Mermaid's pieRenderer.ts.
///
/// Key algorithm details:
/// - D3 pie layout: arcs are sorted by insertion order (sort: null), starting at -π/2 (top)
/// - Arc path uses SVG arc command with large-arc-flag based on sweep > π
/// - Label position at textPosition=0.75 of radius (centroid of arc at that radius)
/// - Legend: 12*LEGEND_RECT_SIZE horizontal offset, vertically centered
/// - viewBox width = max(chartAndLegendWidth, titleRight) where title is centered at pieWidth/2
///
/// Colors from Mermaid default theme (khroma adjust on #ECECFF primary):
///   pie1  = primaryColor    (theme-dependent, e.g. #ECECFF default / #1f2020 dark)
///   pie2  = secondaryColor  (theme-dependent, e.g. #ffffde default / #323232 dark)
///   pie3  = hsl(80 ,100%,56.27%)   tertiaryColor(h-160)+l-40
///   pie4  = hsl(240,100%,86.27%)   primary+l-10
///   pie5  = hsl(60 ,100%,63.53%)   secondary+l-30
///   pie6  = hsl(80 ,100%,76.27%)   tertiary+l-20
///   pie7  = hsl(300,100%,76.27%)   primary+h+60+l-20
///   pie8  = hsl(180,100%,56.27%)   primary+h-60+l-40
///   pie9  = hsl(0  ,100%,56.27%)   primary+h+120+l-40
///   pie10 = hsl(300,100%,56.27%)   primary+h+60+l-40
///   pie11 = hsl(150,100%,56.27%)   primary+h-90+l-40
///   pie12 = hsl(0  ,100%,66.27%)   primary+h+120+l-30
use std::f64::consts::PI;

/// One arc segment ready for rendering.
struct ArcDatum {
    label: String,
    value: f64,
    start_angle: f64, // radians, measured from top (−π/2)
    end_angle: f64,
}

/// Build D3-style pie arcs from the section map.
/// D3 pie with sort=null starts at top (−π/2) and proceeds clockwise.
fn create_pie_arcs(sections: &indexmap::IndexMap<String, f64>) -> Vec<ArcDatum> {
    let sum: f64 = sections.values().sum();
    if sum == 0.0 {
        return Vec::new();
    }

    // Filter out slices that would be < 1% (same as Mermaid's createPieArcs)
    let filtered: Vec<(&String, &f64)> = sections
        .iter()
        .filter(|(_, v)| ((*v) / sum) * 100.0 >= 1.0)
        .collect();

    let filtered_sum: f64 = filtered.iter().map(|(_, v)| **v).sum();

    let mut arcs = Vec::new();
    let mut current_angle = 0.0_f64; // D3 pie startAngle=0 = top of circle

    for (label, value) in &filtered {
        let fraction = **value / filtered_sum;
        let sweep = fraction * 2.0 * PI;
        let start = current_angle;
        let end = current_angle + sweep;
        arcs.push(ArcDatum {
            label: (*label).clone(),
            value: **value,
            start_angle: start,
            end_angle: end,
        });
        current_angle = end;
    }

    arcs
}

/// Compute the SVG arc path for a pie slice (innerRadius=0, outerRadius=radius).
/// Matches D3's arc generator exactly.
fn arc_path(start_angle: f64, end_angle: f64, radius: f64) -> String {
    let x0 = start_angle.sin() * radius;
    let y0 = -start_angle.cos() * radius; // SVG y-axis is flipped vs math
    let x1 = end_angle.sin() * radius;
    let y1 = -end_angle.cos() * radius;

    let sweep = end_angle - start_angle;
    let large_arc = if sweep > PI { 1 } else { 0 };

    // D3 arc: M start_x,start_y  A r,r,0,large,1,end_x,end_y  L 0,0  Z
    format!(
        "M{},{:.3}A{},{},0,{},1,{},{:.3}L0,0Z",
        fmt(x0),
        y0,
        fmt(radius),
        fmt(radius),
        large_arc,
        fmt(x1),
        y1,
    )
}

/// Compute the centroid point of an arc at a given radius (for label placement).
/// D3: centroid = midAngle, at given radius.
fn arc_centroid(start_angle: f64, end_angle: f64, radius: f64) -> (f64, f64) {
    let mid = (start_angle + end_angle) / 2.0;
    let x = mid.sin() * radius;
    let y = -mid.cos() * radius;
    (x, y)
}

/// Pick a color for a slice by index (wraps around after 12).
///
/// Indices 0 and 1 map to theme-dependent primary/secondary colors and are
/// returned as borrowed `&'static str` only for the static entries (indices 2+).
/// Callers must supply `primary_color` and `secondary_color` from theme vars
/// and handle the first two indices separately via `pie_slice_color`.
const DARK_PIE_COLORS: &[&str] = &[
    "#0b0000", "#4d1037", "#3f5258", "#4f2f1b", "#6e0a0a", "#3b0048", "#995a01", "#154706",
    "#161722", "#00296f", "#01629c", "#1f2020",
];
// Forest pie3..12 — values confirmed from reference SVGs.
const FOREST_PIE_COLORS_STATIC: &[&str] = &[
    "hsl(78.1578947368, 58.4615384615%, 84.5098039216%)", // pie3: primary hue +10% light
    "hsl(78.1578947368, 58.4615384615%, 44.5098039216%)", // pie4: primary hue -30% light
    "hsl(98.961038961, 100%, 54.9019607843%)",            // pie5: secondary hue -30% light
    "hsl(118.1578947368, 58.4615384615%, 44.5098039216%)", // pie6: +40° hue -30% light
    "hsl(138.1578947368, 58.4615384615%, 64.5098039216%)", // pie7: +60° hue -10% light
    "hsl(158.1578947368, 58.4615384615%, 44.5098039216%)", // pie8
    "hsl(178.1578947368, 58.4615384615%, 64.5098039216%)", // pie9
    "hsl(198.1578947368, 58.4615384615%, 44.5098039216%)", // pie10
    "hsl(218.1578947368, 58.4615384615%, 64.5098039216%)", // pie11
    "hsl(238.1578947368, 58.4615384615%, 54.5098039216%)", // pie12
];
const NEUTRAL_PIE_COLORS: &[&str] = &[
    "#F4F4F4", "#555", "#BBB", "#777", "#999", "#DDD", "#FFF", "#DDD", "#BBB", "#999", "#777",
    "#555",
];

fn pie_slice_color<'a>(
    index: usize,
    primary: &'a str,
    secondary: &'a str,
    theme: Theme,
) -> &'a str {
    match theme {
        Theme::Dark => DARK_PIE_COLORS[index % DARK_PIE_COLORS.len()],
        Theme::Forest => match index % 12 {
            0 => primary,
            1 => secondary,
            i => FOREST_PIE_COLORS_STATIC[(i - 2) % FOREST_PIE_COLORS_STATIC.len()],
        },
        Theme::Neutral => NEUTRAL_PIE_COLORS[index % NEUTRAL_PIE_COLORS.len()],
        _ => match index % 12 {
            0 => primary,
            1 => secondary,
            i => PIE_COLORS_STATIC[(i - 2) % PIE_COLORS_STATIC.len()],
        },
    }
}

#[allow(clippy::vec_init_then_push)]
pub fn render(diag: &PieDiagram, theme: Theme, _use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    let primary_text = vars.primary_text;
    let primary_color = vars.primary_color;
    let secondary_color = vars.secondary_color;
    let radius = (PIE_WIDTH.min(HEIGHT) / 2.0) - MARGIN; // = 185.0
    let label_radius = radius * TEXT_POSITION; // = 138.75

    let arcs = create_pie_arcs(&diag.sections);

    // Sum of all sections (for percentage display)
    let total_sum: f64 = diag.sections.values().sum();

    // Filter arcs that would display as "0%" (toFixed(0) == "0")
    let filtered_arcs: Vec<&ArcDatum> = arcs
        .iter()
        .filter(|a| (a.value / total_sum * 100.0).round() as u64 != 0)
        .collect();

    // ── Collect all section data for legend (all sections, not filtered) ──────
    let all_sections: Vec<(&String, &f64)> = diag.sections.iter().collect();

    // ── Measure legend text widths ─────────────────────────────────────────────
    let legend_text_width = all_sections
        .iter()
        .map(|(label, value)| {
            let text = if diag.show_data {
                format!("{} [{}]", label, fmt_value(**value))
            } else {
                (*label).clone()
            };
            measure_browser(&text, LEGEND_FONT_SIZE).0
        })
        .fold(0.0_f64, f64::max);

    // ── Compute viewBox ────────────────────────────────────────────────────────
    let chart_and_legend_width =
        PIE_WIDTH + MARGIN + LEGEND_RECT_SIZE + LEGEND_SPACING + legend_text_width;

    // Title width (centered at pieWidth/2)
    let title_text_str = diag.title.as_deref().unwrap_or("");
    let title_width = if title_text_str.is_empty() {
        0.0
    } else {
        measure_browser(title_text_str, TITLE_FONT_SIZE).0
    };
    let title_left = PIE_WIDTH / 2.0 - title_width / 2.0;
    let title_right = PIE_WIDTH / 2.0 + title_width / 2.0;

    let view_box_x = 0.0_f64.min(title_left);
    let view_box_right = chart_and_legend_width.max(title_right);
    let total_width = view_box_right - view_box_x;

    // ── Generate SVG ──────────────────────────────────────────────────────────
    let id = "mermaid-pie";

    // The main group is translated to the center of the pie area
    let group_tx = PIE_WIDTH / 2.0; // 225.0
    let group_ty = HEIGHT / 2.0; // 225.0

    let mut svg_parts: Vec<String> = Vec::new();

    // SVG root
    svg_parts.push(templates::svg_root(
        id,
        &fmt(view_box_x),
        &fmt(total_width),
        &fmt(HEIGHT),
        &fmt(total_width),
        vars.font_family,
    ));

    // Empty first group (Mermaid always emits this)
    svg_parts.push("<g></g>".to_string());

    // Main group translated to pie center
    svg_parts.push(templates::main_group(&fmt(group_tx), &fmt(group_ty)));

    // Outer circle — Mermaid always uses black stroke regardless of theme
    svg_parts.push(templates::outer_circle(
        &fmt(radius + OUTER_STROKE_WIDTH / 2.0),
        "black",
    ));

    // Pie slices (paths)
    for (i, arc) in filtered_arcs.iter().enumerate() {
        // Find original index in all_sections for color assignment
        let color_idx = diag.sections.get_index_of(&arc.label).unwrap_or(i);
        let color = pie_slice_color(color_idx, primary_color, secondary_color, theme);
        let d = arc_path(arc.start_angle, arc.end_angle, radius);
        // Mermaid always uses black stroke for slice dividers regardless of theme
        svg_parts.push(templates::pie_slice(&d, color, "black"));
    }

    // Percentage labels
    for arc in &filtered_arcs {
        let pct = (arc.value / total_sum * 100.0).round() as u64;
        let (cx, cy) = arc_centroid(arc.start_angle, arc.end_angle, label_radius);
        svg_parts.push(templates::slice_label(
            &fmt(cx),
            &fmt(cy),
            pct,
            primary_text,
        ));
    }

    // Title text
    let title_y = -((HEIGHT - 50.0) / 2.0); // -(200.0)
    svg_parts.push(templates::title_text(
        &fmt(title_y),
        &esc(title_text_str),
        primary_text,
    ));

    // Legend items
    let legend_height = LEGEND_RECT_SIZE + LEGEND_SPACING;
    let legend_offset = (legend_height * all_sections.len() as f64) / 2.0;

    for (i, (label, value)) in all_sections.iter().enumerate() {
        // Use insertion-order index for color
        let color = pie_slice_color(i, primary_color, secondary_color, theme);
        let vertical = (i as f64) * legend_height - legend_offset;
        let legend_text = if diag.show_data {
            format!("{} [{}]", label, fmt_value(**value))
        } else {
            (*label).to_string()
        };

        // Colors are pre-computed as rgb(...) strings in constants — use directly.
        svg_parts.push(templates::legend_item(
            &fmt(LEGEND_HORIZONTAL_OFFSET),
            &fmt(vertical),
            color,
            &esc(&legend_text),
            primary_text,
        ));
    }

    // Close main group and SVG
    svg_parts.push("</g>".to_string());
    svg_parts.push("</svg>".to_string());

    svg_parts.join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::pie::parser;

    #[test]
    fn basic_render_produces_svg() {
        let input = "pie\n    \"Dogs\" : 386\n    \"Cats\" : 85\n    \"Rats\" : 15";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("pieCircle"));
        assert!(svg.contains("pieOuterCircle"));
        assert!(svg.contains("79%"));
        assert!(svg.contains("Dogs"));
    }

    #[test]
    fn arc_path_full_circle_large_arc() {
        // A slice > 180° must have large-arc-flag=1
        // Arc format: A rx,ry,x-rot,large-arc,sweep,x,y — so "0,1,1," = no-rotation,large,sweep
        let path = arc_path(-PI / 2.0, PI / 2.0 * 3.0, 185.0);
        assert!(
            path.contains(",0,1,1,"),
            "Expected large-arc-flag=1 in: {}",
            path
        );
    }

    #[test]
    fn fmt_value_integer() {
        assert_eq!(fmt_value(386.0), "386");
        assert_eq!(fmt_value(42.96), "42.96");
    }

    #[test]
    fn pie_color_static_0_is_valid() {
        // PIE_COLORS_STATIC[0] is pie3 (hsl-based)
        assert!(
            PIE_COLORS_STATIC[0].starts_with('#')
                || PIE_COLORS_STATIC[0].starts_with("hsl(")
                || PIE_COLORS_STATIC[0].starts_with("rgb("),
            "Expected a valid color string, got: {}",
            PIE_COLORS_STATIC[0]
        );
    }

    #[test]
    fn pie_slice_color_uses_theme_vars() {
        // Index 0 → primary, index 1 → secondary
        assert_eq!(
            pie_slice_color(0, "#ECECFF", "#ffffde", Theme::Default),
            "#ECECFF"
        );
        assert_eq!(
            pie_slice_color(1, "#ECECFF", "#ffffde", Theme::Default),
            "#ffffde"
        );
        // Index 2 → PIE_COLORS_STATIC[0]
        assert_eq!(
            pie_slice_color(2, "#ECECFF", "#ffffde", Theme::Default),
            PIE_COLORS_STATIC[0]
        );
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "pie title Pets\n    \"Dogs\" : 386\n    \"Cats\" : 85\n    \"Rats\" : 15";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
