/// A lightweight SVG string builder.
///
/// Replaces manual `String::push_str()` chains with a composable API
/// that tracks nesting and reduces intermediate allocations.
pub struct SvgWriter {
    buf: String,
}

impl SvgWriter {
    /// Create a new empty writer with optional pre-allocated capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buf: String::with_capacity(cap),
        }
    }

    /// Create a new empty writer.
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Push a raw string slice — use sparingly; prefer typed methods.
    pub fn raw(&mut self, s: &str) -> &mut Self {
        self.buf.push_str(s);
        self
    }

    /// Open an SVG element with attributes, write children, close it.
    #[allow(dead_code)]
    pub fn elem(
        &mut self,
        tag: &str,
        attrs: &[(&str, &str)],
        children: impl FnOnce(&mut Self),
    ) -> &mut Self {
        self.buf.push('<');
        self.buf.push_str(tag);
        for (k, v) in attrs {
            self.buf.push(' ');
            self.buf.push_str(k);
            self.buf.push_str("=\"");
            self.buf.push_str(v);
            self.buf.push('"');
        }
        self.buf.push('>');
        children(self);
        self.buf.push_str("</");
        self.buf.push_str(tag);
        self.buf.push('>');
        self
    }

    /// Write a self-closing element: `<tag attr="val"/>`.
    #[allow(dead_code)]
    pub fn void_elem(&mut self, tag: &str, attrs: &[(&str, &str)]) -> &mut Self {
        self.buf.push('<');
        self.buf.push_str(tag);
        for (k, v) in attrs {
            self.buf.push(' ');
            self.buf.push_str(k);
            self.buf.push_str("=\"");
            self.buf.push_str(v);
            self.buf.push('"');
        }
        self.buf.push_str("/>");
        self
    }

    /// Consume the writer and return the accumulated SVG string.
    pub fn finish(self) -> String {
        self.buf
    }
}

impl Default for SvgWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Write for SvgWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.push_str(s);
        Ok(())
    }
}

/// Convert HTML named entities to their Unicode equivalents.
/// SVG is XML and only supports &amp; &lt; &gt; &apos; &quot; natively.
/// All other HTML entities must be converted to Unicode characters.
#[allow(dead_code)]
pub fn html_entities_to_unicode(s: &str) -> String {
    // Common HTML entities used in Mermaid diagram labels
    static ENTITIES: &[(&str, &str)] = &[
        ("&laquo;", "\u{00AB}"),  // «
        ("&raquo;", "\u{00BB}"),  // »
        ("&lsaquo;", "\u{2039}"), // ‹
        ("&rsaquo;", "\u{203A}"), // ›
        ("&nbsp;", "\u{00A0}"),   // non-breaking space
        ("&mdash;", "\u{2014}"),  // —
        ("&ndash;", "\u{2013}"),  // –
        ("&hellip;", "\u{2026}"), // …
        ("&copy;", "\u{00A9}"),   // ©
        ("&reg;", "\u{00AE}"),    // ®
        ("&trade;", "\u{2122}"),  // ™
        ("&deg;", "\u{00B0}"),    // °
        ("&plusmn;", "\u{00B1}"), // ±
        ("&times;", "\u{00D7}"),  // ×
        ("&divide;", "\u{00F7}"), // ÷
        ("&frac12;", "\u{00BD}"), // ½
        ("&frac14;", "\u{00BC}"), // ¼
        ("&frac34;", "\u{00BE}"), // ¾
        ("&alpha;", "\u{03B1}"),  // α
        ("&beta;", "\u{03B2}"),   // β
        ("&gamma;", "\u{03B3}"),  // γ
        ("&delta;", "\u{03B4}"),  // δ
        ("&pi;", "\u{03C0}"),     // π
        ("&sigma;", "\u{03C3}"),  // σ
        ("&mu;", "\u{03BC}"),     // μ
        ("&Omega;", "\u{03A9}"),  // Ω
        ("&larr;", "\u{2190}"),   // ←
        ("&rarr;", "\u{2192}"),   // →
        ("&uarr;", "\u{2191}"),   // ↑
        ("&darr;", "\u{2193}"),   // ↓
        ("&harr;", "\u{2194}"),   // ↔
        ("&check;", "\u{2713}"),  // ✓
        ("&cross;", "\u{2717}"),  // ✗
        ("&bull;", "\u{2022}"),   // •
        ("&prime;", "\u{2032}"),  // ′
        ("&infin;", "\u{221E}"),  // ∞
        ("&ne;", "\u{2260}"),     // ≠
        ("&le;", "\u{2264}"),     // ≤
        ("&ge;", "\u{2265}"),     // ≥
        ("&asymp;", "\u{2248}"),  // ≈
                                  // XML builtins — leave as-is (SVG handles these natively)
                                  // &amp; &lt; &gt; &quot; &apos; — do NOT replace these
    ];

    let mut result = s.to_string();
    for (entity, unicode) in ENTITIES {
        result = result.replace(entity, unicode);
    }
    result
}

// ── Rounded-corner polyline path ─────────────────────────────────────────────

/// Render waypoints as straight lines with rounded corners at each bend.
/// Interior waypoints get a small cubic bezier arc of radius `r` instead of
/// a sharp angle, keeping all straight segments but smoothing direction changes.
#[allow(dead_code)]
pub fn rounded_path(pts: &[(f64, f64)], r: f64) -> String {
    let n = pts.len();
    if n == 0 {
        return String::new();
    }
    if n == 1 {
        return format!("M{:.3},{:.3}", pts[0].0, pts[0].1);
    }
    if n == 2 {
        return format!(
            "M{:.3},{:.3}L{:.3},{:.3}",
            pts[0].0, pts[0].1, pts[1].0, pts[1].1
        );
    }

    let unit = |dx: f64, dy: f64| -> (f64, f64) {
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-9 {
            (0.0, 0.0)
        } else {
            (dx / len, dy / len)
        }
    };

    let mut d = format!("M{:.3},{:.3}", pts[0].0, pts[0].1);

    for i in 1..n - 1 {
        let (ax, ay) = pts[i - 1];
        let (bx, by) = pts[i];
        let (cx, cy) = pts[i + 1];

        // Direction vectors into and out of the corner
        let (ux, uy) = unit(bx - ax, by - ay);
        let (vx, vy) = unit(cx - bx, cy - by);

        // Distance to corner limited by half the shorter segment
        let in_len = ((bx - ax) * (bx - ax) + (by - ay) * (by - ay)).sqrt();
        let out_len = ((cx - bx) * (cx - bx) + (cy - by) * (cy - by)).sqrt();
        let cr = r.min(in_len / 2.0).min(out_len / 2.0);

        // Approach point (on incoming segment, r before corner)
        let p1x = bx - ux * cr;
        let p1y = by - uy * cr;
        // Departure point (on outgoing segment, r after corner)
        let p2x = bx + vx * cr;
        let p2y = by + vy * cr;

        // Line to approach point, then cubic bezier through corner
        d.push_str(&format!(
            "L{:.3},{:.3}C{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
            p1x,
            p1y,
            bx,
            by, // CP1 = corner
            bx,
            by, // CP2 = corner
            p2x,
            p2y,
        ));
    }

    // Final straight segment to last point
    let last = pts[n - 1];
    d.push_str(&format!("L{:.3},{:.3}", last.0, last.1));
    d
}

/// Convenience wrapper with a default corner radius of 5px.
#[allow(dead_code)]
pub fn smooth_bezier_path(pts: &[(f64, f64)]) -> String {
    rounded_path(pts, 5.0)
}

pub fn curve_basis_path(pts: &[(f64, f64)]) -> String {
    let n = pts.len();
    if n == 0 {
        return String::new();
    }
    if n == 1 {
        return format!("M{:.3},{:.3}", pts[0].0, pts[0].1);
    }
    if n == 2 {
        return format!(
            "M{:.3},{:.3}L{:.3},{:.3}",
            pts[0].0, pts[0].1, pts[1].0, pts[1].1
        );
    }
    // Extend with phantom endpoints: [P0, P0, P1, ..., Pn-1, Pn-1]
    let mut e: Vec<(f64, f64)> = Vec::with_capacity(n + 2);
    e.push(pts[0]);
    e.extend_from_slice(pts);
    e.push(pts[n - 1]);

    // B-spline helper: (A + 4B + C) / 6
    let bs = |a: (f64, f64), b: (f64, f64), c: (f64, f64)| -> (f64, f64) {
        ((a.0 + 4.0 * b.0 + c.0) / 6.0, (a.1 + 4.0 * b.1 + c.1) / 6.0)
    };

    // Start exactly at first point, line to B-spline start of segment 0
    let bs0 = bs(e[0], e[1], e[2]);
    let mut d = format!("M{:.3},{:.3}L{:.3},{:.3}", pts[0].0, pts[0].1, bs0.0, bs0.1);

    // Cubic bezier segments through the extended points
    for i in 0..n - 1 {
        let (p0, p1, p2, p3) = (e[i], e[i + 1], e[i + 2], e[i + 3]);
        let cp1 = ((2.0 * p1.0 + p2.0) / 3.0, (2.0 * p1.1 + p2.1) / 3.0);
        let cp2 = ((p1.0 + 2.0 * p2.0) / 3.0, (p1.1 + 2.0 * p2.1) / 3.0);
        let end = bs(p1, p2, p3);
        let _ = p0; // p0 used only for phantom, not needed for cp calculation
        d.push_str(&format!(
            "C{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
            cp1.0, cp1.1, cp2.0, cp2.1, end.0, end.1
        ));
    }

    // End exactly at last point
    d.push_str(&format!("L{:.3},{:.3}", pts[n - 1].0, pts[n - 1].1));
    d
}

// ── Generic SVG element builders ─────────────────────────────────────────────
// Phase-1 additions: not yet wired into diagram renderers (Phase 2 will do
// that migration). `dead_code` is suppressed per-item until then.

/// Render an SVG attribute string from key-value pairs.
///
/// Returns a space-leading string like ` foo="bar" baz="qux"` so it can be
/// concatenated directly into an element tag. Returns an empty string when
/// `pairs` is empty.
#[allow(dead_code)]
pub fn svg_attrs(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!(r#" {k}="{v}""#))
        .collect()
}

/// Render a `<rect>` element.
///
/// `x`, `y`, `w`, `h` set the geometry. Pass extra SVG attributes via `attrs`,
/// e.g. `&[("fill", "blue"), ("rx", "4")]`.
#[allow(dead_code)]
pub fn rect(x: f64, y: f64, w: f64, h: f64, attrs: &[(&str, &str)]) -> String {
    let extra = svg_attrs(attrs);
    format!(r#"<rect x="{x}" y="{y}" width="{w}" height="{h}"{extra}/>"#)
}

/// Render a `<circle>` element.
///
/// `cx`, `cy` are the centre coordinates; `r` is the radius. Pass extra SVG
/// attributes via `attrs`, e.g. `&[("fill", "#333"), ("stroke", "none")]`.
#[allow(dead_code)]
pub fn circle(cx: f64, cy: f64, r: f64, attrs: &[(&str, &str)]) -> String {
    let extra = svg_attrs(attrs);
    format!(r#"<circle cx="{cx}" cy="{cy}" r="{r}"{extra}/>"#)
}

/// Render a `<text>` element with optional text content.
///
/// `x`, `y` position the anchor point. `content` is the text body (may be
/// empty). Pass extra SVG attributes via `attrs`, e.g.
/// `&[("text-anchor", "middle"), ("fill", "#000")]`.
#[allow(dead_code)]
pub fn text(x: f64, y: f64, content: &str, attrs: &[(&str, &str)]) -> String {
    let extra = svg_attrs(attrs);
    format!(r#"<text x="{x}" y="{y}"{extra}>{content}</text>"#)
}

/// Render a `<line>` element.
///
/// Pass extra SVG attributes via `attrs`, e.g.
/// `&[("stroke", "#999"), ("stroke-width", "1")]`.
#[allow(dead_code)]
pub fn line(x1: f64, y1: f64, x2: f64, y2: f64, attrs: &[(&str, &str)]) -> String {
    let extra = svg_attrs(attrs);
    format!(r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}"{extra}/>"#)
}

/// Render a `<g>` group element wrapping `children`.
///
/// Pass group-level SVG attributes via `attrs`, e.g.
/// `&[("class", "node"), ("transform", "translate(10,20)")]`.
/// `children` is inserted verbatim between the opening and closing tags.
#[allow(dead_code)]
pub fn g(attrs: &[(&str, &str)], children: &str) -> String {
    let extra = svg_attrs(attrs);
    format!(r#"<g{extra}>{children}</g>"#)
}

/// Render a `<path>` element.
///
/// `d` is the path data string. Pass extra SVG attributes via `attrs`, e.g.
/// `&[("fill", "none"), ("stroke", "black")]`.
#[allow(dead_code)]
pub fn path(d: &str, attrs: &[(&str, &str)]) -> String {
    let extra = svg_attrs(attrs);
    format!(r#"<path d="{d}"{extra}/>"#)
}

/// Render a `<defs>` block.
///
/// `content` is inserted verbatim between `<defs>` and `</defs>`.
#[allow(dead_code)]
pub fn defs(content: &str) -> String {
    format!(r#"<defs>{content}</defs>"#)
}

/// Render a `<marker>` element.
///
/// `id` sets the marker's `id` attribute; `attrs` supplies additional marker
/// attributes such as `refX`, `refY`, `markerWidth`, `markerHeight`, and
/// `orient`; `content` is the inner shape markup (e.g. a `<path>` or
/// `<polygon>`).
#[allow(dead_code)]
pub fn marker(id: &str, attrs: &[(&str, &str)], content: &str) -> String {
    let extra = svg_attrs(attrs);
    format!(r#"<marker id="{id}"{extra}>{content}</marker>"#)
}

/// Render a `<foreignObject>` element with an HTML `<div>` label inside.
///
/// Produces a `<foreignObject>` sized to `w × h` at position `(x, y)`.
/// `label` is placed inside a `<div>` whose inline style is set to
/// `div_style` (may be empty). This matches the pattern used by Mermaid for
/// rich-text node and edge labels.
#[allow(dead_code)]
pub fn foreign_object_label(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    label: &str,
    div_style: &str,
) -> String {
    format!(
        r#"<foreignObject x="{x}" y="{y}" width="{w}" height="{h}"><div xmlns="http://www.w3.org/1999/xhtml" style="{div_style}">{label}</div></foreignObject>"#
    )
}

/// Render an SVG attribute string from key-value pairs.
///
/// This is the free-function alias for [`svg_attrs`]; both are identical.
/// Provided as a short name for use in expression contexts.
#[allow(dead_code)]
pub fn attrs(pairs: &[(&str, &str)]) -> String {
    svg_attrs(pairs)
}

/// Round all floating-point literals in an SVG string to 3 decimal places.
///
/// Eliminates platform-specific f64 precision differences (Windows vs Linux
/// differ in the 14th+ decimal place). Trailing zeros are stripped so
/// `1.500` becomes `1.5` and `2.000` becomes `2`.
pub(crate) fn normalize_floats(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len());
    let bytes = svg.as_bytes();
    let mut i = 0;
    // Track whether we're inside a tag (< ... >) or in text content (> ... <).
    // Only normalize floats inside tags — text content (e.g. "v1.0") must be preserved.
    let mut in_tag = false;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            in_tag = true;
            out.push('<');
            i += 1;
            continue;
        }
        if bytes[i] == b'>' {
            in_tag = false;
            out.push('>');
            i += 1;
            continue;
        }
        // Outside tags: pass text content through unchanged
        if !in_tag {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        }
        // Detect optional leading minus
        let neg = bytes[i] == b'-' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit();
        let start = i;

        let mut j = if neg { i + 1 } else { i };

        // Must start with a digit
        if !bytes[j].is_ascii_digit() {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        }

        // Consume integer digits
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }

        // Only a float if followed by '.' then more digits
        if j < bytes.len() && bytes[j] == b'.' {
            let dot = j;
            j += 1;
            let frac_start = j;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > frac_start {
                // Parse and reformat to 3dp
                if let Ok(v) = svg[start..j].parse::<f64>() {
                    let rounded = format!("{v:.3}");
                    let trimmed = rounded.trim_end_matches('0').trim_end_matches('.');
                    out.push_str(trimmed);
                    i = j;
                    continue;
                }
                // parse failed — emit verbatim up to dot, retry from dot
                out.push_str(&svg[start..dot]);
                i = dot;
                continue;
            }
            // dot not followed by digits — emit integer part, retry from dot
            out.push_str(&svg[start..dot]);
            i = dot;
            continue;
        }

        // Plain integer — emit verbatim
        out.push_str(&svg[start..j]);
        i = j;
    }
    out
}

/// Inject a background rect into an SVG string so the diagram has an explicit
/// background colour when embedded in a dark host page.
#[allow(dead_code)]
pub(crate) fn inject_background(svg: &str, color: &str) -> String {
    let insert = format!(r#"<rect width="100%" height="100%" fill="{}"/>"#, color);
    // Insert after the first closing `>` of the root <svg ...> tag.
    if let Some(pos) = svg.find('>') {
        let mut out = String::with_capacity(svg.len() + insert.len());
        out.push_str(&svg[..pos + 1]);
        out.push_str(&insert);
        out.push_str(&svg[pos + 1..]);
        out
    } else {
        svg.to_string()
    }
}
