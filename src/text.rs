#![allow(dead_code)]
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use std::sync::OnceLock;

static FONT: OnceLock<FontRef<'static>> = OnceLock::new();
static FONT_BYTES: &[u8] = include_bytes!("../assets/LiberationSans-Regular.ttf");

fn font() -> &'static FontRef<'static> {
    FONT.get_or_init(|| {
        FontRef::try_from_slice(FONT_BYTES).expect("LiberationSans-Regular.ttf valid")
    })
}

/// Measure text using ab_glyph with the bundled Arial font + KERN table.
pub fn measure(text: &str, font_size: f64) -> (f64, f64) {
    let f = font();
    let scale = PxScale::from(font_size as f32);
    let scaled = f.as_scaled(scale);
    let chars: Vec<char> = text.chars().collect();
    let ids: Vec<_> = chars.iter().map(|&c| scaled.glyph_id(c)).collect();
    let mut w: f32 = ids.iter().map(|&g| scaled.h_advance(g)).sum();
    for pair in ids.windows(2) {
        w += scaled.kern(pair[0], pair[1]);
    }
    let h = scaled.ascent() - scaled.descent() + scaled.line_gap();
    (w as f64, h as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_hello_14px() {
        let (w, h) = measure("Hello", 14.0);
        assert!(w > 20.0 && w < 60.0, "width: {w}");
        assert!(h > 10.0 && h < 25.0, "height: {h}");
    }

    #[test]
    fn wider_string_is_wider() {
        let (w1, _) = measure("A", 14.0);
        let (w2, _) = measure("AAAA", 14.0);
        assert!(w2 > w1 * 3.0);
    }

    #[test]
    fn scales_with_font_size() {
        let (w1, h1) = measure("Hello", 14.0);
        let (w2, h2) = measure("Hello", 28.0);
        assert!((w2 / w1 - 2.0).abs() < 0.01);
        assert!((h2 / h1 - 2.0).abs() < 0.01);
    }
}
