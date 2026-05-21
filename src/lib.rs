//! `ariel-rs` — a pure-Rust Mermaid diagram renderer.
//!
//! Converts [Mermaid](https://mermaid.js.org/) diagram source text into SVG
//! strings without requiring a JavaScript runtime or browser.
//!
//! # Quick start
//!
//! ```
//! let svg = ariel_rs::render("graph LR\n  A --> B", ariel_rs::theme::Theme::Default);
//! assert!(svg.contains("<svg"));
//! ```
//!
//! The two entry points are [`render`] and [`render_svg`] (an alias).  Both
//! accept any supported Mermaid diagram type and a [`theme::Theme`] variant.
//! For a fallible variant that returns [`RenderError`] instead of an error SVG,
//! use [`try_render`].
#![deny(missing_docs)]
pub(crate) mod diagrams;
/// Error and parse-result types for ariel-rs diagram parsing and rendering.
///
/// Key types exported here are [`ParseError`], [`ParseResult`], and
/// [`RenderError`], which are also re-exported at the crate root.
pub mod error;
pub(crate) mod error_svg;
pub(crate) mod icons;
pub(crate) mod style;
pub(crate) mod svg;
pub(crate) mod text;
pub(crate) mod text_browser_metrics;
/// Colour-theme types for Mermaid diagram rendering.
///
/// The two public types here are [`Theme`](theme::Theme), which you pass to
/// [`render`] / [`render_svg`], and [`ThemeVars`](theme::ThemeVars), which
/// holds the resolved colour and font values.
pub mod theme;

pub use error::{ParseError, ParseResult, RenderError};

/// Per-call rendering configuration passed to [`render_with_options`] and
/// [`try_render_with_options`].
///
/// Use [`RenderOptions::default()`] to get a zero-configuration value and then
/// set individual fields as needed.
///
/// # Examples
///
/// ```
/// use ariel_rs::{RenderOptions, theme::Theme};
///
/// let opts = RenderOptions {
///     theme: Theme::Dark,
///     font_family: Some("monospace".to_string()),
///     ..RenderOptions::default()
/// };
/// ```
pub struct RenderOptions {
    /// The colour theme to apply to the rendered diagram.
    pub theme: theme::Theme,
    /// Optional CSS font-family string (e.g. `"sans-serif"`).
    ///
    /// When `None` the renderer uses its built-in default.
    /// Support for this field in individual renderers is planned for a future
    /// release.
    pub font_family: Option<String>,
    /// Optional base font size in points.
    ///
    /// When `None` the renderer uses its built-in default.
    /// Support for this field in individual renderers is planned for a future
    /// release.
    pub font_size: Option<f64>,
    /// Optional maximum width of the output SVG in pixels.
    ///
    /// When `None` the renderer uses its built-in default.
    /// Support for this field in individual renderers is planned for a future
    /// release.
    pub max_width: Option<f64>,
    /// Optional background colour as a CSS colour string (e.g. `"#ffffff"`).
    ///
    /// When `None` the renderer uses the theme's default background.
    /// Support for this field in individual renderers is planned for a future
    /// release.
    pub background: Option<String>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            theme: theme::Theme::Default,
            font_family: None,
            font_size: None,
            max_width: None,
            background: None,
        }
    }
}

use std::any::Any;
use std::panic::{self, AssertUnwindSafe};

/// The type of a Mermaid diagram, as detected from the source text.
///
/// Returned by [`detect`]. Use this to inspect which diagram kind a source
/// string represents before (or instead of) rendering it.
///
/// This enum is `#[non_exhaustive]`: new diagram types may be added in future
/// versions without a breaking change.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagramType {
    /// Flowchart / graph diagram (`flowchart` or `graph` keyword).
    Flowchart,
    /// Pie chart diagram (`pie` keyword).
    Pie,
    /// Sequence diagram (`sequenceDiagram` keyword).
    Sequence,
    /// Entity-relationship diagram (`erDiagram` keyword).
    Er,
    /// Gantt chart diagram (`gantt` keyword).
    Gantt,
    /// Info diagram (`info` keyword) — displays the Mermaid version string.
    Info,
    /// State diagram (`stateDiagram` or `stateDiagram-v2` keyword).
    State,
    /// Class diagram (`classDiagram` keyword).
    Class,
    /// Git graph diagram (`gitGraph` keyword).
    Git,
    /// Mind map diagram (`mindmap` keyword).
    Mindmap,
    /// Timeline diagram (`timeline` keyword).
    Timeline,
    /// Quadrant chart diagram (`quadrantChart` keyword).
    Quadrant,
    /// XY chart diagram (`xychart-beta` keyword).
    XyChart,
    /// C4 architecture diagram (`C4Context`, `C4Container`, `C4Component`,
    /// `C4Dynamic`, or `C4Deployment` keyword).
    C4,
    /// Block diagram (`block-beta` keyword).
    Block,
    /// Packet diagram (`packet-beta` keyword).
    Packet,
    /// User journey diagram (`journey` keyword).
    Journey,
    /// Requirement diagram (`requirementDiagram` keyword).
    Requirement,
    /// Kanban board diagram (`kanban` keyword).
    Kanban,
    /// Sankey diagram (`sankey-beta` keyword, optionally preceded by YAML
    /// front matter).
    Sankey,
    /// Treemap diagram (`treemap` or `treemap-beta` keyword).
    Treemap,
    /// Radar chart diagram (`radar` or `radar-beta` keyword).
    Radar,
    /// Venn diagram (`venn`, `venn-beta`, or `vennDiagram` keyword).
    Venn,
    /// Architecture diagram (`architecture` or `architecture-beta` keyword).
    Architecture,
    /// Event modeling diagram (`eventmodeling` or `event-modeling` keyword).
    EventModeling,
    /// Ishikawa / fishbone diagram (`ishikawa` or `fishbone` keyword).
    Ishikawa,
    /// Wardley map diagram (`wardley` keyword).
    Wardley,
    /// Tree-view diagram (`treeView-beta` or `treeview-beta` keyword).
    TreeView,
    /// The input did not match any recognised diagram keyword.
    Unknown,
}

impl DiagramType {
    /// Return the canonical string label used in error messages and
    /// [`RenderError::diagram_type`](crate::error::RenderError::diagram_type).
    fn label(&self) -> &'static str {
        match self {
            DiagramType::Flowchart => "flowchart",
            DiagramType::Pie => "pie",
            DiagramType::Sequence => "sequenceDiagram",
            DiagramType::Er => "erDiagram",
            DiagramType::Gantt => "gantt",
            DiagramType::Info => "info",
            DiagramType::State => "stateDiagram",
            DiagramType::Class => "classDiagram",
            DiagramType::Git => "gitGraph",
            DiagramType::Mindmap => "mindmap",
            DiagramType::Timeline => "timeline",
            DiagramType::Quadrant => "quadrantChart",
            DiagramType::XyChart => "xychart-beta",
            DiagramType::C4 => "C4",
            DiagramType::Block => "block-beta",
            DiagramType::Packet => "packet-beta",
            DiagramType::Journey => "journey",
            DiagramType::Requirement => "requirementDiagram",
            DiagramType::Kanban => "kanban",
            DiagramType::Sankey => "sankey-beta",
            DiagramType::Treemap => "treemap",
            DiagramType::Radar => "radar",
            DiagramType::Venn => "venn",
            DiagramType::Architecture => "architecture",
            DiagramType::EventModeling => "eventmodeling",
            DiagramType::Ishikawa => "ishikawa",
            DiagramType::Wardley => "wardley",
            DiagramType::TreeView => "treeView-beta",
            DiagramType::Unknown => "unknown",
        }
    }
}

/// Strip YAML front matter (--- ... ---) from input, returning the remainder.
fn strip_frontmatter(input: &str) -> &str {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("---") {
        return input;
    }
    let after_open = &trimmed[3..];
    if let Some(nl) = after_open.find('\n') {
        let body_start = &after_open[nl + 1..];
        if let Some(close_pos) = body_start.find("\n---") {
            let remainder = &body_start[close_pos + 4..];
            if let Some(nl2) = remainder.find('\n') {
                return &remainder[nl2 + 1..];
            }
            return remainder;
        }
    }
    input
}

/// Parse a `theme:` key from YAML frontmatter (--- ... ---) and return the matching Theme.
fn frontmatter_theme(input: &str) -> Option<theme::Theme> {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after_open = &trimmed[3..];
    let nl = after_open.find('\n')?;
    let body = &after_open[nl + 1..];
    let close = body.find("\n---")?;
    let fm = &body[..close];
    for line in fm.lines() {
        if let Some(v) = line.trim().strip_prefix("theme:") {
            return match v.trim() {
                "dark" => Some(theme::Theme::Dark),
                "forest" => Some(theme::Theme::Forest),
                "neutral" => Some(theme::Theme::Neutral),
                "default" | "base" => Some(theme::Theme::Default),
                _ => None,
            };
        }
    }
    None
}

/// Extract a human-readable message from a `catch_unwind` error payload.
fn unwind_message(e: Box<dyn Any + Send>) -> String {
    if let Some(s) = e.downcast_ref::<&str>() {
        return s.to_string();
    }
    if let Some(s) = e.downcast_ref::<String>() {
        return s.clone();
    }
    "Rendering failed".to_string()
}

/// Detect which [`DiagramType`] a Mermaid source string represents.
///
/// The function inspects the leading keyword(s) of `input` (after stripping
/// ASCII whitespace) and returns the matching variant.  If no keyword matches,
/// [`DiagramType::Unknown`] is returned.
///
/// For the Sankey diagram type the function also checks for an optional YAML
/// front-matter block that precedes the `sankey-beta` keyword.
///
/// # Examples
///
/// ```
/// use ariel_rs::{detect, DiagramType};
///
/// assert_eq!(detect("graph LR\n  A --> B"), DiagramType::Flowchart);
/// assert_eq!(detect("pie\n  title Pets"), DiagramType::Pie);
/// assert_eq!(detect("not a diagram"), DiagramType::Unknown);
/// ```
pub fn detect(input: &str) -> DiagramType {
    // Strip YAML frontmatter before detection — any diagram type can have it.
    let stripped = strip_frontmatter(input.trim_start());
    let trimmed = stripped.trim_start();

    if trimmed.starts_with("flowchart") || trimmed.starts_with("graph ") {
        return DiagramType::Flowchart;
    }
    if trimmed.starts_with("pie") {
        return DiagramType::Pie;
    }
    if trimmed.starts_with("sequenceDiagram") {
        return DiagramType::Sequence;
    }
    if trimmed.starts_with("erDiagram") {
        return DiagramType::Er;
    }
    if trimmed.starts_with("gantt") {
        return DiagramType::Gantt;
    }
    if trimmed.starts_with("info") {
        return DiagramType::Info;
    }
    if trimmed.starts_with("stateDiagram-v2") || trimmed.starts_with("stateDiagram") {
        return DiagramType::State;
    }
    if trimmed.starts_with("classDiagram") {
        return DiagramType::Class;
    }
    if trimmed.starts_with("gitGraph") {
        return DiagramType::Git;
    }
    if trimmed.starts_with("mindmap") {
        return DiagramType::Mindmap;
    }
    if trimmed.starts_with("timeline") {
        return DiagramType::Timeline;
    }
    if trimmed.starts_with("quadrantChart") {
        return DiagramType::Quadrant;
    }
    if trimmed.starts_with("xychart") {
        return DiagramType::XyChart;
    }
    if trimmed.starts_with("C4Context")
        || trimmed.starts_with("C4Container")
        || trimmed.starts_with("C4Component")
        || trimmed.starts_with("C4Dynamic")
        || trimmed.starts_with("C4Deployment")
    {
        return DiagramType::C4;
    }
    if trimmed.starts_with("block") {
        return DiagramType::Block;
    }
    if trimmed.starts_with("packet-beta") || trimmed.starts_with("packet") {
        return DiagramType::Packet;
    }
    if trimmed.starts_with("journey") {
        return DiagramType::Journey;
    }
    if trimmed.starts_with("requirementDiagram") || trimmed.starts_with("requirement") {
        return DiagramType::Requirement;
    }
    if trimmed.starts_with("kanban") {
        return DiagramType::Kanban;
    }
    if trimmed.starts_with("sankey")
        || strip_frontmatter(trimmed)
            .trim_start()
            .starts_with("sankey")
    {
        return DiagramType::Sankey;
    }
    if trimmed.starts_with("treemap-beta") || trimmed.starts_with("treemap") {
        return DiagramType::Treemap;
    }
    if trimmed.starts_with("radar-beta") || trimmed.starts_with("radar") {
        return DiagramType::Radar;
    }
    if trimmed.starts_with("venn-beta")
        || trimmed.starts_with("vennDiagram")
        || trimmed.starts_with("venn")
    {
        return DiagramType::Venn;
    }
    if trimmed.starts_with("architecture-beta") || trimmed.starts_with("architecture") {
        return DiagramType::Architecture;
    }
    if trimmed.starts_with("eventmodeling") || trimmed.starts_with("event-modeling") {
        return DiagramType::EventModeling;
    }
    if trimmed.starts_with("fishbone") || trimmed.starts_with("ishikawa") {
        return DiagramType::Ishikawa;
    }
    if trimmed.starts_with("wardley") {
        return DiagramType::Wardley;
    }
    if trimmed.starts_with("treeView-beta")
        || trimmed.starts_with("treeview-beta")
        || trimmed.starts_with("treeView")
    {
        return DiagramType::TreeView;
    }

    DiagramType::Unknown
}

/// Render any Mermaid diagram source to an SVG string.
///
/// Returns an error SVG if the diagram type is unrecognized or if rendering
/// panics for any reason.
pub fn render(input: &str, theme: theme::Theme) -> String {
    let theme = frontmatter_theme(input).unwrap_or(theme);
    macro_rules! safe_render {
        ($diagram_type:expr, $call:expr) => {{
            let result = panic::catch_unwind(AssertUnwindSafe(|| $call));
            match result {
                Ok(svg) => svg::normalize_floats(&svg),
                Err(e) => {
                    let msg = unwind_message(e);
                    error_svg::render_error_svg($diagram_type, &msg)
                }
            }
        }};
    }

    let dt = detect(input.trim_start());
    let label = dt.label();
    match dt {
        DiagramType::Flowchart => {
            safe_render!(label, diagrams::flowchart::render_html(input, theme))
        }
        DiagramType::Pie => safe_render!(label, diagrams::pie::render_html(input, theme)),
        DiagramType::Sequence => safe_render!(label, diagrams::sequence::render_html(input, theme)),
        DiagramType::Er => safe_render!(label, {
            let d = diagrams::er::parser::parse(input);
            diagrams::er::render(&d.diagram, theme)
        }),
        DiagramType::Gantt => safe_render!(label, diagrams::gantt::render_html(input, theme)),
        DiagramType::Info => safe_render!(label, {
            let d = diagrams::info::parser::parse(input);
            diagrams::info::render(&d, theme)
        }),
        DiagramType::State => safe_render!(label, {
            let d = diagrams::state::parser::parse(input);
            diagrams::state::render(&d, theme)
        }),
        DiagramType::Class => {
            safe_render!(label, diagrams::class_diagram::render_html(input, theme))
        }
        DiagramType::Git => safe_render!(label, diagrams::git::render_html(input, theme)),
        DiagramType::Mindmap => safe_render!(label, diagrams::mindmap::render_html(input, theme)),
        DiagramType::Timeline => safe_render!(label, diagrams::timeline::render_html(input, theme)),
        DiagramType::Quadrant => safe_render!(label, diagrams::quadrant::render_html(input, theme)),
        DiagramType::XyChart => safe_render!(label, diagrams::xychart::render_html(input, theme)),
        DiagramType::C4 => safe_render!(label, diagrams::c4::render_html(input, theme)),
        DiagramType::Block => safe_render!(label, diagrams::block::render_html(input, theme)),
        DiagramType::Packet => safe_render!(label, diagrams::packet::render_html(input, theme)),
        DiagramType::Journey => safe_render!(label, diagrams::journey::render_html(input, theme)),
        DiagramType::Requirement => {
            safe_render!(label, diagrams::requirement::render_html(input, theme))
        }
        DiagramType::Kanban => safe_render!(label, diagrams::kanban::render_html(input, theme)),
        DiagramType::Sankey => safe_render!(label, diagrams::sankey::render_html(input, theme)),
        DiagramType::Treemap => safe_render!(label, diagrams::treemap::render_html(input, theme)),
        DiagramType::Radar => safe_render!(label, diagrams::radar::render_html(input, theme)),
        DiagramType::Venn => safe_render!(label, diagrams::venn::render_html(input, theme)),
        DiagramType::Architecture => {
            safe_render!(label, diagrams::architecture::render_html(input, theme))
        }
        DiagramType::EventModeling => {
            safe_render!(label, diagrams::eventmodeling::render_html(input, theme))
        }
        DiagramType::Ishikawa => safe_render!(label, diagrams::ishikawa::render_html(input, theme)),
        DiagramType::Wardley => safe_render!(label, diagrams::wardley::render_html(input, theme)),
        DiagramType::TreeView => {
            safe_render!(label, diagrams::treeview::render_html(input, theme))
        }
        DiagramType::Unknown => error_svg::render_error_svg(label, "Unrecognized diagram type."),
    }
}

/// Alias for [`render`] — renders any Mermaid diagram source to an SVG string.
pub fn render_svg(input: &str, theme: theme::Theme) -> String {
    render(input, theme)
}

/// Render any Mermaid diagram source to an SVG string.
///
/// Returns `Err(RenderError)` if the diagram type is unrecognised or if
/// rendering panics for any reason. On success returns the SVG string.
///
/// For a version that never fails and returns an error SVG instead, use
/// [`render`].
///
/// # Errors
///
/// Returns [`RenderError::unknown_type`] when the input does not match any
/// supported diagram keyword, or [`RenderError::from_panic`] if the renderer
/// panics internally.
///
/// # Examples
///
/// ```
/// let result = ariel_rs::try_render("graph LR\n  A --> B", ariel_rs::theme::Theme::Default);
/// assert!(result.is_ok());
/// let svg = result.unwrap();
/// assert!(svg.contains("<svg"));
/// ```
pub fn try_render(input: &str, theme: theme::Theme) -> Result<String, RenderError> {
    macro_rules! safe_try_render {
        ($diagram_type:expr, $call:expr) => {{
            let result = panic::catch_unwind(AssertUnwindSafe(|| $call));
            match result {
                Ok(svg) => Ok(svg::normalize_floats(&svg)),
                Err(e) => {
                    let msg = unwind_message(e);
                    Err(RenderError::from_panic($diagram_type, msg))
                }
            }
        }};
    }

    let dt = detect(input.trim_start());
    let label = dt.label();
    match dt {
        DiagramType::Flowchart => {
            safe_try_render!(label, diagrams::flowchart::render_html(input, theme))
        }
        DiagramType::Pie => safe_try_render!(label, diagrams::pie::render_html(input, theme)),
        DiagramType::Sequence => {
            safe_try_render!(label, diagrams::sequence::render_html(input, theme))
        }
        DiagramType::Er => safe_try_render!(label, {
            let d = diagrams::er::parser::parse(input);
            diagrams::er::render(&d.diagram, theme)
        }),
        DiagramType::Gantt => safe_try_render!(label, diagrams::gantt::render_html(input, theme)),
        DiagramType::Info => safe_try_render!(label, {
            let d = diagrams::info::parser::parse(input);
            diagrams::info::render(&d, theme)
        }),
        DiagramType::State => safe_try_render!(label, {
            let d = diagrams::state::parser::parse(input);
            diagrams::state::render(&d, theme)
        }),
        DiagramType::Class => {
            safe_try_render!(label, diagrams::class_diagram::render_html(input, theme))
        }
        DiagramType::Git => safe_try_render!(label, diagrams::git::render_html(input, theme)),
        DiagramType::Mindmap => {
            safe_try_render!(label, diagrams::mindmap::render_html(input, theme))
        }
        DiagramType::Timeline => {
            safe_try_render!(label, diagrams::timeline::render_html(input, theme))
        }
        DiagramType::Quadrant => {
            safe_try_render!(label, diagrams::quadrant::render_html(input, theme))
        }
        DiagramType::XyChart => {
            safe_try_render!(label, diagrams::xychart::render_html(input, theme))
        }
        DiagramType::C4 => safe_try_render!(label, diagrams::c4::render_html(input, theme)),
        DiagramType::Block => safe_try_render!(label, diagrams::block::render_html(input, theme)),
        DiagramType::Packet => safe_try_render!(label, diagrams::packet::render_html(input, theme)),
        DiagramType::Journey => {
            safe_try_render!(label, diagrams::journey::render_html(input, theme))
        }
        DiagramType::Requirement => {
            safe_try_render!(label, diagrams::requirement::render_html(input, theme))
        }
        DiagramType::Kanban => safe_try_render!(label, diagrams::kanban::render_html(input, theme)),
        DiagramType::Sankey => safe_try_render!(label, diagrams::sankey::render_html(input, theme)),
        DiagramType::Treemap => {
            safe_try_render!(label, diagrams::treemap::render_html(input, theme))
        }
        DiagramType::Radar => safe_try_render!(label, diagrams::radar::render_html(input, theme)),
        DiagramType::Venn => safe_try_render!(label, diagrams::venn::render_html(input, theme)),
        DiagramType::Architecture => {
            safe_try_render!(label, diagrams::architecture::render_html(input, theme))
        }
        DiagramType::EventModeling => {
            safe_try_render!(label, diagrams::eventmodeling::render_html(input, theme))
        }
        DiagramType::Ishikawa => {
            safe_try_render!(label, diagrams::ishikawa::render_html(input, theme))
        }
        DiagramType::Wardley => {
            safe_try_render!(label, diagrams::wardley::render_html(input, theme))
        }
        DiagramType::TreeView => {
            safe_try_render!(label, diagrams::treeview::render_html(input, theme))
        }
        DiagramType::Unknown => Err(RenderError::unknown_type()),
    }
}

/// Render any Mermaid diagram source to an SVG string using [`RenderOptions`].
///
/// This is the options-aware counterpart of [`render`].  Currently only
/// `options.theme` is forwarded to the underlying renderer; the remaining
/// fields (`font_family`, `font_size`, `max_width`, `background`) are reserved
/// for future use once individual renderers gain option support.
///
/// Returns an error SVG if the diagram type is unrecognized or if rendering
/// panics for any reason.
///
/// # Examples
///
/// ```
/// use ariel_rs::{render_with_options, RenderOptions, theme::Theme};
///
/// let opts = RenderOptions {
///     theme: Theme::Forest,
///     ..RenderOptions::default()
/// };
/// let svg = render_with_options("graph LR\n  A --> B", opts);
/// assert!(svg.contains("<svg"));
/// ```
pub fn render_with_options(input: &str, options: RenderOptions) -> String {
    render(input, options.theme)
}

/// Render any Mermaid diagram source to an SVG string using [`RenderOptions`].
///
/// This is the options-aware counterpart of [`try_render`].  Currently only
/// `options.theme` is forwarded to the underlying renderer; the remaining
/// fields (`font_family`, `font_size`, `max_width`, `background`) are reserved
/// for future use once individual renderers gain option support.
///
/// Returns `Err(RenderError)` if the diagram type is unrecognised or if
/// rendering panics for any reason.
///
/// # Errors
///
/// Returns [`RenderError::unknown_type`] when the input does not match any
/// supported diagram keyword, or [`RenderError::from_panic`] if the renderer
/// panics internally.
///
/// # Examples
///
/// ```
/// use ariel_rs::{try_render_with_options, RenderOptions, theme::Theme};
///
/// let opts = RenderOptions {
///     theme: Theme::Dark,
///     max_width: Some(800.0),
///     ..RenderOptions::default()
/// };
/// let result = try_render_with_options("pie\n  title Pets\n  \"Dogs\" : 40\n  \"Cats\" : 60", opts);
/// assert!(result.is_ok());
/// assert!(result.unwrap().contains("<svg"));
/// ```
pub fn try_render_with_options(input: &str, options: RenderOptions) -> Result<String, RenderError> {
    try_render(input, options.theme)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── render() happy-path tests ────────────────────────────────────────────

    #[test]
    fn render_flowchart() {
        let svg = render("flowchart TD\n  A --> B", theme::Theme::Default);
        assert!(svg.contains("<svg"));
        assert!(!svg.contains("Syntax error"));
    }

    #[test]
    fn render_pie() {
        let svg = render("pie title X\n  \"A\" : 1", theme::Theme::Default);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_sequence() {
        let svg = render(
            "sequenceDiagram\n  Alice->>Bob: Hello",
            theme::Theme::Default,
        );
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_er() {
        let svg = render("erDiagram\n  A ||--o{ B : has", theme::Theme::Default);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_gantt() {
        let svg = render(
            "gantt\n  dateFormat YYYY-MM-DD\n  section A\n  Task1: 2024-01-01, 7d",
            theme::Theme::Default,
        );
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_state() {
        let svg = render("stateDiagram-v2\n  [*] --> A", theme::Theme::Default);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_class() {
        let svg = render("classDiagram\n  class A", theme::Theme::Default);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_git() {
        let svg = render("gitGraph\n  commit", theme::Theme::Default);
        assert!(svg.contains("<svg"));
    }

    // ── render() unknown type returns error SVG, no panic ────────────────────

    #[test]
    fn render_unknown_type_returns_error_svg() {
        let svg = render("unknownDiagram\n  foo", theme::Theme::Default);
        assert!(svg.contains("<svg"));
    }

    // ── try_render() ─────────────────────────────────────────────────────────

    #[test]
    fn try_render_ok() {
        let result = try_render("pie title X\n  \"A\" : 1", theme::Theme::Default);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("<svg"));
    }

    #[test]
    fn try_render_unknown_returns_err() {
        let result = try_render("unknownDiagram\n  foo", theme::Theme::Default);
        assert!(result.is_err());
    }

    // ── detect() — every variant ──────────────────────────────────────────────

    #[test]
    fn detect_flowchart_keyword() {
        assert_eq!(detect("flowchart TD\n  A --> B"), DiagramType::Flowchart);
    }

    #[test]
    fn detect_graph_keyword() {
        assert_eq!(detect("graph LR\n  A --> B"), DiagramType::Flowchart);
    }

    #[test]
    fn detect_pie() {
        assert_eq!(detect("pie title X\n  \"A\" : 1"), DiagramType::Pie);
    }

    #[test]
    fn detect_sequence() {
        assert_eq!(
            detect("sequenceDiagram\n  A->>B: hi"),
            DiagramType::Sequence
        );
    }

    #[test]
    fn detect_er() {
        assert_eq!(detect("erDiagram\n  A ||--o{ B : has"), DiagramType::Er);
    }

    #[test]
    fn detect_gantt() {
        assert_eq!(detect("gantt\n  dateFormat YYYY-MM-DD"), DiagramType::Gantt);
    }

    #[test]
    fn detect_state() {
        assert_eq!(detect("stateDiagram\n  [*] --> A"), DiagramType::State);
    }

    #[test]
    fn detect_state_v2() {
        assert_eq!(detect("stateDiagram-v2\n  [*] --> A"), DiagramType::State);
    }

    #[test]
    fn detect_class() {
        assert_eq!(detect("classDiagram\n  class A"), DiagramType::Class);
    }

    #[test]
    fn detect_git() {
        assert_eq!(detect("gitGraph\n  commit"), DiagramType::Git);
    }

    #[test]
    fn detect_mindmap() {
        assert_eq!(detect("mindmap\n  root((A))"), DiagramType::Mindmap);
    }

    #[test]
    fn detect_timeline() {
        assert_eq!(detect("timeline\n  title History"), DiagramType::Timeline);
    }

    #[test]
    fn detect_quadrant() {
        assert_eq!(detect("quadrantChart\n  title Q"), DiagramType::Quadrant);
    }

    #[test]
    fn detect_xychart() {
        assert_eq!(detect("xychart-beta\n  line [1, 2]"), DiagramType::XyChart);
    }

    #[test]
    fn detect_c4_context() {
        assert_eq!(detect("C4Context\n  title T"), DiagramType::C4);
    }

    #[test]
    fn detect_c4_container() {
        assert_eq!(detect("C4Container\n  title T"), DiagramType::C4);
    }

    #[test]
    fn detect_block() {
        assert_eq!(detect("block-beta\n  A"), DiagramType::Block);
    }

    #[test]
    fn detect_packet() {
        assert_eq!(detect("packet-beta\n  0-7: A"), DiagramType::Packet);
    }

    #[test]
    fn detect_journey() {
        assert_eq!(detect("journey\n  title My"), DiagramType::Journey);
    }

    #[test]
    fn detect_requirement() {
        assert_eq!(
            detect("requirementDiagram\n  requirement R {}"),
            DiagramType::Requirement
        );
    }

    #[test]
    fn detect_kanban() {
        assert_eq!(detect("kanban\n  Todo\n    task1"), DiagramType::Kanban);
    }

    #[test]
    fn detect_sankey() {
        assert_eq!(detect("sankey-beta\n  A,B,10"), DiagramType::Sankey);
    }

    #[test]
    fn detect_treemap() {
        assert_eq!(detect("treemap\n  root\n    A: 1"), DiagramType::Treemap);
    }

    #[test]
    fn detect_treemap_beta() {
        assert_eq!(
            detect("treemap-beta\n  root\n    A: 1"),
            DiagramType::Treemap
        );
    }

    #[test]
    fn detect_radar() {
        assert_eq!(detect("radar\n  title R"), DiagramType::Radar);
    }

    #[test]
    fn detect_radar_beta() {
        assert_eq!(detect("radar-beta\n  title R"), DiagramType::Radar);
    }

    #[test]
    fn detect_venn() {
        assert_eq!(detect("venn\n  A"), DiagramType::Venn);
    }

    #[test]
    fn detect_venn_beta() {
        assert_eq!(detect("venn-beta\n  A"), DiagramType::Venn);
    }

    #[test]
    fn detect_venn_diagram() {
        assert_eq!(detect("vennDiagram\n  A"), DiagramType::Venn);
    }

    #[test]
    fn detect_architecture() {
        assert_eq!(
            detect("architecture\n  service A"),
            DiagramType::Architecture
        );
    }

    #[test]
    fn detect_architecture_beta() {
        assert_eq!(
            detect("architecture-beta\n  service A"),
            DiagramType::Architecture
        );
    }

    #[test]
    fn detect_event_modeling() {
        assert_eq!(detect("eventmodeling\n  A"), DiagramType::EventModeling);
    }

    #[test]
    fn detect_event_modeling_hyphen() {
        assert_eq!(detect("event-modeling\n  A"), DiagramType::EventModeling);
    }

    #[test]
    fn detect_ishikawa() {
        assert_eq!(detect("ishikawa\n  effect"), DiagramType::Ishikawa);
    }

    #[test]
    fn detect_fishbone() {
        assert_eq!(detect("fishbone\n  effect"), DiagramType::Ishikawa);
    }

    #[test]
    fn detect_wardley() {
        assert_eq!(detect("wardley\n  title W"), DiagramType::Wardley);
    }

    #[test]
    fn detect_treeview_beta() {
        assert_eq!(detect("treeView-beta\n    \"docs\""), DiagramType::TreeView);
    }

    #[test]
    fn detect_treeview_lowercase() {
        assert_eq!(detect("treeview-beta\n    \"docs\""), DiagramType::TreeView);
    }

    #[test]
    fn render_treeview() {
        let svg = render(
            "treeView-beta\n    \"docs\"\n        \"build\"\n",
            theme::Theme::Default,
        );
        assert!(svg.contains("<svg"));
        assert!(svg.contains("tree-view"));
        assert!(svg.contains("docs"));
    }

    #[test]
    fn detect_unknown() {
        assert_eq!(detect("not a diagram"), DiagramType::Unknown);
    }

    // ── render_with_options() ────────────────────────────────────────────────

    #[test]
    fn render_with_options_dark_theme() {
        let opts = RenderOptions {
            theme: theme::Theme::Dark,
            ..Default::default()
        };
        let svg = render_with_options("pie title X\n  \"A\" : 1", opts);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_with_options_forest_theme() {
        let opts = RenderOptions {
            theme: theme::Theme::Forest,
            ..Default::default()
        };
        let svg = render_with_options("flowchart TD\n  A --> B", opts);
        assert!(svg.contains("<svg"));
    }

    // ── try_render_with_options() ────────────────────────────────────────────

    #[test]
    fn try_render_with_options_ok() {
        let opts = RenderOptions {
            theme: theme::Theme::Dark,
            max_width: Some(800.0),
            ..Default::default()
        };
        let result =
            try_render_with_options("pie\n  title Pets\n  \"Dogs\" : 40\n  \"Cats\" : 60", opts);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("<svg"));
    }

    #[test]
    fn try_render_with_options_unknown_returns_err() {
        let opts = RenderOptions::default();
        let result = try_render_with_options("not a diagram", opts);
        assert!(result.is_err());
    }

    // ── render_svg() alias ───────────────────────────────────────────────────

    #[test]
    fn render_svg_alias() {
        let svg = render_svg(
            "gantt\n  dateFormat YYYY-MM-DD\n  section A\n  Task1: 2024-01-01, 7d",
            theme::Theme::Default,
        );
        assert!(svg.contains("<svg"));
    }

    // ── all themes produce SVG ───────────────────────────────────────────────

    #[test]
    fn all_themes_render() {
        let input = "flowchart TD\n  A --> B";
        for t in [
            theme::Theme::Default,
            theme::Theme::Dark,
            theme::Theme::Forest,
            theme::Theme::Neutral,
        ] {
            let svg = render(input, t);
            assert!(svg.contains("<svg"));
        }
    }

    // ── leading whitespace is ignored by detect ──────────────────────────────

    #[test]
    fn detect_ignores_leading_whitespace() {
        assert_eq!(detect("   flowchart TD\n  A --> B"), DiagramType::Flowchart);
    }

    // ── sankey with YAML frontmatter ─────────────────────────────────────────

    #[test]
    fn detect_sankey_with_frontmatter() {
        let input = "---\nconfig:\n  sankey:\n    showValues: false\n---\nsankey-beta\n  A,B,10";
        assert_eq!(detect(input), DiagramType::Sankey);
    }
}
