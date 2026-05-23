/// The colour palette to use when rendering a Mermaid diagram.
///
/// Pass one of these variants to [`crate::render`] or [`crate::render_svg`].
/// `Theme::Default` is the standard Mermaid light theme and is returned by
/// `Default::default()`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Theme {
    /// Standard Mermaid light theme (white background, purple accents).
    #[default]
    Default,
    /// Dark theme suitable for dark-mode UIs.
    Dark,
    /// Forest/green-tinted light theme.
    Forest,
    /// Neutral greyscale theme.
    Neutral,
}

/// Resolved colour and font values for a [`Theme`].
///
/// Obtain an instance by calling [`Theme::resolve`].  All string fields are
/// `'static` CSS colour literals (e.g. `"#ffffff"`).
#[derive(Debug, Clone)]
pub struct ThemeVars {
    /// Diagram canvas background colour.
    pub background: &'static str,
    /// Fill colour for primary nodes.
    pub primary_color: &'static str,
    /// Stroke colour for primary node borders.
    pub primary_border: &'static str,
    /// Text colour inside primary nodes.
    pub primary_text: &'static str,
    /// Fill colour for secondary nodes.
    pub secondary_color: &'static str,
    /// Stroke colour for secondary node borders.
    pub secondary_border: &'static str,
    /// Text colour inside secondary nodes.
    pub secondary_text: &'static str,
    /// Fill colour for tertiary nodes.
    pub tertiary_color: &'static str,
    /// Stroke colour for tertiary node borders.
    pub tertiary_border: &'static str,
    /// Text colour inside tertiary nodes.
    pub tertiary_text: &'static str,
    /// Colour used for connector lines and edges.
    pub line_color: &'static str,
    /// Default text colour for labels that don't belong to a specific node tier.
    pub text_color: &'static str,
    /// Stroke colour for generic node borders.
    pub node_border: &'static str,
    /// Background fill for cluster / subgraph containers.
    pub cluster_bg: &'static str,
    /// Stroke colour for cluster / subgraph borders.
    pub cluster_border: &'static str,
    /// Block diagram container fill (cluster_bg at 50% opacity, `.cluster{fill:rgba(...,0.5)}`).
    pub block_container_fill: &'static str,
    /// Block diagram container stroke (cluster_border at 20% opacity, `.cluster{stroke:rgba(...,0.2)}`).
    pub block_container_stroke: &'static str,
    /// Colour used for diagram titles.
    pub title_color: &'static str,
    /// Background fill for edge labels.
    pub edge_label_bg: &'static str,
    /// Fill colour for note boxes (state diagram, sequence diagram).
    pub note_bg: &'static str,
    /// Stroke colour for note box borders.
    pub note_border: &'static str,
    /// Text colour inside note boxes.
    pub note_text_color: &'static str,
    /// CSS font-family string (e.g. `"Arial, sans-serif"`).
    pub font_family: &'static str,
    /// Base font size in pixels.
    pub font_size: f64,
    /// Venn diagram set colours (venn1–venn8 from Mermaid themeVariables).
    pub venn_colors: &'static [&'static str],
    /// Text colour used inside venn set labels and intersections.
    pub venn_set_text_color: &'static str,
    /// Text colour used for the venn diagram title.
    pub venn_title_text_color: &'static str,
    /// XyChart canvas background colour (matches Mermaid xyChart background per theme).
    pub xychart_bg: &'static str,
    /// XyChart plot colour palette (bar/line series colours, indexed by plot order).
    pub xychart_plot_colors: &'static [&'static str],
    /// XyChart axis line, tick, label, and title colour.
    pub xychart_axis_color: &'static str,
    /// ER diagram attribute row fill for even-indexed rows (0-based).
    pub er_row_fill_even: &'static str,
    /// ER diagram attribute row fill for odd-indexed rows (0-based).
    pub er_row_fill_odd: &'static str,
    /// ER relationship label background (`.relationshipLabelBox` fill per theme).
    pub er_relation_label_bg: &'static str,
    /// Git branch commit/arrow/label-box fill+stroke per branch index (8 entries, cycling).
    pub git_branch_colors: &'static [(&'static str, &'static str)],
    /// Git branch label text colour per branch index (8 entries, cycling).
    pub git_branch_label_text_colors: &'static [&'static str],
    /// Git HIGHLIGHT commit outer rect colour per branch index (8 entries, cycling).
    pub git_highlight_colors: &'static [&'static str],
    /// Git branch spine line colour (dashed line running the branch length).
    pub git_spine_color: &'static str,
    /// Fill colour for the tag-badge hole circle.
    pub git_tag_hole_color: &'static str,
    /// Stroke colour for the tag-badge polygon outline.
    pub git_tag_bkg_stroke: &'static str,
    /// Fill colour for kanban item cards (`.node rect` CSS fill per theme).
    pub kanban_card_bg: &'static str,
    /// Fill colour for the inner area of composite (nested) state nodes (`.composit` CSS).
    pub state_composit_bg: &'static str,
    /// Fill/stroke for state-start circle.
    pub state_start_fill: &'static str,
    /// Inline fill for fork/join bar path (overrides CSS; differs from state_start_fill in some themes).
    pub fork_join_fill: &'static str,
    /// Fill for state-end outer ring.
    pub state_end_fill: &'static str,
    /// Stroke for state-end outer ring and fill for state-end inner dot.
    pub state_end_bg: &'static str,
    /// Stroke colour for sequence actor boxes and lifelines (`.actor{stroke}`, `.actor-line`).
    pub sequence_actor_border: &'static str,
    /// Stroke+fill for sequence loop/alt/par frames and label boxes (`.loopLine`, `.labelBox{stroke}`).
    pub sequence_loop_color: &'static str,
    /// Colour for sequence diagram lines, arrowheads, and autonumber circle (`signalColor` ThemeVar).
    pub signal_color: &'static str,
    /// Stroke colour for state diagram transitions (`.transition{stroke:...}` CSS rule).
    pub state_transition_color: &'static str,

    // ── Gantt diagram section colours ─────────────────────────────────────────
    /// Fill for even-indexed gantt section bands (section0, section2).
    pub gantt_section_fill0: &'static str,
    /// Fill for odd-indexed gantt section bands (section1, section3).
    pub gantt_section_fill1: &'static str,
    /// Fill for weekend/exclusion shading bands.
    pub gantt_exclude_fill: &'static str,

    // ── Requirement diagram (defaultConfig.requirement) ───────────────────────
    /// Fill for requirement/element boxes (`rect_fill`).
    pub requirement_rect_fill: &'static str,
    /// Stroke for box borders (`rect_border_color`).
    pub requirement_rect_border: &'static str,
    /// Stroke width for box borders (`rect_border_size`).
    pub requirement_rect_border_size: &'static str,
    /// Text colour for requirement content (`text_color`).
    pub requirement_text_color: &'static str,

    // ── Mindmap diagram ────────────────────────────────────────────────────────
    /// Fill for the mindmap root node.
    pub mindmap_root_fill: &'static str,
    /// Text colour for the root node label.
    pub mindmap_root_text: &'static str,
    /// Fill colours for branch nodes (11 entries, indexed mod 11).
    pub mindmap_section_fills: &'static [&'static str],
    /// Text colours for branch node labels (11 entries).
    pub mindmap_section_text: &'static [&'static str],
    /// Edge/line colours for branches (11 entries).
    pub mindmap_section_lines: &'static [&'static str],

    // ── Event Modeling (emXXX themeVariables) ─────────────────────────────────
    /// Fill for UI/screen frames (`emUiFill`).
    pub em_ui_fill: &'static str,
    /// Stroke for UI/screen frame borders (`emUiStroke`).
    pub em_ui_stroke: &'static str,
    /// Fill for processor frames (`emProcessorFill`).
    pub em_processor_fill: &'static str,
    /// Stroke for processor frame borders (`emProcessorStroke`).
    pub em_processor_stroke: &'static str,
    /// Fill for read-model frames (`emReadModelFill`).
    pub em_readmodel_fill: &'static str,
    /// Stroke for read-model frame borders (`emReadModelStroke`).
    pub em_readmodel_stroke: &'static str,
    /// Fill for command frames (`emCommandFill`).
    pub em_command_fill: &'static str,
    /// Stroke for command frame borders (`emCommandStroke`).
    pub em_command_stroke: &'static str,
    /// Fill for event frames (`emEventFill`).
    pub em_event_fill: &'static str,
    /// Stroke for event frame borders (`emEventStroke`).
    pub em_event_stroke: &'static str,
    /// Fill for arrowhead polygons (`emArrowhead`).
    pub em_arrowhead: &'static str,
    /// Stroke for relation lines (`emRelationStroke`).
    pub em_relation_stroke: &'static str,
    /// Background fill for swimlane strips (`emSwimlaneBackgroundOdd`).
    pub em_swimlane_bg: &'static str,
    /// Stroke for swimlane strip borders (`emSwimlaneBackgroundStroke`).
    pub em_swimlane_bg_stroke: &'static str,

    // ── Packet diagram (.packetBlock / .packetLabel / .packetByte / .packetTitle CSS) ──
    /// Fill colour for packet field blocks (`blockFillColor`).
    pub packet_block_fill: &'static str,
    /// Stroke colour for packet field block borders (`blockStrokeColor`).
    pub packet_block_stroke: &'static str,
    /// Text colour for packet labels, byte numbers and title (`labelColor` / `startByteColor` / `titleColor`).
    pub packet_text_color: &'static str,
}

impl Theme {
    /// Resolve this theme variant into its concrete [`ThemeVars`] colour set.
    pub fn resolve(self) -> ThemeVars {
        match self {
            Theme::Default => ThemeVars {
                background: "#ffffff",
                primary_color: "#ECECFF",
                primary_border: "#9370DB",
                primary_text: "#333333",
                secondary_color: "#ffffde",
                secondary_border: "#aaaa33",
                secondary_text: "#333333",
                tertiary_color: "#fff0f0",
                tertiary_border: "#ff0000",
                tertiary_text: "#333333",
                line_color: "#333333",
                text_color: "#333333",
                node_border: "#9370DB",
                cluster_bg: "#ffffde",
                cluster_border: "#aaaa33",
                block_container_fill: "rgba(255, 255, 222, 0.5)",
                block_container_stroke: "rgba(170, 170, 51, 0.2)",
                title_color: "#333333",
                edge_label_bg: "rgba(232,232,232,0.8)",
                note_bg: "#fff5ad",
                note_border: "#aaaa33",
                note_text_color: "black",
                font_family: "Arial, sans-serif",
                font_size: 14.0,
                venn_colors: &[
                    "#5353ff", "#ffff45", "#b5ff20", "#ff6b6b", "#9467bd", "#8c564b", "#e377c2",
                    "#7f7f7f",
                ],
                venn_set_text_color: "#333333",
                venn_title_text_color: "#333333",
                xychart_bg: "white",
                xychart_plot_colors: &[
                    "#ECECFF", "#8493A6", "#3949AB", "#00ACC1", "#43A047", "#FB8C00", "#E53935",
                    "#FD79A8", "#636E72", "#FDCB6E",
                ],
                xychart_axis_color: "#131300",
                er_row_fill_even: "#ECECFF",
                er_row_fill_odd: "#ffffff",
                er_relation_label_bg: "rgba(248.6666666666, 255, 235.9999999999, 0.5)",
                git_branch_colors: &[
                    (
                        "hsl(240, 100%, 46.2745098039%)",
                        "hsl(240, 100%, 46.2745098039%)",
                    ),
                    (
                        "hsl(60, 100%, 43.5294117647%)",
                        "hsl(60, 100%, 43.5294117647%)",
                    ),
                    (
                        "hsl(80, 100%, 46.2745098039%)",
                        "hsl(80, 100%, 46.2745098039%)",
                    ),
                    (
                        "hsl(210, 100%, 46.2745098039%)",
                        "hsl(210, 100%, 46.2745098039%)",
                    ),
                    (
                        "hsl(180, 100%, 46.2745098039%)",
                        "hsl(180, 100%, 46.2745098039%)",
                    ),
                    (
                        "hsl(150, 100%, 46.2745098039%)",
                        "hsl(150, 100%, 46.2745098039%)",
                    ),
                    (
                        "hsl(300, 100%, 46.2745098039%)",
                        "hsl(300, 100%, 46.2745098039%)",
                    ),
                    (
                        "hsl(0, 100%, 46.2745098039%)",
                        "hsl(0, 100%, 46.2745098039%)",
                    ),
                ],
                git_branch_label_text_colors: &[
                    "#ffffff", "black", "black", "#ffffff", "black", "black", "black", "black",
                ],
                git_highlight_colors: &[
                    "hsl(60,100%,3.7254901961%)",
                    "rgb(0,0,160.5)",
                    "rgb(48.8333333334,0,146.5000000001)",
                    "rgb(146.5000000001,73.2500000001,0)",
                    "rgb(146.5000000001,0,0)",
                    "rgb(146.5000000001,0,73.2500000001)",
                    "rgb(0,146.5000000001,0)",
                    "rgb(0,146.5000000001,146.5000000001)",
                ],
                git_spine_color: "#333333",
                git_tag_hole_color: "#333",
                git_tag_bkg_stroke: "hsl(240, 60%, 86.2745098039%)",
                kanban_card_bg: "#ffffff",
                state_composit_bg: "white",
                state_start_fill: "#333333",
                fork_join_fill: "#333333",
                state_end_fill: "#ECECFF",
                state_end_bg: "#9370DB",
                sequence_actor_border: "#9370DB",
                sequence_loop_color: "#9370DB",
                signal_color: "#333333",
                state_transition_color: "#333333",
                gantt_section_fill0: "rgba(102,102,255,0.49)",
                gantt_section_fill1: "#fff400",
                gantt_exclude_fill: "#eeeeee",
                requirement_rect_fill: "#f9f9f9",
                requirement_rect_border: "#bbb",
                requirement_rect_border_size: "0.5px",
                requirement_text_color: "#333",
                mindmap_root_fill: "hsl(240, 100%, 46.2745098039%)",
                mindmap_root_text: "#ffffff",
                mindmap_section_fills: &[
                    "hsl(60, 100%, 73.5294117647%)",
                    "hsl(80, 100%, 76.2745098039%)",
                    "hsl(270, 100%, 76.2745098039%)",
                    "hsl(300, 100%, 76.2745098039%)",
                    "hsl(330, 100%, 76.2745098039%)",
                    "hsl(0, 100%, 76.2745098039%)",
                    "hsl(30, 100%, 76.2745098039%)",
                    "hsl(90, 100%, 76.2745098039%)",
                    "hsl(150, 100%, 76.2745098039%)",
                    "hsl(180, 100%, 76.2745098039%)",
                    "hsl(210, 100%, 76.2745098039%)",
                ],
                mindmap_section_text: &[
                    "black", "black", "#ffffff", "black", "black", "black", "black", "black",
                    "black", "black", "black",
                ],
                mindmap_section_lines: &[
                    "hsl(240, 100%, 83.5294117647%)",
                    "hsl(260, 100%, 86.2745098039%)",
                    "hsl(90, 100%, 86.2745098039%)",
                    "hsl(120, 100%, 86.2745098039%)",
                    "hsl(150, 100%, 86.2745098039%)",
                    "hsl(180, 100%, 86.2745098039%)",
                    "hsl(210, 100%, 86.2745098039%)",
                    "hsl(270, 100%, 86.2745098039%)",
                    "hsl(330, 100%, 86.2745098039%)",
                    "hsl(0, 100%, 86.2745098039%)",
                    "hsl(30, 100%, 86.2745098039%)",
                ],
                em_ui_fill: "white",
                em_ui_stroke: "#dbdada",
                em_processor_fill: "#edb3f6",
                em_processor_stroke: "#b88cbf",
                em_readmodel_fill: "#d3f1a2",
                em_readmodel_stroke: "#a3b732",
                em_command_fill: "#bcd6fe",
                em_command_stroke: "#679ac3",
                em_event_fill: "#ffb778",
                em_event_stroke: "#c19a0f",
                em_arrowhead: "#333333",
                em_relation_stroke: "#333333",
                em_swimlane_bg: "rgb(250,250,250)",
                em_swimlane_bg_stroke: "rgb(240,240,240)",
                packet_block_fill: "#efefef",
                packet_block_stroke: "black",
                packet_text_color: "black",
            },
            Theme::Dark => ThemeVars {
                background: "#1e1e1e",
                primary_color: "#1f2020",
                primary_border: "#cccccc",
                primary_text: "#ccc",
                secondary_color: "#323232",
                secondary_border: "#cccccc",
                secondary_text: "#ccc",
                tertiary_color: "#3a3a3a",
                tertiary_border: "#cccccc",
                tertiary_text: "#ccc",
                line_color: "lightgrey",
                text_color: "#ccc",
                node_border: "#cccccc",
                // hsl(180, 1.5873%, 28.3529%) = Mermaid dark .cluster rect fill
                cluster_bg: "hsl(180,1.5873015873%,28.3529411765%)",
                cluster_border: "rgba(255,255,255,0.25)",
                block_container_fill: "rgba(71, 73, 73, 0.5)",
                block_container_stroke: "rgba(255, 255, 255, 0.2)",
                title_color: "#F9FFFE",
                edge_label_bg: "hsl(0, 0%, 34.4117647059%)",
                // hsl(180, 1.5873%, 28.3529%) ≈ Mermaid dark note fill
                note_bg: "#474848",
                // hsl(180, 0%, 18.3529%) ≈ Mermaid dark note border
                note_border: "#2f2f2f",
                note_text_color: "#ccc",
                font_family: "Arial, sans-serif",
                font_size: 14.0,
                venn_colors: &[
                    "hsl(180, 1.5873015873%, 42.3529411765%)",
                    "hsl(0, 100%, 32.1568627451%)",
                    "hsl(321.6393442623, 65.5913978495%, 48.2352941176%)",
                    "hsl(194.4, 16.5562913907%, 59.6078431373%)",
                    "hsl(23.0769230769, 49.0566037736%, 50.7843137255%)",
                    "hsl(0, 83.3333333333%, 53.5294117647%)",
                    "hsl(289.1666666667, 100%, 44.1176470588%)",
                    "hsl(35.1315789474, 98.7012987013%, 60.1960784314%)",
                ],
                venn_set_text_color: "#ccc",
                venn_title_text_color: "#F9FFFE",
                xychart_bg: "#333",
                xychart_plot_colors: &[
                    "#3498db", "#2ecc71", "#e74c3c", "#f39c12", "#9b59b6", "#1abc9c", "#e67e22",
                    "#ecf0f1", "#95a5a6", "#f1c40f",
                ],
                xychart_axis_color: "#e0dfdf",
                er_row_fill_even: "hsl(180,1.5873015873%,2.3529411765%)",
                er_row_fill_odd: "hsl(180,1.5873015873%,17.3529411765%)",
                er_relation_label_bg: "rgba(32.0000000001, 31.3333333334, 31.0000000001, 0.5)",
                git_branch_colors: &[
                    (
                        "hsl(180, 1.5873015873%, 48.3529411765%)",
                        "hsl(180, 1.5873015873%, 48.3529411765%)",
                    ),
                    (
                        "hsl(321.6393442623, 65.5913978495%, 38.2352941176%)",
                        "hsl(321.6393442623, 65.5913978495%, 38.2352941176%)",
                    ),
                    (
                        "hsl(194.4, 16.5562913907%, 49.6078431373%)",
                        "hsl(194.4, 16.5562913907%, 49.6078431373%)",
                    ),
                    (
                        "hsl(23.0769230769, 49.0566037736%, 40.7843137255%)",
                        "hsl(23.0769230769, 49.0566037736%, 40.7843137255%)",
                    ),
                    (
                        "hsl(0, 83.3333333333%, 43.5294117647%)",
                        "hsl(0, 83.3333333333%, 43.5294117647%)",
                    ),
                    (
                        "hsl(289.1666666667, 100%, 24.1176470588%)",
                        "hsl(289.1666666667, 100%, 24.1176470588%)",
                    ),
                    (
                        "hsl(35.1315789474, 98.7012987013%, 40.1960784314%)",
                        "hsl(35.1315789474, 98.7012987013%, 40.1960784314%)",
                    ),
                    (
                        "hsl(106.1538461538, 84.4155844156%, 35.0980392157%)",
                        "hsl(106.1538461538, 84.4155844156%, 35.0980392157%)",
                    ),
                ],
                git_branch_label_text_colors: &[
                    "#2c2c2c",
                    "lightgrey",
                    "lightgrey",
                    "#2c2c2c",
                    "lightgrey",
                    "lightgrey",
                    "lightgrey",
                    "lightgrey",
                ],
                git_highlight_colors: &[
                    "rgb(133.6571428571,129.7428571428,129.7428571428)",
                    "rgb(93.5483870969,221.4516129033,139.677419355)",
                    "rgb(149.4437086091,117.6092715231,107.5562913906)",
                    "rgb(99.9811320754,162.7735849057,202.0188679245)",
                    "rgb(51.5000000001,236.5,236.5)",
                    "rgb(154.2083333334,255,132.0000000001)",
                    "rgb(51.331168831,135.1948051946,253.6688311688)",
                    "rgb(206.1818181817,89.948051948,241.051948052)",
                ],
                git_spine_color: "lightgrey",
                git_tag_hole_color: "#ccc",
                git_tag_bkg_stroke: "#cccccc",
                kanban_card_bg: "#333",
                state_composit_bg: "#333",
                state_start_fill: "#f4f4f4",
                fork_join_fill: "lightgrey",
                state_end_fill: "#1f2020",
                state_end_bg: "#cccccc",
                sequence_actor_border: "#ccc",
                sequence_loop_color: "#ccc",
                signal_color: "lightgrey",
                state_transition_color: "lightgrey",
                gantt_section_fill0: "hsl(52.9411764706,28.813559322%,58.431372549%)",
                gantt_section_fill1: "#EAE8D9",
                gantt_exclude_fill: "hsl(52.9411764706,28.813559322%,48.431372549%)",
                requirement_rect_fill: "#f9f9f9",
                requirement_rect_border: "#bbb",
                requirement_rect_border_size: "0.5px",
                requirement_text_color: "#333",
                mindmap_root_fill: "hsl(180, 1.5873015873%, 48.3529411765%)",
                mindmap_root_text: "#ccc",
                mindmap_section_fills: &[
                    "hsl(60, 17.6470588235%, 58.431372549%)",
                    "hsl(80, 17.6470588235%, 60%)",
                    "hsl(270, 17.6470588235%, 60%)",
                    "hsl(300, 17.6470588235%, 60%)",
                    "hsl(330, 17.6470588235%, 60%)",
                    "hsl(0, 17.6470588235%, 60%)",
                    "hsl(30, 17.6470588235%, 60%)",
                    "hsl(90, 17.6470588235%, 60%)",
                    "hsl(150, 17.6470588235%, 60%)",
                    "hsl(180, 17.6470588235%, 60%)",
                    "hsl(210, 17.6470588235%, 60%)",
                ],
                mindmap_section_text: &[
                    "#ccc", "#ccc", "#ccc", "#ccc", "#ccc", "#ccc", "#ccc", "#ccc", "#ccc", "#ccc",
                    "#ccc",
                ],
                mindmap_section_lines: &[
                    "hsl(180, 1.5873015873%, 38.3529411765%)",
                    "hsl(200, 1.5873015873%, 40%)",
                    "hsl(110, 1.5873015873%, 40%)",
                    "hsl(140, 1.5873015873%, 40%)",
                    "hsl(170, 1.5873015873%, 40%)",
                    "hsl(200, 1.5873015873%, 40%)",
                    "hsl(230, 1.5873015873%, 40%)",
                    "hsl(290, 1.5873015873%, 40%)",
                    "hsl(350, 1.5873015873%, 40%)",
                    "hsl(20, 1.5873015873%, 40%)",
                    "hsl(50, 1.5873015873%, 40%)",
                ],
                em_ui_fill: "#2d2d2d",
                em_ui_stroke: "#555",
                em_processor_fill: "hsl(296.1290322581, 20.2614379085%, 40%)",
                em_processor_stroke: "#8a6d8c",
                em_readmodel_fill: "hsl(98.6666666667, 33.3333333333%, 36.4705882353%)",
                em_readmodel_stroke: "#6d8c5c",
                em_command_fill: "hsl(218.6666666667, 33.3333333333%, 36.4705882353%)",
                em_command_stroke: "#5c6d8c",
                em_event_fill: "hsl(32, 33.3333333333%, 36.4705882353%)",
                em_event_stroke: "#8c755c",
                em_arrowhead: "lightgrey",
                em_relation_stroke: "lightgrey",
                em_swimlane_bg: "hsl(0, 0%, 25%)",
                em_swimlane_bg_stroke: "hsl(0, 0%, 32%)",
                packet_block_fill: "#333",
                packet_block_stroke: "#e0dfdf",
                packet_text_color: "#e0dfdf",
            },
            Theme::Forest => ThemeVars {
                background: "#ffffff",
                primary_color: "#cde498",
                primary_border: "#13540c",
                primary_text: "#333333",
                secondary_color: "#cdffb2",
                secondary_border: "#6eaa49",
                secondary_text: "#333333",
                tertiary_color: "#fff",
                tertiary_border: "#13540c",
                tertiary_text: "#333333",
                line_color: "#000000",
                text_color: "#000000",
                node_border: "#13540c",
                cluster_bg: "#cdffb2",
                cluster_border: "#6eaa49",
                block_container_fill: "rgba(205, 255, 178, 0.5)",
                block_container_stroke: "rgba(110, 170, 73, 0.2)",
                title_color: "#333333",
                edge_label_bg: "#e8e8e8",
                note_bg: "#fff5ad",
                note_border: "#6eaa49",
                note_text_color: "black",
                font_family: "Arial, sans-serif",
                font_size: 14.0,
                venn_colors: &[
                    "hsl(78.1578947368, 58.4615384615%, 44.5098039216%)",
                    "hsl(98.961038961, 100%, 54.9019607843%)",
                    "hsl(78.1578947368, 58.4615384615%, 54.5098039216%)",
                    "hsl(138.1578947368, 58.4615384615%, 44.5098039216%)",
                    "hsl(18.1578947368, 58.4615384615%, 44.5098039216%)",
                    "hsl(158.961038961, 100%, 54.9019607843%)",
                    "hsl(198.1578947368, 58.4615384615%, 44.5098039216%)",
                    "hsl(218.961038961, 100%, 54.9019607843%)",
                ],
                venn_set_text_color: "#333333",
                venn_title_text_color: "#333333",
                xychart_bg: "white",
                xychart_plot_colors: &[
                    "#CDE498", "#FF6B6B", "#13540c", "#6eaa49", "#cdffb2", "#321b67", "#e377c2",
                    "#7f7f7f", "#9467bd", "#8c564b",
                ],
                xychart_axis_color: "#321b67",
                er_row_fill_even: "hsl(78.1578947368,58.4615384615%,94.5098039216%)",
                er_row_fill_odd: "#ffffff",
                er_relation_label_bg: "rgba(224.6153846155, 238.5923076923, 192.4076923078, 0.5)",
                git_branch_colors: &[
                    (
                        "hsl(78.1578947368, 58.4615384615%, 49.5098039216%)",
                        "hsl(78.1578947368, 58.4615384615%, 49.5098039216%)",
                    ),
                    (
                        "hsl(98.961038961, 100%, 59.9019607843%)",
                        "hsl(98.961038961, 100%, 59.9019607843%)",
                    ),
                    (
                        "hsl(78.1578947368, 58.4615384615%, 59.5098039216%)",
                        "hsl(78.1578947368, 58.4615384615%, 59.5098039216%)",
                    ),
                    (
                        "hsl(48.1578947368, 58.4615384615%, 49.5098039216%)",
                        "hsl(48.1578947368, 58.4615384615%, 49.5098039216%)",
                    ),
                    (
                        "hsl(18.1578947368, 58.4615384615%, 49.5098039216%)",
                        "hsl(18.1578947368, 58.4615384615%, 49.5098039216%)",
                    ),
                    (
                        "hsl(-11.8421052632, 58.4615384615%, 49.5098039216%)",
                        "hsl(-11.8421052632, 58.4615384615%, 49.5098039216%)",
                    ),
                    (
                        "hsl(138.1578947368, 58.4615384615%, 49.5098039216%)",
                        "hsl(138.1578947368, 58.4615384615%, 49.5098039216%)",
                    ),
                    (
                        "hsl(198.1578947368, 58.4615384615%, 49.5098039216%)",
                        "hsl(198.1578947368, 58.4615384615%, 49.5098039216%)",
                    ),
                ],
                git_branch_label_text_colors: &[
                    "#ffffff", "black", "black", "#ffffff", "black", "black", "black", "black",
                ],
                git_highlight_colors: &[
                    "rgb(99.6153846152,54.9423076922,202.5576923076)",
                    "rgb(132.7922077921,0,204.5000000001)",
                    "rgb(79.4230769229,42.8884615385,163.6115384614)",
                    "rgb(54.9423076922,84.0769230769,202.5576923076)",
                    "rgb(54.9423076922,157.8846153846,202.5576923076)",
                    "rgb(54.9423076922,202.5576923076,173.4230769229)",
                    "rgb(202.5576923076,54.9423076922,157.8846153846)",
                    "rgb(202.5576923076,99.6153846152,54.9423076922)",
                ],
                git_spine_color: "#000000",
                git_tag_hole_color: "#000000",
                git_tag_bkg_stroke: "hsl(78.1578947368, 18.4615384615%, 64.5098039216%)",
                kanban_card_bg: "#ffffff",
                state_composit_bg: "white",
                state_start_fill: "#000000",
                fork_join_fill: "#000000",
                state_end_fill: "hsl(78.1578947368, 18.4615384615%, 64.5098039216%)",
                state_end_bg: "#13540c",
                sequence_actor_border: "hsl(78.1578947368, 58.4615384615%, 54.5098039216%)",
                sequence_loop_color: "#326932",
                signal_color: "#333333",
                state_transition_color: "#000000",
                gantt_section_fill0: "#6eaa49",
                gantt_section_fill1: "#6eaa49",
                gantt_exclude_fill: "#eeeeee",
                requirement_rect_fill: "#f9f9f9",
                requirement_rect_border: "#bbb",
                requirement_rect_border_size: "0.5px",
                requirement_text_color: "#333",
                mindmap_root_fill: "hsl(78.1578947368, 58.4615384615%, 49.5098039216%)",
                mindmap_root_text: "#ffffff",
                mindmap_section_fills: &[
                    "hsl(98.961038961, 100%, 74.9019607843%)",
                    "hsl(98.961038961, 80%, 76.2745098039%)",
                    "hsl(270, 58.4615384615%, 76.2745098039%)",
                    "hsl(300, 58.4615384615%, 76.2745098039%)",
                    "hsl(330, 58.4615384615%, 76.2745098039%)",
                    "hsl(0, 58.4615384615%, 76.2745098039%)",
                    "hsl(30, 58.4615384615%, 76.2745098039%)",
                    "hsl(90, 58.4615384615%, 76.2745098039%)",
                    "hsl(150, 58.4615384615%, 76.2745098039%)",
                    "hsl(180, 58.4615384615%, 76.2745098039%)",
                    "hsl(210, 58.4615384615%, 76.2745098039%)",
                ],
                mindmap_section_text: &[
                    "black", "black", "#ffffff", "black", "black", "black", "black", "black",
                    "black", "black", "black",
                ],
                mindmap_section_lines: &[
                    "hsl(78.1578947368, 58.4615384615%, 83.5294117647%)",
                    "hsl(98.961038961, 58.4615384615%, 86.2745098039%)",
                    "hsl(90, 58.4615384615%, 86.2745098039%)",
                    "hsl(120, 58.4615384615%, 86.2745098039%)",
                    "hsl(150, 58.4615384615%, 86.2745098039%)",
                    "hsl(180, 58.4615384615%, 86.2745098039%)",
                    "hsl(210, 58.4615384615%, 86.2745098039%)",
                    "hsl(270, 58.4615384615%, 86.2745098039%)",
                    "hsl(330, 58.4615384615%, 86.2745098039%)",
                    "hsl(0, 58.4615384615%, 86.2745098039%)",
                    "hsl(30, 58.4615384615%, 86.2745098039%)",
                ],
                em_ui_fill: "white",
                em_ui_stroke: "#dbdada",
                em_processor_fill: "#edb3f6",
                em_processor_stroke: "#b88cbf",
                em_readmodel_fill: "#d3f1a2",
                em_readmodel_stroke: "#a3b732",
                em_command_fill: "#bcd6fe",
                em_command_stroke: "#679ac3",
                em_event_fill: "#ffb778",
                em_event_stroke: "#c19a0f",
                em_arrowhead: "#000000",
                em_relation_stroke: "#000000",
                em_swimlane_bg: "rgb(250,250,250)",
                em_swimlane_bg_stroke: "rgb(240,240,240)",
                packet_block_fill: "#cde498",
                packet_block_stroke: "#321b67",
                packet_text_color: "#321b67",
            },
            Theme::Neutral => ThemeVars {
                background: "#ffffff",
                primary_color: "#eee",
                primary_border: "#999",
                primary_text: "#333333",
                secondary_color: "#f4f4f4",
                secondary_border: "#999",
                secondary_text: "#333333",
                tertiary_color: "#fff",
                tertiary_border: "#999",
                tertiary_text: "#333333",
                line_color: "#666",
                text_color: "#000000",
                node_border: "#000",
                cluster_bg: "hsl(0, 0%, 98.9215686275%)",
                cluster_border: "#707070",
                block_container_fill: "rgba(252, 252, 252, 0.5)",
                block_container_stroke: "rgba(112, 112, 112, 0.2)",
                title_color: "#333333",
                edge_label_bg: "white",
                note_bg: "#666",
                note_border: "#999",
                note_text_color: "#fff",
                font_family: "Arial, sans-serif",
                font_size: 14.0,
                venn_colors: &[
                    "#555", "#F4F4F4", "#555", "#BBB", "#777", "#999", "#DDD", "#FFF",
                ],
                venn_set_text_color: "#333333",
                venn_title_text_color: "#333333",
                xychart_bg: "#ffffff",
                xychart_plot_colors: &[
                    "#EEE", "#6BB8E4", "#999", "#f4f4f4", "#333333", "#9467bd", "#8c564b",
                    "#e377c2", "#7f7f7f", "#FDCB6E",
                ],
                xychart_axis_color: "#111111",
                er_row_fill_even: "#f4f4f4",
                er_row_fill_odd: "#ffffff",
                er_relation_label_bg: "rgba(237.9999999999, 237.9999999999, 237.9999999999, 0.5)",
                git_branch_colors: &[
                    ("hsl(0, 0%, 70.6862745098%)", "hsl(0, 0%, 70.6862745098%)"),
                    ("#555", "#555"),
                    ("#BBB", "#BBB"),
                    ("#777", "#777"),
                    ("#999", "#999"),
                    ("#DDD", "#DDD"),
                    ("#FFF", "#FFF"),
                    ("#DDD", "#DDD"),
                ],
                git_branch_label_text_colors: &[
                    "#333", "white", "#333", "white", "#333", "#333", "#333", "#333",
                ],
                git_highlight_colors: &[
                    "rgb(74.75,74.75,74.75)",
                    "#aaaaaa",
                    "#444444",
                    "#888888",
                    "#666666",
                    "#222222",
                    "#000000",
                    "#222222",
                ],
                git_spine_color: "#666",
                git_tag_hole_color: "#000000",
                git_tag_bkg_stroke: "hsl(0, 0%, 83.3333333333%)",
                kanban_card_bg: "#ffffff",
                state_composit_bg: "#ffffff",
                state_start_fill: "#222",
                fork_join_fill: "#666",
                state_end_fill: "#eee",
                state_end_bg: "#000",
                sequence_actor_border: "hsl(0, 0%, 83%)",
                sequence_loop_color: "hsl(0, 0%, 83%)",
                signal_color: "#333333",
                state_transition_color: "#000000",
                gantt_section_fill0: "hsl(0,0%,73.9215686275%)",
                gantt_section_fill1: "hsl(0,0%,73.9215686275%)",
                gantt_exclude_fill: "#eeeeee",
                requirement_rect_fill: "#f9f9f9",
                requirement_rect_border: "#bbb",
                requirement_rect_border_size: "0.5px",
                requirement_text_color: "#333",
                mindmap_root_fill: "hsl(0, 0%, 70.6862745098%)",
                mindmap_root_text: "#333333",
                mindmap_section_fills: &[
                    "hsl(0, 0%, 83.5294117647%)",
                    "hsl(0, 0%, 86.2745098039%)",
                    "hsl(0, 0%, 86.2745098039%)",
                    "hsl(0, 0%, 86.2745098039%)",
                    "hsl(0, 0%, 86.2745098039%)",
                    "hsl(0, 0%, 86.2745098039%)",
                    "hsl(0, 0%, 86.2745098039%)",
                    "hsl(0, 0%, 86.2745098039%)",
                    "hsl(0, 0%, 86.2745098039%)",
                    "hsl(0, 0%, 86.2745098039%)",
                    "hsl(0, 0%, 86.2745098039%)",
                ],
                mindmap_section_text: &[
                    "#333333", "#333333", "#333333", "#333333", "#333333", "#333333", "#333333",
                    "#333333", "#333333", "#333333", "#333333",
                ],
                mindmap_section_lines: &[
                    "hsl(0, 0%, 60%)",
                    "hsl(0, 0%, 60%)",
                    "hsl(0, 0%, 60%)",
                    "hsl(0, 0%, 60%)",
                    "hsl(0, 0%, 60%)",
                    "hsl(0, 0%, 60%)",
                    "hsl(0, 0%, 60%)",
                    "hsl(0, 0%, 60%)",
                    "hsl(0, 0%, 60%)",
                    "hsl(0, 0%, 60%)",
                    "hsl(0, 0%, 60%)",
                ],
                em_ui_fill: "white",
                em_ui_stroke: "#dbdada",
                em_processor_fill: "#edb3f6",
                em_processor_stroke: "#b88cbf",
                em_readmodel_fill: "#d3f1a2",
                em_readmodel_stroke: "#a3b732",
                em_command_fill: "#bcd6fe",
                em_command_stroke: "#679ac3",
                em_event_fill: "#ffb778",
                em_event_stroke: "#c19a0f",
                em_arrowhead: "#666",
                em_relation_stroke: "#666",
                em_swimlane_bg: "rgb(250,250,250)",
                em_swimlane_bg_stroke: "rgb(240,240,240)",
                packet_block_fill: "#efefef",
                packet_block_stroke: "black",
                packet_text_color: "black",
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_has_white_bg() {
        let vars = Theme::Default.resolve();
        assert_eq!(vars.background, "#ffffff");
    }

    #[test]
    fn dark_theme_has_dark_bg() {
        let vars = Theme::Dark.resolve();
        assert_eq!(vars.background, "#1e1e1e");
    }

    #[test]
    fn all_themes_resolve() {
        for t in [Theme::Default, Theme::Dark, Theme::Forest, Theme::Neutral] {
            let v = t.resolve();
            assert!(!v.font_family.is_empty());
            assert!(v.font_size > 0.0);
        }
    }
}
