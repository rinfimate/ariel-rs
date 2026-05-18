use super::constants::*;
use super::parser::PieDiagram;
use super::templates;
use crate::text::measure;
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
///   pie1  = #ECECFF       (primaryColor, HSL 240,100%,96.27%)
///   pie2  = #ffffde       (secondaryColor, HSL 60,100%,93.53%)
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

/// Format f64 with enough precision but stripping trailing zeros (matches D3 output style).
fn fmt(v: f64) -> String {
    // Snap near-zero to zero (avoids floating-point accumulation artifacts like -4.5e-14)
    let v = if v.abs() < 1e-10 { 0.0 } else { v };
    // Use up to 15 significant digits, trim trailing zeros after decimal
    let s = format!("{:.15}", v);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    // If empty or just "-", return "0"
    if s.is_empty() || s == "-" {
        "0".to_string()
    } else {
        s.to_string()
    }
}

/// Pick a color for a slice by index (wraps around after 12).
fn slice_color(index: usize) -> &'static str {
    PIE_COLORS[index % PIE_COLORS.len()]
}

pub fn render(diag: &PieDiagram, theme: Theme, _use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
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
            measure(&text, LEGEND_FONT_SIZE).0
        })
        .fold(0.0_f64, f64::max)
        * LEGEND_TEXT_SCALE;

    // ── Compute viewBox ────────────────────────────────────────────────────────
    let chart_and_legend_width =
        PIE_WIDTH + MARGIN + LEGEND_RECT_SIZE + LEGEND_SPACING + legend_text_width;

    // Title width (centered at pieWidth/2)
    let title_text_str = diag.title.as_deref().unwrap_or("");
    let title_width = if title_text_str.is_empty() {
        0.0
    } else {
        measure(title_text_str, TITLE_FONT_SIZE).0
    };
    let title_left = PIE_WIDTH / 2.0 - title_width / 2.0;
    let title_right = PIE_WIDTH / 2.0 + title_width / 2.0;

    let view_box_x = 0.0_f64.min(title_left);
    let view_box_right = chart_and_legend_width.max(title_right);
    let total_width = view_box_right - view_box_x;

    // ── Generate SVG ──────────────────────────────────────────────────────────
    let id = "mermaid-pie";
    let style = build_style(id, ff);

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
    ));
    svg_parts.push(format!("<style>{}</style>", style));

    // Empty first group (Mermaid always emits this)
    svg_parts.push("<g></g>".to_string());

    // Main group translated to pie center
    svg_parts.push(templates::main_group(&fmt(group_tx), &fmt(group_ty)));

    // Outer circle
    svg_parts.push(templates::outer_circle(&fmt(
        radius + OUTER_STROKE_WIDTH / 2.0
    )));

    // Pie slices (paths)
    for (i, arc) in filtered_arcs.iter().enumerate() {
        // Find original index in all_sections for color assignment
        let color_idx = diag.sections.get_index_of(&arc.label).unwrap_or(i);
        let color = slice_color(color_idx);
        let d = arc_path(arc.start_angle, arc.end_angle, radius);
        svg_parts.push(templates::pie_slice(&d, color));
    }

    // Percentage labels
    for arc in &filtered_arcs {
        let pct = (arc.value / total_sum * 100.0).round() as u64;
        let (cx, cy) = arc_centroid(arc.start_angle, arc.end_angle, label_radius);
        svg_parts.push(templates::slice_label(&fmt(cx), &fmt(cy), pct));
    }

    // Title text
    let title_y = -((HEIGHT - 50.0) / 2.0); // -(200.0)
    svg_parts.push(templates::title_text(
        &fmt(title_y),
        &escape_text(title_text_str),
    ));

    // Legend items
    let legend_height = LEGEND_RECT_SIZE + LEGEND_SPACING;
    let legend_offset = (legend_height * all_sections.len() as f64) / 2.0;

    for (i, (label, value)) in all_sections.iter().enumerate() {
        // Use insertion-order index for color
        let color = slice_color(i);
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
            &escape_text(&legend_text),
        ));
    }

    // Close main group and SVG
    svg_parts.push("</g>".to_string());
    svg_parts.push("</svg>".to_string());

    svg_parts.join("")
}

/// Format a value for showData display: integers show without decimal, floats show as-is.
fn fmt_value(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        // Strip trailing zeros
        let s = format!("{:.10}", v);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        s.to_string()
    }
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn build_style(id: &str, ff: &str) -> String {
    format!(
        r#"#{id}{{font-family:{ff};font-size:16px;fill:#333;}}@keyframes edge-animation-frame{{from{{stroke-dashoffset:0;}}}}@keyframes dash{{to{{stroke-dashoffset:0;}}}}#{id} .edge-animation-slow{{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 50s linear infinite;stroke-linecap:round;}}#{id} .edge-animation-fast{{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 20s linear infinite;stroke-linecap:round;}}#{id} .error-icon{{fill:#552222;}}#{id} .error-text{{fill:#552222;stroke:#552222;}}#{id} .edge-thickness-normal{{stroke-width:1px;}}#{id} .edge-thickness-thick{{stroke-width:3.5px;}}#{id} .edge-pattern-solid{{stroke-dasharray:0;}}#{id} .edge-thickness-invisible{{stroke-width:0;fill:none;}}#{id} .edge-pattern-dashed{{stroke-dasharray:3;}}#{id} .edge-pattern-dotted{{stroke-dasharray:2;}}#{id} .marker{{fill:#333333;stroke:#333333;}}#{id} .marker.cross{{stroke:#333333;}}#{id} svg{{font-family:{ff};font-size:16px;}}#{id} p{{margin:0;}}#{id} .pieCircle{{stroke:black;stroke-width:2px;opacity:0.7;}}#{id} .pieOuterCircle{{stroke:black;stroke-width:2px;fill:none;}}#{id} .pieTitleText{{text-anchor:middle;font-size:25px;fill:black;font-family:{ff};}}#{id} .slice{{font-family:{ff};fill:#333;font-size:17px;}}#{id} .legend text{{fill:black;font-family:{ff};font-size:17px;}}#{id} :root{{--mermaid-font-family:{ff};}}"#,
        id = id,
        ff = ff,
    )
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
    fn pie_color_0_is_rgb() {
        // PIE_COLORS are pre-computed as rgb() strings — no parsing needed
        assert!(
            PIE_COLORS[0].starts_with("rgb("),
            "Expected rgb() string, got: {}",
            PIE_COLORS[0]
        );
    }

    #[test]
    fn pie_color_1_is_rgb() {
        assert!(
            PIE_COLORS[1].starts_with("rgb("),
            "Expected rgb() string, got: {}",
            PIE_COLORS[1]
        );
    }

    #[test]
    #[ignore = "platform-specific float precision — run locally"]
    fn snapshot_default_theme() {
        let input = "pie title Pets\n    \"Dogs\" : 386\n    \"Cats\" : 85\n    \"Rats\" : 15";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(svg);
    }
}
