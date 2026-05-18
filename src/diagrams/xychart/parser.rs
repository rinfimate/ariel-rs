/// Parser for Mermaid xychart-beta syntax.
///
/// Supported grammar (from xychartDb.ts):
///   xychart-beta [horizontal]
///       title "Chart Title"
///       x-axis [cat1, cat2, ...] | x-axis "Title" min --> max
///       y-axis "Title" min --> max
///       bar [v1, v2, ...]
///       line [v1, v2, ...]
/// Axis data type: either categorical (band) or numeric (linear).
#[derive(Debug, Clone)]
pub enum AxisData {
    Band {
        title: String,
        categories: Vec<String>,
    },
    Linear {
        title: String,
        min: f64,
        max: f64,
    },
}

impl AxisData {
    pub fn title(&self) -> &str {
        match self {
            AxisData::Band { title, .. } => title,
            AxisData::Linear { title, .. } => title,
        }
    }
}

/// A single plot series.
#[derive(Debug, Clone)]
pub enum PlotData {
    Line {
        stroke_fill: String,
        stroke_width: f64,
        data: Vec<(String, f64)>,
    },
    Bar {
        fill: String,
        data: Vec<(String, f64)>,
    },
}

/// Parsed representation of an xychart-beta diagram.
#[derive(Debug, Clone)]
pub struct XyChart {
    pub title: String,
    pub orientation: Orientation,
    pub x_axis: AxisData,
    pub y_axis: AxisData,
    pub plots: Vec<PlotData>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

/// Color palette matching Mermaid default theme `plotColorPalette`.
/// From theme-default.ts xyChart section (default Mermaid theme).
const PLOT_COLORS: &[&str] = &[
    "#ECECFF", "#8493A6", "#3949AB", "#00ACC1", "#43A047", "#FB8C00", "#E53935", "#FD79A8",
    "#636E72", "#FDCB6E",
];

fn plot_color(index: usize) -> String {
    PLOT_COLORS[index % PLOT_COLORS.len()].to_string()
}

pub fn parse(input: &str) -> crate::error::ParseResult<XyChart> {
    let mut title = String::new();
    let mut orientation = Orientation::Vertical;
    let mut x_axis: Option<AxisData> = None;
    let mut y_axis: Option<AxisData> = None;
    let mut plots: Vec<PlotData> = Vec::new();
    let mut plot_index = 0usize;
    // Track whether axes were explicitly set (for auto-range logic)
    let mut has_set_x_axis = false;
    let mut has_set_y_axis = false;
    // Raw numeric data for each plot (needed for auto y-range)
    let mut plot_raw: Vec<(String, Vec<f64>)> = Vec::new(); // ("line"|"bar", data)

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        let lower = trimmed.to_lowercase();

        if lower.starts_with("xychart-beta") {
            // Check for horizontal modifier
            let rest = trimmed["xychart-beta".len()..].trim();
            if rest.eq_ignore_ascii_case("horizontal") {
                orientation = Orientation::Horizontal;
            }
            continue;
        }

        if lower.starts_with("title") {
            let rest = trimmed["title".len()..].trim();
            title = unquote(rest);
            continue;
        }

        if lower.starts_with("x-axis") {
            let rest = trimmed["x-axis".len()..].trim();
            if let Some(axis) = parse_axis_line(rest) {
                x_axis = Some(axis);
                has_set_x_axis = true;
            }
            continue;
        }

        if lower.starts_with("y-axis") {
            let rest = trimmed["y-axis".len()..].trim();
            if let Some(axis) = parse_axis_line(rest) {
                y_axis = Some(axis);
                has_set_y_axis = true;
            }
            continue;
        }

        if lower.starts_with("bar") {
            let rest = trimmed["bar".len()..].trim();
            let data = parse_number_list(rest);
            plot_raw.push(("bar".to_string(), data.clone()));
            // We'll resolve plot data after all lines are parsed
            plots.push(PlotData::Bar {
                fill: plot_color(plot_index),
                data: Vec::new(), // filled in post-process
            });
            plot_index += 1;
            continue;
        }

        if lower.starts_with("line") {
            let rest = trimmed["line".len()..].trim();
            let data = parse_number_list(rest);
            plot_raw.push(("line".to_string(), data.clone()));
            plots.push(PlotData::Line {
                stroke_fill: plot_color(plot_index),
                stroke_width: 2.0,
                data: Vec::new(), // filled in post-process
            });
            plot_index += 1;
            continue;
        }
    }

    // Post-process: compute auto y-axis range from all plot data if not set
    if !has_set_y_axis && !plot_raw.is_empty() {
        let all_values: Vec<f64> = plot_raw
            .iter()
            .flat_map(|(_, v)| v.iter().copied())
            .collect();
        if !all_values.is_empty() {
            let min_v = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_v = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let ytitle = y_axis
                .as_ref()
                .map(|a| a.title().to_string())
                .unwrap_or_default();
            y_axis = Some(AxisData::Linear {
                title: ytitle,
                min: min_v,
                max: max_v,
            });
        }
    }

    // Post-process: compute auto x-axis range if not set
    if !has_set_x_axis && !plot_raw.is_empty() {
        let max_len = plot_raw.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
        let xtitle = x_axis
            .as_ref()
            .map(|a| a.title().to_string())
            .unwrap_or_default();
        x_axis = Some(AxisData::Linear {
            title: xtitle,
            min: 1.0,
            max: max_len as f64,
        });
    }

    let x_axis = x_axis.unwrap_or(AxisData::Band {
        title: String::new(),
        categories: Vec::new(),
    });
    let y_axis = y_axis.unwrap_or(AxisData::Linear {
        title: String::new(),
        min: 0.0,
        max: 100.0,
    });

    // Now transform plot data using axes (like transformDataWithoutCategory in xychartDb.ts)
    for (i, plot) in plots.iter_mut().enumerate() {
        let raw_data = &plot_raw[i].1;
        let transformed = transform_data(raw_data, &x_axis);
        match plot {
            PlotData::Line { data, .. } => *data = transformed,
            PlotData::Bar { data, .. } => *data = transformed,
        }
    }

    crate::error::ParseResult::ok(XyChart {
        title,
        orientation,
        x_axis,
        y_axis,
        plots,
    })
}

/// Transform raw numeric data into (category, value) pairs,
/// matching xychartDb.ts `transformDataWithoutCategory`.
fn transform_data(data: &[f64], x_axis: &AxisData) -> Vec<(String, f64)> {
    match x_axis {
        AxisData::Band { categories, .. } => categories
            .iter()
            .enumerate()
            .filter_map(|(i, cat)| data.get(i).map(|&v| (cat.clone(), v)))
            .collect(),
        AxisData::Linear { min, max, .. } => {
            if data.is_empty() {
                return Vec::new();
            }
            let n = data.len();
            let step = if n > 1 {
                (max - min) / (n as f64 - 1.0)
            } else {
                0.0
            };
            let mut cats = Vec::with_capacity(n);
            let mut x = *min;
            for _ in 0..n {
                cats.push(format!("{}", x));
                x += step;
            }
            cats.into_iter().zip(data.iter().copied()).collect()
        }
    }
}

/// Parse an axis line which is one of:
///   [cat1, cat2, ...]                    → Band
///   "Title" [cat1, ...]                  → Band with title
///   "Title" min --> max                  → Linear with title
///   min --> max                          → Linear without title
fn parse_axis_line(rest: &str) -> Option<AxisData> {
    let mut title = String::new();
    let mut s = rest;

    // Optional quoted title at the start
    if s.starts_with('"') {
        if let Some(end) = s[1..].find('"') {
            title = s[1..end + 1].to_string();
            s = s[end + 2..].trim();
        }
    }

    // Check for bracket list
    if s.starts_with('[') {
        let cats = parse_category_list(s);
        return Some(AxisData::Band {
            title,
            categories: cats,
        });
    }

    // Check for linear range: min --> max
    if s.contains("-->") {
        let parts: Vec<&str> = s.splitn(2, "-->").collect();
        let min: f64 = parts[0].trim().parse().ok()?;
        let max: f64 = parts[1].trim().parse().ok()?;
        return Some(AxisData::Linear { title, min, max });
    }

    None
}

/// Parse `[cat1, cat2, ...]` → Vec<String>
fn parse_category_list(s: &str) -> Vec<String> {
    let inner = s.trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(|c| c.trim().trim_matches('"').to_string())
        .filter(|c| !c.is_empty())
        .collect()
}

/// Parse `[1, 2, 3.5, ...]` → Vec<f64>
fn parse_number_list(s: &str) -> Vec<f64> {
    let inner = s.trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .filter_map(|n| n.trim().parse::<f64>().ok())
        .collect()
}

/// Remove surrounding quotes from a string.
fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_xychart() {
        let input = r#"xychart-beta
    title "Sales Revenue"
    x-axis [jan, feb, mar, apr, may, jun, jul, aug, sep, oct, nov, dec]
    y-axis "Revenue (in $)" 4000 --> 11000
    bar [5000, 6000, 7500, 8200, 9500, 10500, 11000, 10200, 9200, 8500, 7000, 6000]
    line [5000, 6000, 7500, 8200, 9500, 10500, 11000, 10200, 9200, 8500, 7000, 6000]"#;
        let chart = parse(input).diagram;
        assert_eq!(chart.title, "Sales Revenue");
        assert_eq!(chart.orientation, Orientation::Vertical);
        assert!(matches!(chart.x_axis, AxisData::Band { .. }));
        if let AxisData::Band { categories, .. } = &chart.x_axis {
            assert_eq!(categories.len(), 12);
            assert_eq!(categories[0], "jan");
        }
        assert!(
            matches!(chart.y_axis, AxisData::Linear { min, max, .. } if (min - 4000.0).abs() < 1e-9 && (max - 11000.0).abs() < 1e-9)
        );
        assert_eq!(chart.plots.len(), 2);
        if let PlotData::Bar { data, .. } = &chart.plots[0] {
            assert_eq!(data.len(), 12);
            assert_eq!(data[0], ("jan".to_string(), 5000.0));
        }
    }

    #[test]
    fn horizontal_orientation() {
        let input = "xychart-beta horizontal\n    bar [1, 2, 3]";
        let chart = parse(input).diagram;
        assert_eq!(chart.orientation, Orientation::Horizontal);
    }
}
