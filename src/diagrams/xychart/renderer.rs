use super::constants::*;
use super::parser::{AxisData, Orientation, PlotData, XyChart};
use super::templates;
/// Faithful Rust port of Mermaid's xychart renderer.
///
/// Architecture mirrors the TypeScript source exactly:
///   - Orchestrator: calculateVerticalSpace / calculateHorizontalSpace
///   - BaseAxis (BandAxis / LinearAxis): calculateSpace, getScaleValue, getDrawableElements
///   - ChartTitle: calculateSpace, getDrawableElements
///   - BasePlot → LinePlot / BarPlot: getDrawableElement
///   - xychartRenderer.ts draw() function: emits SVG from DrawableElem list
///
/// Default config values from config.schema.yaml:
///   width=700, height=500, titleFontSize=20, titlePadding=10, showTitle=true
///   plotReservedSpacePercent=50, chartOrientation=vertical
///   xAxis/yAxis: showLabel=true, labelFontSize=14, labelPadding=5,
///                showTitle=true, titleFontSize=16, titlePadding=5,
///                showTick=true, tickLength=5, tickWidth=2,
///                showAxisLine=true, axisLineWidth=2
use crate::text_browser_metrics::measure_exact as measure;
use crate::theme::Theme;

// ── Default config constants (from config.schema.yaml) ────────────────────────
// All constants are imported from super::constants via `use super::constants::*`.

// ── Drawable element types (mirrors interfaces.ts) ────────────────────────────

#[derive(Debug, Clone)]
struct RectElem {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    fill: String,
    stroke_width: f64,
    stroke_fill: String,
}

#[derive(Debug, Clone, PartialEq)]
enum TextVerticalPos {
    Top,
    Middle,
}

#[derive(Debug, Clone, PartialEq)]
enum TextHorizontalPos {
    #[allow(dead_code)]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
struct TextElem {
    x: f64,
    y: f64,
    text: String,
    fill: String,
    vertical_pos: TextVerticalPos,
    horizontal_pos: TextHorizontalPos,
    font_size: f64,
    rotation: f64,
}

#[derive(Debug, Clone)]
struct PathElem {
    path: String,
    fill: Option<String>,
    stroke_width: f64,
    stroke_fill: String,
}

#[derive(Debug, Clone)]
enum DrawableElem {
    Rect {
        group_texts: Vec<String>,
        data: Vec<RectElem>,
    },
    Text {
        group_texts: Vec<String>,
        data: Vec<TextElem>,
    },
    Path {
        group_texts: Vec<String>,
        data: Vec<PathElem>,
    },
}

// ── Bounding rectangle ────────────────────────────────────────────────────────
#[derive(Debug, Clone, Default)]
struct BoundingRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

// ── Axis position ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
enum AxisPosition {
    Left,
    #[allow(dead_code)]
    Right,
    Top,
    Bottom,
}

// ── Axis implementation (BaseAxis + BandAxis + LinearAxis) ────────────────────
struct Axis {
    data: AxisData,
    position: AxisPosition,
    bounding_rect: BoundingRect,
    range: [f64; 2],
    outer_padding: f64,
    show_title: bool,
    show_label: bool,
    show_tick: bool,
    show_axis_line: bool,
    title_text_height: f64,
    label_text_height: f64,
    // Resolved colors
    label_color: String,
    title_color: String,
    tick_color: String,
    axis_line_color: String,
}

impl Axis {
    fn new(
        data: AxisData,
        label_color: &str,
        title_color: &str,
        tick_color: &str,
        axis_line_color: &str,
    ) -> Self {
        Axis {
            data,
            position: AxisPosition::Left,
            bounding_rect: BoundingRect::default(),
            range: [0.0, 10.0],
            outer_padding: 0.0,
            show_title: false,
            show_label: false,
            show_tick: false,
            show_axis_line: false,
            title_text_height: 0.0,
            label_text_height: 0.0,
            label_color: label_color.to_string(),
            title_color: title_color.to_string(),
            tick_color: tick_color.to_string(),
            axis_line_color: axis_line_color.to_string(),
        }
    }

    fn set_axis_position(&mut self, pos: AxisPosition) {
        self.position = pos;
        let range = self.range;
        self.set_range(range);
    }

    fn set_range(&mut self, range: [f64; 2]) {
        self.range = range;
        match self.position {
            AxisPosition::Left | AxisPosition::Right => {
                self.bounding_rect.height = range[1] - range[0];
            }
            _ => {
                self.bounding_rect.width = range[1] - range[0];
            }
        }
        self.recalculate_scale();
    }

    /// Effective range after outer padding (matches BaseAxis.getRange())
    fn get_range(&self) -> [f64; 2] {
        [
            self.range[0] + self.outer_padding,
            self.range[1] - self.outer_padding,
        ]
    }

    fn recalculate_scale(&mut self) {
        // Scale is computed on-demand in get_scale_value; nothing to precompute here.
        // (The D3 scale is replaced by direct linear/band interpolation in get_scale_value)
    }

    /// Map a category string to pixel position on a Band axis.
    /// BandAxis uses D3 scalePoint semantics (paddingInner=1, bandwidth=0):
    ///   positions are evenly spaced across [r[0], r[1]] with n-1 intervals.
    ///   position[i] = r[0] + step * i, where step = span / (n - 1).
    ///   For n==1, position[0] = (r[0] + r[1]) / 2.
    /// For Linear axes use get_scale_value_f64 (no string parsing).
    fn get_scale_value(&self, value: &str) -> f64 {
        match &self.data {
            AxisData::Band { categories, .. } => {
                // D3 scalePoint (paddingInner=1, paddingOuter=0):
                // bandwidth=0, step = span / max(1, n-1)
                // position[i] = r[0] + step * i
                let r = self.get_range();
                let n = categories.len();
                if n == 0 {
                    return r[0];
                }
                let idx = categories.iter().position(|c| c == value).unwrap_or(0);
                let span = r[1] - r[0];
                if n == 1 {
                    return r[0] + span / 2.0;
                }
                let step = span / (n - 1) as f64;
                r[0] + step * idx as f64
            }
            AxisData::Linear { .. } => {
                // Linear axes should be positioned with get_scale_value_f64 or scale_linear.
                // This branch is only reached if called directly on a Linear axis with a string,
                // which should not happen in normal rendering flow.
                self.get_range()[0]
            }
        }
    }

    /// Map a pre-parsed f64 value to pixel position on a Linear axis.
    /// Used for plot data points (category, value) where value is already f64.
    fn get_scale_value_f64(&self, val: f64) -> f64 {
        match &self.data {
            AxisData::Band { .. } => {
                // Not applicable for Band axis in plot context; fall back to range start.
                self.get_range()[0]
            }
            AxisData::Linear { .. } => self.scale_linear(val),
        }
    }

    /// Core linear interpolation — separated so both get_scale_value and
    /// get_scale_value_f64 can share it without any string parsing.
    fn scale_linear(&self, val: f64) -> f64 {
        let r = self.get_range();
        let domain = if let AxisData::Linear { min, max, .. } = &self.data {
            if self.position == AxisPosition::Left {
                [*max, *min]
            } else {
                [*min, *max]
            }
        } else {
            return r[0];
        };
        if (domain[1] - domain[0]).abs() < 1e-15 {
            return r[0];
        }
        r[0] + (val - domain[0]) / (domain[1] - domain[0]) * (r[1] - r[0])
    }

    /// Returns (label, pixel_position) pairs for all tick marks.
    /// For Band axes the label is the category name and the position is looked up by name.
    /// For Linear axes the label is a formatted number and the position is computed from the
    /// pre-parsed f64 tick value — no re-parsing of strings needed.
    fn get_tick_entries(&self) -> Vec<(String, f64)> {
        match &self.data {
            AxisData::Band { categories, .. } => categories
                .iter()
                .map(|c| (c.clone(), self.get_scale_value(c)))
                .collect(),
            AxisData::Linear { min, max, .. } => nice_ticks(*min, *max, 10)
                .into_iter()
                .map(|v| (format_tick_value(v), self.scale_linear(v)))
                .collect(),
        }
    }

    /// Returns just the tick label strings (for measure/layout purposes).
    fn get_tick_values(&self) -> Vec<String> {
        match &self.data {
            AxisData::Band { categories, .. } => categories.clone(),
            AxisData::Linear { min, max, .. } => {
                // Match D3's scale.ticks() default: count = 10
                nice_ticks(*min, *max, 10)
                    .into_iter()
                    .map(format_tick_value)
                    .collect()
            }
        }
    }

    fn get_tick_distance(&self) -> f64 {
        let ticks = self.get_tick_values();
        let n = ticks.len();
        if n == 0 {
            return 0.0;
        }
        match &self.data {
            AxisData::Band { .. } => {
                // For bar-width calculation, use the FULL range span / n
                // (mirrors Mermaid's baseAxis.ts getTickDistance for BandAxis).
                let full_span = (self.range[1] - self.range[0]).abs();
                full_span / n as f64
            }
            AxisData::Linear { .. } => {
                let r = self.get_range();
                let span = (r[1] - r[0]).abs();
                span / n as f64
            }
        }
    }

    fn get_axis_outer_padding(&self) -> f64 {
        self.outer_padding
    }

    fn recalculate_outer_padding_to_draw_bar(&mut self) {
        let tick_dist = self.get_tick_distance();
        // Mirrors baseAxis.ts recalculateOuterPaddingToDrawBar():
        //   outerPaddingToSet = floor(BAR_WIDTH_TO_TICK_WIDTH_RATIO * tickDist / 2)
        //   if outerPaddingToSet > outerPadding: outerPadding = outerPaddingToSet
        let desired = (BAR_WIDTH_TO_TICK_WIDTH_RATIO * tick_dist / 2.0).floor();
        if desired > self.outer_padding {
            self.outer_padding = desired;
        }
        self.recalculate_scale();
    }

    fn set_bounding_box_xy(&mut self, x: f64, y: f64) {
        self.bounding_rect.x = x;
        self.bounding_rect.y = y;
    }

    /// calculateSpace — mirrors BaseAxis.calculateSpace()
    fn calculate_space(&mut self, avail_w: f64, avail_h: f64) -> (f64, f64) {
        match self.position {
            AxisPosition::Left | AxisPosition::Right => {
                self.calculate_space_vertical(avail_w, avail_h);
            }
            _ => {
                self.calculate_space_horizontal(avail_w, avail_h);
            }
        }
        self.recalculate_scale();
        (self.bounding_rect.width, self.bounding_rect.height)
    }

    /// calculateSpaceIfDrawnVertical
    fn calculate_space_vertical(&mut self, avail_w: f64, avail_h: f64) {
        let mut available_width = avail_w;

        if AXIS_SHOW_AXIS_LINE && available_width > AXIS_LINE_WIDTH {
            available_width -= AXIS_LINE_WIDTH;
            self.show_axis_line = true;
        }
        if AXIS_SHOW_LABEL {
            let (label_w, label_h) = self.get_max_label_dimension();
            let max_padding = MAX_OUTER_PADDING_PERCENT_FOR_WRT_LABEL * avail_h;
            self.outer_padding = (label_h / 2.0).min(max_padding);
            let width_required = label_w + AXIS_LABEL_PADDING * 2.0;
            if width_required <= available_width {
                available_width -= width_required;
                self.show_label = true;
            }
        }
        if AXIS_SHOW_TICK && available_width >= AXIS_TICK_LENGTH {
            self.show_tick = true;
            available_width -= AXIS_TICK_LENGTH;
        }
        if AXIS_SHOW_TITLE && !self.data.title().is_empty() {
            let (_, title_h) = measure(self.data.title(), AXIS_TITLE_FONT_SIZE);
            let width_required = title_h + AXIS_TITLE_PADDING * 2.0;
            self.title_text_height = title_h;
            if width_required <= available_width {
                available_width -= width_required;
                self.show_title = true;
            }
        }
        self.bounding_rect.width = avail_w - available_width;
        self.bounding_rect.height = avail_h;
    }

    /// calculateSpaceIfDrawnHorizontally
    fn calculate_space_horizontal(&mut self, avail_w: f64, avail_h: f64) {
        let mut available_height = avail_h;

        if AXIS_SHOW_AXIS_LINE && available_height > AXIS_LINE_WIDTH {
            available_height -= AXIS_LINE_WIDTH;
            self.show_axis_line = true;
        }
        if AXIS_SHOW_LABEL {
            let (label_w, label_h) = self.get_max_label_dimension();
            self.label_text_height = label_h;
            let max_padding = MAX_OUTER_PADDING_PERCENT_FOR_WRT_LABEL * avail_w;
            self.outer_padding = (label_w / 2.0).min(max_padding);
            let height_required = label_h + AXIS_LABEL_PADDING * 2.0;
            if height_required <= available_height {
                available_height -= height_required;
                self.show_label = true;
            }
        }
        if AXIS_SHOW_TICK && available_height >= AXIS_TICK_LENGTH {
            self.show_tick = true;
            available_height -= AXIS_TICK_LENGTH;
        }
        if AXIS_SHOW_TITLE && !self.data.title().is_empty() {
            let (_, title_h) = measure(self.data.title(), AXIS_TITLE_FONT_SIZE);
            let height_required = title_h + AXIS_TITLE_PADDING * 2.0;
            self.title_text_height = title_h;
            if height_required <= available_height {
                available_height -= height_required;
                self.show_title = true;
            }
        }
        self.bounding_rect.width = avail_w;
        self.bounding_rect.height = avail_h - available_height;
    }

    fn get_max_label_dimension(&self) -> (f64, f64) {
        let ticks = self.get_tick_values();
        ticks
            .iter()
            .map(|t| measure(t, AXIS_LABEL_FONT_SIZE))
            .fold((0.0_f64, 0.0_f64), |(mw, mh), (w, h)| {
                (mw.max(w), mh.max(h))
            })
    }

    fn get_drawable_elements(&self) -> Vec<DrawableElem> {
        match self.position {
            AxisPosition::Left => self.drawable_for_left(),
            AxisPosition::Bottom => self.drawable_for_bottom(),
            AxisPosition::Top => self.drawable_for_top(),
            AxisPosition::Right => vec![],
        }
    }

    // ── Left axis drawable elements ───────────────────────────────────────────

    fn drawable_for_left(&self) -> Vec<DrawableElem> {
        let mut out = vec![];
        let br = &self.bounding_rect;

        if self.show_axis_line {
            let x = br.x + br.width - AXIS_LINE_WIDTH / 2.0;
            out.push(DrawableElem::Path {
                group_texts: vec!["left-axis".into(), "axisl-line".into()],
                data: vec![PathElem {
                    path: format!("M {},{} L {},{}", x, br.y, x, br.y + br.height),
                    stroke_fill: self.axis_line_color.clone(),
                    stroke_width: AXIS_LINE_WIDTH,
                    fill: None,
                }],
            });
        }

        if self.show_label {
            let tick_entries = self.get_tick_entries();
            let label_x = br.x + br.width
                - if self.show_label {
                    AXIS_LABEL_PADDING
                } else {
                    0.0
                }
                - if self.show_tick {
                    AXIS_TICK_LENGTH
                } else {
                    0.0
                }
                - if self.show_axis_line {
                    AXIS_LINE_WIDTH
                } else {
                    0.0
                };
            out.push(DrawableElem::Text {
                group_texts: vec!["left-axis".into(), "label".into()],
                data: tick_entries
                    .iter()
                    .map(|(label, pos)| TextElem {
                        text: label.clone(),
                        x: label_x,
                        y: *pos,
                        fill: self.label_color.clone(),
                        font_size: AXIS_LABEL_FONT_SIZE,
                        rotation: 0.0,
                        vertical_pos: TextVerticalPos::Middle,
                        horizontal_pos: TextHorizontalPos::Right,
                    })
                    .collect(),
            });
        }

        if self.show_tick {
            let tick_entries = self.get_tick_entries();
            let x = br.x + br.width
                - if self.show_axis_line {
                    AXIS_LINE_WIDTH
                } else {
                    0.0
                };
            out.push(DrawableElem::Path {
                group_texts: vec!["left-axis".into(), "ticks".into()],
                data: tick_entries
                    .iter()
                    .map(|(_, pos)| {
                        let sy = *pos;
                        PathElem {
                            path: format!("M {},{} L {},{}", x, sy, x - AXIS_TICK_LENGTH, sy),
                            stroke_fill: self.tick_color.clone(),
                            stroke_width: AXIS_TICK_WIDTH,
                            fill: None,
                        }
                    })
                    .collect(),
            });
        }

        if self.show_title {
            out.push(DrawableElem::Text {
                group_texts: vec!["left-axis".into(), "title".into()],
                data: vec![TextElem {
                    text: self.data.title().to_string(),
                    x: br.x + AXIS_TITLE_PADDING,
                    y: br.y + br.height / 2.0,
                    fill: self.title_color.clone(),
                    font_size: AXIS_TITLE_FONT_SIZE,
                    rotation: 270.0,
                    vertical_pos: TextVerticalPos::Top,
                    horizontal_pos: TextHorizontalPos::Center,
                }],
            });
        }

        out
    }

    // ── Bottom axis drawable elements ─────────────────────────────────────────

    fn drawable_for_bottom(&self) -> Vec<DrawableElem> {
        let mut out = vec![];
        let br = &self.bounding_rect;

        if self.show_axis_line {
            let y = br.y + AXIS_LINE_WIDTH / 2.0;
            out.push(DrawableElem::Path {
                group_texts: vec!["bottom-axis".into(), "axis-line".into()],
                data: vec![PathElem {
                    path: format!("M {},{} L {},{}", br.x, y, br.x + br.width, y),
                    stroke_fill: self.axis_line_color.clone(),
                    stroke_width: AXIS_LINE_WIDTH,
                    fill: None,
                }],
            });
        }

        if self.show_label {
            let tick_entries = self.get_tick_entries();
            let label_y = br.y
                + AXIS_LABEL_PADDING
                + if self.show_tick {
                    AXIS_TICK_LENGTH
                } else {
                    0.0
                }
                + if self.show_axis_line {
                    AXIS_LINE_WIDTH
                } else {
                    0.0
                };
            out.push(DrawableElem::Text {
                group_texts: vec!["bottom-axis".into(), "label".into()],
                data: tick_entries
                    .iter()
                    .map(|(label, pos)| TextElem {
                        text: label.clone(),
                        x: *pos,
                        y: label_y,
                        fill: self.label_color.clone(),
                        font_size: AXIS_LABEL_FONT_SIZE,
                        rotation: 0.0,
                        vertical_pos: TextVerticalPos::Top,
                        horizontal_pos: TextHorizontalPos::Center,
                    })
                    .collect(),
            });
        }

        if self.show_tick {
            let tick_entries = self.get_tick_entries();
            let y = br.y
                + if self.show_axis_line {
                    AXIS_LINE_WIDTH
                } else {
                    0.0
                };
            out.push(DrawableElem::Path {
                group_texts: vec!["bottom-axis".into(), "ticks".into()],
                data: tick_entries
                    .iter()
                    .map(|(_, pos)| {
                        let sx = *pos;
                        PathElem {
                            path: format!("M {},{} L {},{}", sx, y, sx, y + AXIS_TICK_LENGTH),
                            stroke_fill: self.tick_color.clone(),
                            stroke_width: AXIS_TICK_WIDTH,
                            fill: None,
                        }
                    })
                    .collect(),
            });
        }

        if self.show_title {
            let r = self.range;
            out.push(DrawableElem::Text {
                group_texts: vec!["bottom-axis".into(), "title".into()],
                data: vec![TextElem {
                    text: self.data.title().to_string(),
                    x: r[0] + (r[1] - r[0]) / 2.0,
                    y: br.y + br.height - AXIS_TITLE_PADDING - self.title_text_height,
                    fill: self.title_color.clone(),
                    font_size: AXIS_TITLE_FONT_SIZE,
                    rotation: 0.0,
                    vertical_pos: TextVerticalPos::Top,
                    horizontal_pos: TextHorizontalPos::Center,
                }],
            });
        }

        out
    }

    // ── Top axis drawable elements ────────────────────────────────────────────

    fn drawable_for_top(&self) -> Vec<DrawableElem> {
        let mut out = vec![];
        let br = &self.bounding_rect;

        if self.show_axis_line {
            let y = br.y + br.height - AXIS_LINE_WIDTH / 2.0;
            out.push(DrawableElem::Path {
                group_texts: vec!["top-axis".into(), "axis-line".into()],
                data: vec![PathElem {
                    path: format!("M {},{} L {},{}", br.x, y, br.x + br.width, y),
                    stroke_fill: self.axis_line_color.clone(),
                    stroke_width: AXIS_LINE_WIDTH,
                    fill: None,
                }],
            });
        }

        if self.show_label {
            let tick_entries = self.get_tick_entries();
            let label_y =
                br.y + if self.show_title {
                    self.title_text_height + AXIS_TITLE_PADDING * 2.0
                } else {
                    0.0
                } + AXIS_LABEL_PADDING;
            out.push(DrawableElem::Text {
                group_texts: vec!["top-axis".into(), "label".into()],
                data: tick_entries
                    .iter()
                    .map(|(label, pos)| TextElem {
                        text: label.clone(),
                        x: *pos,
                        y: label_y,
                        fill: self.label_color.clone(),
                        font_size: AXIS_LABEL_FONT_SIZE,
                        rotation: 0.0,
                        vertical_pos: TextVerticalPos::Top,
                        horizontal_pos: TextHorizontalPos::Center,
                    })
                    .collect(),
            });
        }

        if self.show_tick {
            let tick_entries = self.get_tick_entries();
            let y = br.y;
            out.push(DrawableElem::Path {
                group_texts: vec!["top-axis".into(), "ticks".into()],
                data: tick_entries
                    .iter()
                    .map(|(_, pos)| {
                        let sx = *pos;
                        let tick_y_bot = y + br.height
                            - if self.show_axis_line {
                                AXIS_LINE_WIDTH
                            } else {
                                0.0
                            };
                        let tick_y_top = tick_y_bot
                            - AXIS_TICK_LENGTH
                            - if self.show_axis_line {
                                AXIS_LINE_WIDTH
                            } else {
                                0.0
                            };
                        PathElem {
                            path: format!("M {},{} L {},{}", sx, tick_y_bot, sx, tick_y_top),
                            stroke_fill: self.tick_color.clone(),
                            stroke_width: AXIS_TICK_WIDTH,
                            fill: None,
                        }
                    })
                    .collect(),
            });
        }

        if self.show_title {
            out.push(DrawableElem::Text {
                group_texts: vec!["top-axis".into(), "title".into()],
                data: vec![TextElem {
                    text: self.data.title().to_string(),
                    x: br.x + br.width / 2.0,
                    y: br.y + AXIS_TITLE_PADDING,
                    fill: self.title_color.clone(),
                    font_size: AXIS_TITLE_FONT_SIZE,
                    rotation: 0.0,
                    vertical_pos: TextVerticalPos::Top,
                    horizontal_pos: TextHorizontalPos::Center,
                }],
            });
        }

        out
    }
}

// ── Chart Title component ─────────────────────────────────────────────────────

struct ChartTitle {
    bounding_rect: BoundingRect,
    show_chart_title: bool,
    title: String,
    title_color: String,
}

impl ChartTitle {
    fn new(title: &str, title_color: &str) -> Self {
        ChartTitle {
            bounding_rect: BoundingRect::default(),
            show_chart_title: false,
            title: title.to_string(),
            title_color: title_color.to_string(),
        }
    }

    fn calculate_space(&mut self, avail_w: f64, _avail_h: f64) -> (f64, f64) {
        if self.title.is_empty() {
            return (0.0, 0.0);
        }
        let (title_w, title_h) = measure(&self.title, TITLE_FONT_SIZE);
        let width_required = title_w.max(avail_w);
        let height_required = title_h + 2.0 * TITLE_PADDING;
        if title_w <= width_required && title_h <= height_required {
            self.bounding_rect.width = width_required;
            self.bounding_rect.height = height_required;
            self.show_chart_title = true;
        }
        (self.bounding_rect.width, self.bounding_rect.height)
    }

    #[allow(dead_code)]
    fn set_bounding_box_xy(&mut self, x: f64, y: f64) {
        self.bounding_rect.x = x;
        self.bounding_rect.y = y;
    }

    fn get_drawable_elements(&self) -> Vec<DrawableElem> {
        if !self.show_chart_title {
            return vec![];
        }
        let br = &self.bounding_rect;
        vec![DrawableElem::Text {
            group_texts: vec!["chart-title".into()],
            data: vec![TextElem {
                font_size: TITLE_FONT_SIZE,
                text: self.title.clone(),
                vertical_pos: TextVerticalPos::Middle,
                horizontal_pos: TextHorizontalPos::Center,
                x: br.x + br.width / 2.0,
                y: br.y + br.height / 2.0,
                fill: self.title_color.clone(),
                rotation: 0.0,
            }],
        }]
    }
}

// ── Plot bounding rect (used by bar plot) ─────────────────────────────────────

struct PlotArea {
    bounding_rect: BoundingRect,
}

impl PlotArea {
    fn new() -> Self {
        PlotArea {
            bounding_rect: BoundingRect::default(),
        }
    }
    fn calculate_space(&mut self, w: f64, h: f64) -> (f64, f64) {
        self.bounding_rect.width = w;
        self.bounding_rect.height = h;
        (w, h)
    }
    fn set_bounding_box_xy(&mut self, x: f64, y: f64) {
        self.bounding_rect.x = x;
        self.bounding_rect.y = y;
    }
}

// ── Line plot (mirrors linePlot.ts) ──────────────────────────────────────────

fn line_plot_elements(
    plot: &PlotData,
    x_axis: &Axis,
    y_axis: &Axis,
    orientation: &Orientation,
    plot_index: usize,
) -> Vec<DrawableElem> {
    let (stroke_fill, stroke_width, data) = match plot {
        PlotData::Line {
            stroke_fill,
            stroke_width,
            data,
        } => (stroke_fill, *stroke_width, data),
        _ => return vec![],
    };

    if data.is_empty() {
        return vec![];
    }

    // Map each data point to (x_pixel, y_pixel)
    let points: Vec<(f64, f64)> = data
        .iter()
        .map(|(cat, val)| {
            let xp = x_axis.get_scale_value(cat);
            let yp = y_axis.get_scale_value_f64(*val);
            (xp, yp)
        })
        .collect();

    // Build SVG path (D3 line generator equivalent)
    let path = if *orientation == Orientation::Horizontal {
        // horizontal: x↔y swapped
        build_line_path(points.iter().map(|(x, y)| (*y, *x)))
    } else {
        build_line_path(points.iter().copied())
    };

    vec![DrawableElem::Path {
        group_texts: vec!["plot".into(), format!("line-plot-{}", plot_index)],
        data: vec![PathElem {
            path,
            stroke_fill: stroke_fill.clone(),
            stroke_width,
            fill: None,
        }],
    }]
}

fn build_line_path<I: Iterator<Item = (f64, f64)>>(mut points: I) -> String {
    let mut path = String::new();
    if let Some((x, y)) = points.next() {
        path.push_str(&format!("M{},{}", fmt(x), fmt(y)));
        for (x, y) in points {
            path.push_str(&format!("L{},{}", fmt(x), fmt(y)));
        }
    }
    path
}

// ── Bar plot (mirrors barPlot.ts) ─────────────────────────────────────────────

fn bar_plot_elements(
    plot: &PlotData,
    plot_bounding_rect: &BoundingRect,
    x_axis: &Axis,
    y_axis: &Axis,
    orientation: &Orientation,
    plot_index: usize,
) -> Vec<DrawableElem> {
    let (fill, data) = match plot {
        PlotData::Bar { fill, data } => (fill, data),
        _ => return vec![],
    };

    if data.is_empty() {
        return vec![];
    }

    let bar_padding_percent = BAR_PADDING_PERCENT;
    let bar_width = (x_axis.get_axis_outer_padding() * 2.0).min(x_axis.get_tick_distance())
        * (1.0 - bar_padding_percent);
    let bar_width_half = bar_width / 2.0;

    let rects: Vec<RectElem> = data
        .iter()
        .map(|(cat, val)| {
            let xp = x_axis.get_scale_value(cat);
            let yp = y_axis.get_scale_value_f64(*val);

            if *orientation == Orientation::Horizontal {
                RectElem {
                    x: plot_bounding_rect.x,
                    y: xp - bar_width_half,
                    height: bar_width,
                    width: yp - plot_bounding_rect.x,
                    fill: fill.clone(),
                    stroke_width: 0.0,
                    stroke_fill: fill.clone(),
                }
            } else {
                RectElem {
                    x: xp - bar_width_half,
                    y: yp,
                    width: bar_width,
                    height: plot_bounding_rect.y + plot_bounding_rect.height - yp,
                    fill: fill.clone(),
                    stroke_width: 0.0,
                    stroke_fill: fill.clone(),
                }
            }
        })
        .collect();

    vec![DrawableElem::Rect {
        group_texts: vec!["plot".into(), format!("bar-plot-{}", plot_index)],
        data: rects,
    }]
}

// ── Orchestrator (mirrors orchestrator.ts) ────────────────────────────────────

fn orchestrate(chart: &XyChart) -> Vec<DrawableElem> {
    let mut title_comp = ChartTitle::new(&chart.title, TITLE_COLOR);
    let mut plot_area = PlotArea::new();

    let mut x_axis = Axis::new(
        chart.x_axis.clone(),
        X_AXIS_LABEL_COLOR,
        X_AXIS_TITLE_COLOR,
        X_AXIS_TICK_COLOR,
        X_AXIS_LINE_COLOR,
    );
    let mut y_axis = Axis::new(
        chart.y_axis.clone(),
        Y_AXIS_LABEL_COLOR,
        Y_AXIS_TITLE_COLOR,
        Y_AXIS_TICK_COLOR,
        Y_AXIS_LINE_COLOR,
    );

    let has_bar = chart
        .plots
        .iter()
        .any(|p| matches!(p, PlotData::Bar { .. }));

    if chart.orientation == Orientation::Horizontal {
        calculate_horizontal_space(
            &mut title_comp,
            &mut plot_area,
            &mut x_axis,
            &mut y_axis,
            has_bar,
        );
    } else {
        calculate_vertical_space(
            &mut title_comp,
            &mut plot_area,
            &mut x_axis,
            &mut y_axis,
            has_bar,
        );
    }

    // Collect all drawable elements.
    // Order matches Mermaid's xychartRenderer.ts: title → plot → axes.
    let mut drawables: Vec<DrawableElem> = vec![];

    // Title
    drawables.extend(title_comp.get_drawable_elements());

    // Plots (drawn first so axes render on top)
    for (i, plot) in chart.plots.iter().enumerate() {
        match plot {
            PlotData::Line { .. } => {
                drawables.extend(line_plot_elements(
                    plot,
                    &x_axis,
                    &y_axis,
                    &chart.orientation,
                    i,
                ));
            }
            PlotData::Bar { .. } => {
                drawables.extend(bar_plot_elements(
                    plot,
                    &plot_area.bounding_rect,
                    &x_axis,
                    &y_axis,
                    &chart.orientation,
                    i,
                ));
            }
        }
    }

    // Axes (drawn on top of plots)
    drawables.extend(x_axis.get_drawable_elements());
    drawables.extend(y_axis.get_drawable_elements());

    drawables
}

/// calculateVerticalSpace (mirrors orchestrator.ts)
fn calculate_vertical_space(
    title: &mut ChartTitle,
    plot_area: &mut PlotArea,
    x_axis: &mut Axis,
    y_axis: &mut Axis,
    has_bar: bool,
) {
    let mut avail_w = WIDTH;
    let mut avail_h = HEIGHT;
    let mut chart_w = ((avail_w * PLOT_RESERVED_SPACE_PERCENT) / 100.0).floor();
    let mut chart_h = ((avail_h * PLOT_RESERVED_SPACE_PERCENT) / 100.0).floor();

    let (sw, sh) = plot_area.calculate_space(chart_w, chart_h);
    avail_w -= sw;
    avail_h -= sh;

    let (_, th) = title.calculate_space(WIDTH, avail_h);
    let plot_y = th;
    avail_h -= th;

    x_axis.set_axis_position(AxisPosition::Bottom);
    let (_, xsh) = x_axis.calculate_space(avail_w, avail_h);
    avail_h -= xsh;

    y_axis.set_axis_position(AxisPosition::Left);
    let (ysw, _) = y_axis.calculate_space(avail_w, avail_h);
    let plot_x = ysw;
    avail_w -= ysw;

    if avail_w > 0.0 {
        chart_w += avail_w;
    }
    if avail_h > 0.0 {
        chart_h += avail_h;
    }

    plot_area.calculate_space(chart_w, chart_h);
    plot_area.set_bounding_box_xy(plot_x, plot_y);

    x_axis.set_range([plot_x, plot_x + chart_w]);
    x_axis.set_bounding_box_xy(plot_x, plot_y + chart_h);

    y_axis.set_range([plot_y, plot_y + chart_h]);
    y_axis.set_bounding_box_xy(0.0, plot_y);

    if has_bar {
        x_axis.recalculate_outer_padding_to_draw_bar();
    }
}

/// calculateHorizontalSpace (mirrors orchestrator.ts)
fn calculate_horizontal_space(
    title: &mut ChartTitle,
    plot_area: &mut PlotArea,
    x_axis: &mut Axis,
    y_axis: &mut Axis,
    has_bar: bool,
) {
    let mut avail_w = WIDTH;
    let mut avail_h = HEIGHT;
    let mut chart_w = ((avail_w * PLOT_RESERVED_SPACE_PERCENT) / 100.0).floor();
    let mut chart_h = ((avail_h * PLOT_RESERVED_SPACE_PERCENT) / 100.0).floor();

    let (sw, sh) = plot_area.calculate_space(chart_w, chart_h);
    avail_w -= sw;
    avail_h -= sh;

    let (_, th) = title.calculate_space(WIDTH, avail_h);
    let title_y_end = th;
    avail_h -= th;

    // x-axis is on the left for horizontal charts
    x_axis.set_axis_position(AxisPosition::Left);
    let (xsw, _) = x_axis.calculate_space(avail_w, avail_h);
    avail_w -= xsw;
    let plot_x = xsw;

    // y-axis is on top for horizontal charts
    y_axis.set_axis_position(AxisPosition::Top);
    let (_, ysh) = y_axis.calculate_space(avail_w, avail_h);
    avail_h -= ysh;
    let plot_y = title_y_end + ysh;

    if avail_w > 0.0 {
        chart_w += avail_w;
    }
    if avail_h > 0.0 {
        chart_h += avail_h;
    }

    plot_area.calculate_space(chart_w, chart_h);
    plot_area.set_bounding_box_xy(plot_x, plot_y);

    y_axis.set_range([plot_x, plot_x + chart_w]);
    y_axis.set_bounding_box_xy(plot_x, title_y_end);

    x_axis.set_range([plot_y, plot_y + chart_h]);
    x_axis.set_bounding_box_xy(0.0, plot_y);

    if has_bar {
        x_axis.recalculate_outer_padding_to_draw_bar();
    }
}

// ── SVG emission (mirrors xychartRenderer.ts draw()) ─────────────────────────

pub fn render(chart: &XyChart, theme: Theme, _use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let shapes = orchestrate(chart);

    let id = SVG_ID;
    let mut out = String::new();

    // SVG root
    out.push_str(&templates::svg_root(id, WIDTH as i64, HEIGHT as i64));
    out.push_str(&format!("<style>{}</style>", build_style(id, ff)));

    // Background rect (mirrors xychartRenderer.ts background rect)
    out.push_str(&templates::main_group_with_bg(
        WIDTH as i64,
        HEIGHT as i64,
        BG_COLOR,
    ));

    // Collect groups → hierarchical group structure like getGroup() in renderer.ts
    // We flatten: each DrawableElem has group_texts; we emit them in order.
    let mut current_groups: Vec<String> = vec![];

    // Close any open groups first at the end, so we track depth.
    // Simpler approach: emit each drawable in its own group context.
    // Mirror getGroup() by nesting groups matching group_texts prefix chain.
    for shape in &shapes {
        let (group_texts, elem_svg) = match shape {
            DrawableElem::Rect { group_texts, data } => {
                if data.is_empty() {
                    continue;
                }
                let svg = render_rects(data);
                (group_texts, svg)
            }
            DrawableElem::Text { group_texts, data } => {
                if data.is_empty() {
                    continue;
                }
                let svg = render_texts(data);
                (group_texts, svg)
            }
            DrawableElem::Path { group_texts, data } => {
                if data.is_empty() {
                    continue;
                }
                let svg = render_paths(data);
                (group_texts, svg)
            }
        };

        // Close/open groups to match group_texts
        let common_prefix = current_groups
            .iter()
            .zip(group_texts.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // Close extra groups
        for _ in 0..(current_groups.len() - common_prefix) {
            out.push_str("</g>");
        }
        current_groups.truncate(common_prefix);

        // Open new groups
        for g in &group_texts[common_prefix..] {
            out.push_str(&templates::group_open(&escape_attr(g)));
            current_groups.push(g.clone());
        }

        out.push_str(&elem_svg);
    }

    // Close remaining groups
    for _ in &current_groups {
        out.push_str("</g>");
    }

    out.push_str("</g>"); // close main
    out.push_str("</svg>");
    out
}

fn render_rects(data: &[RectElem]) -> String {
    data.iter()
        .map(|r| {
            templates::chart_rect(
                &fmt(r.x),
                &fmt(r.y),
                &fmt(r.width),
                &fmt(r.height),
                &escape_attr(&r.fill),
                &escape_attr(&r.stroke_fill),
                &fmt(r.stroke_width),
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn render_texts(data: &[TextElem]) -> String {
    data.iter()
        .map(|t| {
            let dominant_baseline = match t.vertical_pos {
                TextVerticalPos::Top => "text-before-edge",
                TextVerticalPos::Middle => "middle",
            };
            let text_anchor = match t.horizontal_pos {
                TextHorizontalPos::Left => "start",
                TextHorizontalPos::Right => "end",
                TextHorizontalPos::Center => "middle",
            };
            let transform = format!(
                "translate({},{}) rotate({})",
                fmt(t.x),
                fmt(t.y),
                fmt(t.rotation)
            );
            templates::chart_text(
                &escape_attr(&t.fill),
                &fmt(t.font_size),
                dominant_baseline,
                text_anchor,
                &escape_attr(&transform),
                &escape_text(&t.text),
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn render_paths(data: &[PathElem]) -> String {
    data.iter()
        .map(|p| {
            templates::chart_path(
                &escape_attr(&p.path),
                &escape_attr(p.fill.as_deref().unwrap_or("none")),
                &escape_attr(&p.stroke_fill),
                &fmt(p.stroke_width),
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

// ── Utility functions ─────────────────────────────────────────────────────────

/// Format a f64 value with reasonable precision, stripping trailing zeros.
fn fmt(v: f64) -> String {
    let v = if v.abs() < 1e-10 { 0.0 } else { v };
    let s = format!("{:.3}", v);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    if s.is_empty() || s == "-" {
        "0".to_string()
    } else {
        s.to_string()
    }
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Generate nice tick values matching D3's `scale.ticks(count)` algorithm.
///
/// D3 uses `tickIncrement(start, stop, count)` with thresholds:
///   e10 = sqrt(50) ≈ 7.0711, e5 = sqrt(10) ≈ 3.1623, e2 = sqrt(2) ≈ 1.4142
///
/// Default count = 10 (mirrors D3's `scale.ticks()` with no argument).
fn nice_ticks(min: f64, max: f64, target_count: usize) -> Vec<f64> {
    if min >= max || target_count == 0 {
        return vec![min, max];
    }
    let e10: f64 = 50.0_f64.sqrt(); // ≈ 7.0711
    let e5: f64 = 10.0_f64.sqrt(); // ≈ 3.1623
    let e2: f64 = 2.0_f64.sqrt(); // ≈ 1.4142

    let span = max - min;
    let raw_step = span / target_count as f64;
    let power = raw_step.log10().floor();
    let factor = 10.0_f64.powf(power);
    let error = raw_step / factor;

    let nice_factor = if error >= e10 {
        10.0
    } else if error >= e5 {
        5.0
    } else if error >= e2 {
        2.0
    } else {
        1.0
    };
    let step = nice_factor * factor;

    let first = (min / step).ceil() * step;
    let last = (max / step).floor() * step;
    let mut ticks = vec![];
    let mut t = first;
    // Use a small epsilon relative to step to avoid floating-point overshoot
    while t <= last + step * 1e-10 {
        // Round to avoid floating-point drift (e.g. 4999.9999 instead of 5000)
        let rounded = (t / step).round() * step;
        ticks.push(rounded);
        t += step;
    }
    ticks
}

fn format_tick_value(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e12 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.6}", v);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

fn build_style(id: &str, ff: &str) -> String {
    format!(
        concat!(
            "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}",
            "#{id} .main{{}}",
            "#{id} text{{font-family:{ff};}}",
            "#{id} .chart-title text{{font-size:{tfs}px;fill:{tc};text-anchor:middle;dominant-baseline:middle;}}",
            "#{id} .left-axis path,#{id} .bottom-axis path,#{id} .top-axis path{{fill:none;stroke:#333;}}",
            "#{id} .left-axis .label text,#{id} .bottom-axis .label text,#{id} .top-axis .label text{{fill:#333;font-size:{lfs}px;}}",
            "#{id} .plot rect{{opacity:0.85;}}",
            "#{id} .plot path{{fill:none;}}",
        ),
        id = id,
        ff = ff,
        tfs = TITLE_FONT_SIZE as i64,
        tc = TITLE_COLOR,
        lfs = AXIS_LABEL_FONT_SIZE as i64,
    )
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    #[test]
    fn basic_render_produces_svg() {
        let input = r#"xychart-beta
    title "Sales Revenue"
    x-axis [jan, feb, mar, apr, may, jun, jul, aug, sep, oct, nov, dec]
    y-axis "Revenue (in $)" 4000 --> 11000
    bar [5000, 6000, 7500, 8200, 9500, 10500, 11000, 10200, 9200, 8500, 7000, 6000]
    line [5000, 6000, 7500, 8200, 9500, 10500, 11000, 10200, 9200, 8500, 7000, 6000]"#;
        let chart = parser::parse(input).diagram;
        let svg = render(&chart, crate::theme::Theme::Default, false);
        assert!(svg.contains("<svg"), "no svg root");
        assert!(svg.contains("Sales Revenue"), "no title");
        assert!(svg.contains("rect"), "no rects");
        assert!(svg.contains("path"), "no paths");
    }

    #[test]
    fn nice_ticks_basic() {
        let ticks = nice_ticks(4000.0, 11000.0, 5);
        assert!(!ticks.is_empty());
        assert!(ticks[0] >= 4000.0);
        assert!(*ticks.last().unwrap() <= 12000.0);
    }

    #[test]
    #[ignore = "platform-specific float precision — run locally"]
    fn snapshot_default_theme() {
        let input = "xychart-beta\n    title \"Sales Revenue\"\n    x-axis [jan, feb, mar, apr, may, jun, jul, aug, sep, oct, nov, dec]\n    y-axis \"Revenue (in $)\" 4000 --> 11000\n    bar [5000, 6000, 7500, 8200, 9500, 10500, 11000, 10200, 9200, 8500, 7000, 6000]\n    line [5000, 6000, 7500, 8200, 9500, 10500, 11000, 10200, 9200, 8500, 7000, 6000]";
        let chart = parser::parse(input).diagram;
        let svg = render(&chart, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(svg);
    }
}
