//! Font Awesome icon name–to–Unicode codepoint mapping for Mermaid `fa:fa-…` syntax.
//!
//! Mermaid flowchart node labels support `fa:fa-iconname` syntax, e.g.
//! `fa:fa-ban forbidden`.  This module resolves those names to the
//! corresponding Font Awesome 6 Free unicode codepoints so the renderer
//! can emit the glyph directly (with `font-family: "Font Awesome 6 Free"`)
//! rather than showing the literal text.

/// Returns the Unicode codepoint for a Font Awesome icon name, if known.
///
/// The `name` argument may include or omit the `fa-` prefix:
/// `"fa-ban"`, `"fa-spinner"`, `"ban"`, and `"spinner"` all work.
///
/// Returns `None` when the name is not in the built-in table.
pub(crate) fn fa_icon_char(name: &str) -> Option<char> {
    let name = name.strip_prefix("fa-").unwrap_or(name);
    match name {
        "ban" => Some('\u{f05e}'),
        "spinner" => Some('\u{f110}'),
        "check" => Some('\u{f00c}'),
        "times" | "xmark" => Some('\u{f00d}'),
        "home" | "house" => Some('\u{f015}'),
        "user" => Some('\u{f007}'),
        "users" => Some('\u{f0c0}'),
        "cog" | "gear" => Some('\u{f013}'),
        "heart" => Some('\u{f004}'),
        "star" => Some('\u{f005}'),
        "exclamation-triangle" | "triangle-exclamation" => Some('\u{f071}'),
        "info-circle" | "circle-info" => Some('\u{f05a}'),
        "question-circle" | "circle-question" => Some('\u{f059}'),
        "arrow-right" => Some('\u{f061}'),
        "arrow-left" => Some('\u{f060}'),
        "arrow-up" => Some('\u{f062}'),
        "arrow-down" => Some('\u{f063}'),
        "plus" => Some('\u{f067}'),
        "minus" => Some('\u{f068}'),
        "search" | "magnifying-glass" => Some('\u{f002}'),
        "envelope" => Some('\u{f0e0}'),
        "bell" => Some('\u{f0f3}'),
        "lock" => Some('\u{f023}'),
        "unlock" => Some('\u{f09c}'),
        "eye" => Some('\u{f06e}'),
        "eye-slash" => Some('\u{f070}'),
        "trash" | "trash-can" => Some('\u{f1f8}'),
        "edit" | "pen-to-square" => Some('\u{f044}'),
        "download" => Some('\u{f019}'),
        "upload" => Some('\u{f093}'),
        "cloud" => Some('\u{f0c2}'),
        "database" => Some('\u{f1c0}'),
        "server" => Some('\u{f233}'),
        "mobile" | "mobile-screen" => Some('\u{f10b}'),
        "desktop" | "display" => Some('\u{f108}'),
        "laptop" => Some('\u{f109}'),
        "globe" => Some('\u{f0ac}'),
        "link" => Some('\u{f0c1}'),
        "calendar" => Some('\u{f073}'),
        "clock" => Some('\u{f017}'),
        "map-marker" | "location-dot" => Some('\u{f041}'),
        "phone" => Some('\u{f095}'),
        "print" | "printer" => Some('\u{f02f}'),
        "save" | "floppy-disk" => Some('\u{f0c7}'),
        "share" => Some('\u{f064}'),
        "shopping-cart" | "cart-shopping" => Some('\u{f07a}'),
        "tag" => Some('\u{f02b}'),
        "thumbs-up" => Some('\u{f164}'),
        "thumbs-down" => Some('\u{f165}'),
        "chart-bar" | "bar-chart" => Some('\u{f080}'),
        "chart-line" | "line-chart" => Some('\u{f201}'),
        "chart-pie" | "pie-chart" => Some('\u{f200}'),
        "rocket" => Some('\u{f135}'),
        "shield" => Some('\u{f132}'),
        "bolt" | "lightning-bolt" => Some('\u{f0e7}'),
        "fire" => Some('\u{f06d}'),
        "flag" => Some('\u{f024}'),
        "gift" => Some('\u{f06b}'),
        "image" | "picture" => Some('\u{f03e}'),
        "key" => Some('\u{f084}'),
        "map" => Some('\u{f279}'),
        "music" => Some('\u{f001}'),
        "paper-plane" => Some('\u{f1d8}'),
        "puzzle-piece" => Some('\u{f12e}'),
        "robot" => Some('\u{f544}'),
        "sitemap" => Some('\u{f0e8}'),
        "sliders" | "sliders-h" => Some('\u{f1de}'),
        "terminal" => Some('\u{f120}'),
        "truck" => Some('\u{f0d1}'),
        "car" => Some('\u{f1b9}'),
        "bus" => Some('\u{f207}'),
        "bicycle" => Some('\u{f206}'),
        "plane" | "plane-up" => Some('\u{f072}'),
        "ship" => Some('\u{f21a}'),
        "wifi" => Some('\u{f1eb}'),
        "wrench" => Some('\u{f0ad}'),
        _ => None,
    }
}

/// Parse a node label that may contain `fa:fa-iconname text` syntax.
///
/// Returns `(Some(char), remaining_text)` when the label starts with a
/// recognised `fa:fa-iconname` prefix, or `(None, original_label)` when
/// there is no Font Awesome syntax.
///
/// Both forms are supported:
/// - `"fa:fa-ban forbidden"` → `(Some('\u{f05e}'), "forbidden")`
/// - `"fa:fa-spinner"` (icon only) → `(Some('\u{f110}'), "")`
/// - `"ordinary text"` → `(None, "ordinary text")`
pub(crate) fn parse_fa_label(label: &str) -> (Option<char>, &str) {
    let trimmed = label.trim();
    if let Some(rest) = trimmed.strip_prefix("fa:") {
        let (icon_part, text_part) = rest
            .find(|c: char| c.is_whitespace())
            .map(|i| (&rest[..i], rest[i..].trim()))
            .unwrap_or((rest, ""));
        if let Some(ch) = fa_icon_char(icon_part) {
            return (Some(ch), text_part);
        }
        // Unknown icon — strip the fa:fa-xxx prefix, keep the text only.
        // Matches Mermaid JS behaviour: shows remaining text, no literal "fa:fa-xxx".
        return (None, text_part);
    }
    (None, label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_icon_with_prefix() {
        assert_eq!(fa_icon_char("fa-ban"), Some('\u{f05e}'));
    }

    #[test]
    fn known_icon_without_prefix() {
        assert_eq!(fa_icon_char("ban"), Some('\u{f05e}'));
    }

    #[test]
    fn unknown_icon_returns_none() {
        assert_eq!(fa_icon_char("nonexistent-icon"), None);
    }

    #[test]
    fn parse_fa_label_with_text() {
        let (ch, text) = parse_fa_label("fa:fa-ban forbidden");
        assert_eq!(ch, Some('\u{f05e}'));
        assert_eq!(text, "forbidden");
    }

    #[test]
    fn parse_fa_label_icon_only() {
        let (ch, text) = parse_fa_label("fa:fa-spinner");
        assert_eq!(ch, Some('\u{f110}'));
        assert_eq!(text, "");
    }

    #[test]
    fn parse_fa_label_no_fa_syntax() {
        let (ch, text) = parse_fa_label("ordinary text");
        assert_eq!(ch, None);
        assert_eq!(text, "ordinary text");
    }

    #[test]
    fn parse_fa_label_unknown_icon() {
        // Unknown icon strips the fa:fa-xxx prefix, keeps only the remaining text.
        let (ch, text) = parse_fa_label("fa:fa-nonexistent label");
        assert_eq!(ch, None);
        assert_eq!(text, "label");
    }

    #[test]
    fn parse_fa_label_alias_xmark() {
        let (ch, _) = parse_fa_label("fa:fa-xmark");
        assert_eq!(ch, Some('\u{f00d}'));
    }
}
