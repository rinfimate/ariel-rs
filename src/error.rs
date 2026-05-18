//! Error types for ariel-rs diagram parsing and rendering.

/// A single parse error with source location.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// 1-based line number where the error occurred, if known.
    pub line: Option<usize>,
    /// 1-based column number where the error occurred, if known.
    pub column: Option<usize>,
    /// Human-readable description of what went wrong.
    pub message: String,
}

impl ParseError {
    /// Create a parse error with a message only (no location).
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            line: None,
            column: None,
            message: message.into(),
        }
    }

    /// Create a parse error with a line number.
    pub fn at_line(line: usize, message: impl Into<String>) -> Self {
        Self {
            line: Some(line),
            column: None,
            message: message.into(),
        }
    }

    /// Create a parse error with a full source location.
    pub fn at(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self {
            line: Some(line),
            column: Some(column),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.line, self.column) {
            (Some(l), Some(c)) => write!(f, "line {l}, col {c}: {}", self.message),
            (Some(l), None) => write!(f, "line {l}: {}", self.message),
            _ => write!(f, "{}", self.message),
        }
    }
}

/// Error returned by [`crate::try_render`].
#[derive(Debug, Clone)]
pub struct RenderError {
    /// The detected diagram type (e.g. `"flowchart"`, `"unknown"`).
    pub diagram_type: String,
    /// Human-readable description of the failure.
    pub message: String,
    /// Parse errors collected during parsing, if any.
    pub parse_errors: Vec<ParseError>,
}

impl RenderError {
    /// Create a render error for an unrecognised diagram type.
    pub fn unknown_type() -> Self {
        Self {
            diagram_type: "unknown".into(),
            message: "Unrecognized diagram type.".into(),
            parse_errors: vec![],
        }
    }

    /// Create a render error from a panic message.
    pub fn from_panic(diagram_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            diagram_type: diagram_type.into(),
            message: message.into(),
            parse_errors: vec![],
        }
    }
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} render error: {}", self.diagram_type, self.message)?;
        for e in &self.parse_errors {
            write!(f, "\n  - {e}")?;
        }
        Ok(())
    }
}

impl std::error::Error for RenderError {}

/// Best-effort parse output: always contains a (possibly empty) diagram
/// plus any errors collected during parsing.
#[derive(Debug)]
pub struct ParseResult<T> {
    /// The parsed diagram (may be partial if errors occurred).
    pub diagram: T,
    /// Errors collected during parsing.
    pub errors: Vec<ParseError>,
}

impl<T> ParseResult<T> {
    /// Create a successful parse result with no errors.
    pub fn ok(diagram: T) -> Self {
        Self {
            diagram,
            errors: vec![],
        }
    }

    /// Create a parse result with errors.
    pub fn with_errors(diagram: T, errors: Vec<ParseError>) -> Self {
        Self { diagram, errors }
    }

    /// Return true if there are no parse errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // ── ParseError constructors ───────────────────────────────────────────────

    #[test]
    fn parse_error_new_no_location() {
        let e = ParseError::new("oops");
        assert_eq!(e.message, "oops");
        assert_eq!(e.line, None);
        assert_eq!(e.column, None);
        assert_eq!(e.to_string(), "oops");
    }

    #[test]
    fn parse_error_at_line() {
        let e = ParseError::at_line(3, "bad token");
        assert_eq!(e.line, Some(3));
        assert_eq!(e.column, None);
        assert_eq!(e.to_string(), "line 3: bad token");
    }

    #[test]
    fn parse_error_at_full_location() {
        let e = ParseError::at(5, 12, "unexpected '}'");
        assert_eq!(e.line, Some(5));
        assert_eq!(e.column, Some(12));
        assert_eq!(e.to_string(), "line 5, col 12: unexpected '}'");
    }

    // ── RenderError constructors ──────────────────────────────────────────────

    #[test]
    fn render_error_unknown_type() {
        let e = RenderError::unknown_type();
        assert_eq!(e.diagram_type, "unknown");
        assert!(e.message.contains("Unrecognized"));
        assert!(e.parse_errors.is_empty());
    }

    #[test]
    fn render_error_from_panic() {
        let e = RenderError::from_panic("flowchart", "index out of bounds");
        assert_eq!(e.diagram_type, "flowchart");
        assert_eq!(e.message, "index out of bounds");
        assert!(e.parse_errors.is_empty());
    }

    // ── RenderError Display ───────────────────────────────────────────────────

    #[test]
    fn render_error_display_no_parse_errors() {
        let e = RenderError::from_panic("pie", "something failed");
        let s = e.to_string();
        assert!(s.contains("pie"));
        assert!(s.contains("something failed"));
    }

    #[test]
    fn render_error_display_with_parse_errors() {
        let mut e = RenderError::unknown_type();
        e.parse_errors.push(ParseError::at_line(2, "bad input"));
        let s = e.to_string();
        assert!(s.contains("line 2: bad input"));
    }

    // ── RenderError implements std::error::Error ──────────────────────────────

    #[test]
    fn render_error_source_is_none() {
        let e = RenderError::unknown_type();
        assert!(e.source().is_none());
    }

    // ── ParseResult ───────────────────────────────────────────────────────────

    #[test]
    fn parse_result_ok_is_ok() {
        let r: ParseResult<i32> = ParseResult::ok(42);
        assert!(r.is_ok());
        assert_eq!(r.diagram, 42);
        assert!(r.errors.is_empty());
    }

    #[test]
    fn parse_result_with_errors_not_ok() {
        let errs = vec![ParseError::new("err1"), ParseError::new("err2")];
        let r: ParseResult<&str> = ParseResult::with_errors("partial", errs);
        assert!(!r.is_ok());
        assert_eq!(r.errors.len(), 2);
        assert_eq!(r.diagram, "partial");
    }
}
