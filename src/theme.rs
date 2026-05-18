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
    /// Colour used for diagram titles.
    pub title_color: &'static str,
    /// Background fill for edge labels.
    pub edge_label_bg: &'static str,
    /// CSS font-family string (e.g. `"Arial, sans-serif"`).
    pub font_family: &'static str,
    /// Base font size in pixels.
    pub font_size: f64,
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
                title_color: "#333333",
                edge_label_bg: "#ffffff",
                font_family: "Arial, sans-serif",
                font_size: 14.0,
            },
            Theme::Dark => ThemeVars {
                background: "#1e1e1e",
                primary_color: "#1f2020",
                primary_border: "#81B1DB",
                primary_text: "#ccc",
                secondary_color: "#323232",
                secondary_border: "#81B1DB",
                secondary_text: "#ccc",
                tertiary_color: "#3a3a3a",
                tertiary_border: "#81B1DB",
                tertiary_text: "#ccc",
                line_color: "#81B1DB",
                text_color: "#ccc",
                node_border: "#81B1DB",
                cluster_bg: "#323232",
                cluster_border: "#81B1DB",
                title_color: "#F9FFFE",
                edge_label_bg: "#323232",
                font_family: "Arial, sans-serif",
                font_size: 14.0,
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
                line_color: "#333333",
                text_color: "#333333",
                node_border: "#13540c",
                cluster_bg: "#cdffb2",
                cluster_border: "#6eaa49",
                title_color: "#333333",
                edge_label_bg: "#ffffff",
                font_family: "Arial, sans-serif",
                font_size: 14.0,
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
                line_color: "#999",
                text_color: "#333333",
                node_border: "#999",
                cluster_bg: "#f4f4f4",
                cluster_border: "#999",
                title_color: "#333333",
                edge_label_bg: "#ffffff",
                font_family: "Arial, sans-serif",
                font_size: 14.0,
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
