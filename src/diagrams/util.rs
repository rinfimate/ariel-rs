//! Shared SVG utility functions used across all diagram renderers.

/// Format a float for SVG: integer if whole, else trim trailing zeros.
pub fn fmt(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e12 {
        return format!("{}", v as i64);
    }
    let s = format!("{:.13}", v);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

/// XML-escape a string for SVG text content and attribute values.
pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
