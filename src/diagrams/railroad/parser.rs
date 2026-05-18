/// Parser for Mermaid Railroad (syntax/grammar) diagram syntax.
///
/// Faithful port of the Railroad diagram grammar from railroadDb.ts.
///
/// Grammar (EBNF-like notation used by Mermaid):
///   railroad
///   [title <text>]
///   <rule_name> ::= <expression>
///   <rule_name> = <expression>
///
/// Expressions (EBNF):
///   terminal     — quoted string: "text" or 'text'
///   nonterminal  — unquoted identifier
///   sequence     — expr expr ...
///   choice       — expr | expr | ...
///   optional     — [ expr ]
///   repetition*  — { expr }   (zero or more)
///   repetition+  — expr+      (one or more, or { expr }+ if needed)
///   group        — ( expr )
///   special      — ? text ?
///
/// The parser produces an AST of ASTNode variants.

#[derive(Debug, Clone)]
pub enum AstNode {
    Terminal(String),
    NonTerminal(String),
    Sequence(Vec<AstNode>),
    Choice(Vec<AstNode>),
    Optional(Box<AstNode>),
    Repetition { element: Box<AstNode>, min: u32 }, // min=0 → *, min=1 → +
    Special(String),
}

#[derive(Debug, Clone)]
pub struct RailroadRule {
    pub name: String,
    pub definition: AstNode,
    pub comment: Option<String>,
}

pub struct RailroadDiagram {
    pub title: Option<String>,
    pub rules: Vec<RailroadRule>,
}

pub fn parse(input: &str) -> crate::error::ParseResult<RailroadDiagram> {
    let mut title: Option<String> = None;
    let mut rules: Vec<RailroadRule> = Vec::new();
    let mut header_seen = false;

    // Accumulate lines into rule definitions (which may span multiple lines)
    let mut pending_rule: Option<(String, String, Option<String>)> = None;

    for raw_line in input.lines() {
        let trimmed = raw_line.trim();

        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        if !header_seen {
            let lower = trimmed.to_lowercase();
            if lower == "railroad"
                || lower == "railroad-beta"
                || lower.starts_with("railroad ")
                || lower.starts_with("railroad-beta ")
            {
                header_seen = true;
                continue;
            }
            continue;
        }

        // title
        if trimmed.to_lowercase().starts_with("title ") {
            title = Some(trimmed[6..].trim().to_string());
            continue;
        }

        // Comment extraction (// at end of line or standalone)
        let (line_content, comment) = split_comment(trimmed);
        let line_content = line_content.trim();

        if line_content.is_empty() {
            continue;
        }

        // Check for rule definition: "name ::= expr" or "name = expr"
        // Must check "::=" before "="
        let rule_sep = if line_content.contains("::=") {
            Some("::=")
        } else if let Some(eq_pos) = find_rule_equals(line_content) {
            let _ = eq_pos;
            Some("=")
        } else {
            None
        };

        if let Some(sep) = rule_sep {
            // Flush previous pending rule
            if let Some((name, expr_str, cmt)) = pending_rule.take() {
                if let Some(rule) = build_rule(&name, &expr_str, cmt) {
                    rules.push(rule);
                }
            }

            let parts: Vec<&str> = line_content.splitn(2, sep).collect();
            if parts.len() == 2 {
                let name = parts[0].trim().to_string();
                let expr_str = parts[1].trim().to_string();
                pending_rule = Some((name, expr_str, comment.map(|s| s.to_string())));
            }
        } else if pending_rule.is_some() {
            // Continuation of previous rule
            if let Some((_, ref mut expr_str, _)) = pending_rule {
                expr_str.push(' ');
                expr_str.push_str(line_content);
            }
        }
    }

    // Flush last rule
    if let Some((name, expr_str, cmt)) = pending_rule {
        if let Some(rule) = build_rule(&name, &expr_str, cmt) {
            rules.push(rule);
        }
    }

    crate::error::ParseResult::ok(RailroadDiagram { title, rules })
}

/// Split a line into (content, optional comment after //)
fn split_comment(s: &str) -> (&str, Option<&str>) {
    if let Some(pos) = find_comment_pos(s) {
        (&s[..pos], Some(s[pos + 2..].trim()))
    } else {
        (s, None)
    }
}

fn find_comment_pos(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' if !in_double => in_single = !in_single,
            b'"' if !in_single => in_double = !in_double,
            b'/' if !in_single && !in_double && bytes.get(i + 1) == Some(&b'/') => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find the position of a bare '=' that is not part of '::='
fn find_rule_equals(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'=' {
            // Check it's not part of ::=
            if i > 0 && bytes[i - 1] == b':' {
                continue;
            }
            // Check it's not ==, >=, <=, !=
            if bytes.get(i + 1) == Some(&b'=') {
                continue;
            }
            if i > 0 && (bytes[i - 1] == b'>' || bytes[i - 1] == b'<' || bytes[i - 1] == b'!') {
                continue;
            }
            return Some(i);
        }
    }
    None
}

fn build_rule(name: &str, expr_str: &str, comment: Option<String>) -> Option<RailroadRule> {
    if name.is_empty() {
        return None;
    }
    let definition = parse_expression(expr_str.trim())?;
    Some(RailroadRule {
        name: name.to_string(),
        definition,
        comment,
    })
}

// ── Expression parser ─────────────────────────────────────────────────────────

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Parser {
            input: s.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self) -> Option<u8> {
        let b = self.input.get(self.pos).copied();
        if b.is_some() {
            self.pos += 1;
        }
        b
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    #[allow(dead_code)]
    fn at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    #[allow(dead_code)]
    fn rest(&self) -> &str {
        std::str::from_utf8(&self.input[self.pos..]).unwrap_or("")
    }

    /// Parse a choice expression (highest precedence = lowest binding).
    fn parse_choice(&mut self) -> Option<AstNode> {
        let first = self.parse_sequence()?;
        self.skip_ws();

        if self.peek() != Some(b'|') {
            return Some(first);
        }

        let mut alternatives = vec![first];
        while self.peek() == Some(b'|') {
            self.consume(); // consume '|'
            self.skip_ws();
            let next = self.parse_sequence()?;
            alternatives.push(next);
            self.skip_ws();
        }
        Some(AstNode::Choice(alternatives))
    }

    /// Parse a sequence of atoms.
    fn parse_sequence(&mut self) -> Option<AstNode> {
        let mut elements = Vec::new();
        loop {
            self.skip_ws();
            // Stop at | ) ] }  or end
            match self.peek() {
                None | Some(b'|' | b')' | b']' | b'}') => break,
                _ => {}
            }
            if let Some(atom) = self.parse_atom() {
                elements.push(atom);
            } else {
                break;
            }
        }
        match elements.len() {
            0 => None,
            1 => Some(elements.remove(0)),
            _ => Some(AstNode::Sequence(elements)),
        }
    }

    /// Parse a single atom (terminal, nonterminal, group, optional, repetition, special).
    fn parse_atom(&mut self) -> Option<AstNode> {
        self.skip_ws();
        let b = self.peek()?;

        match b {
            // Terminal: "..." or '...'
            b'"' | b'\'' => {
                let quote = self.consume().unwrap();
                let mut text = String::new();
                loop {
                    match self.consume() {
                        Some(c) if c == quote => break,
                        Some(c) => text.push(c as char),
                        None => break,
                    }
                }
                // Check for + suffix (one or more repetition)
                let min = if self.peek() == Some(b'+') {
                    self.consume();
                    1
                } else if self.peek() == Some(b'*') {
                    self.consume();
                    0
                } else {
                    return Some(AstNode::Terminal(text));
                };
                Some(AstNode::Repetition {
                    element: Box::new(AstNode::Terminal(text)),
                    min,
                })
            }

            // Group: (...)
            b'(' => {
                self.consume();
                let inner = self.parse_choice()?;
                self.skip_ws();
                if self.peek() == Some(b')') {
                    self.consume();
                }
                // Suffix?
                let min_opt = match self.peek() {
                    Some(b'+') => {
                        self.consume();
                        Some(1u32)
                    }
                    Some(b'*') => {
                        self.consume();
                        Some(0u32)
                    }
                    Some(b'?') => {
                        self.consume();
                        None
                    } // optional
                    _ => return Some(inner),
                };
                match min_opt {
                    Some(min) => Some(AstNode::Repetition {
                        element: Box::new(inner),
                        min,
                    }),
                    None => Some(AstNode::Optional(Box::new(inner))),
                }
            }

            // Optional: [...]
            b'[' => {
                self.consume();
                let inner = self.parse_choice()?;
                self.skip_ws();
                if self.peek() == Some(b']') {
                    self.consume();
                }
                Some(AstNode::Optional(Box::new(inner)))
            }

            // Repetition / group: {...}
            b'{' => {
                self.consume();
                let inner = self.parse_choice()?;
                self.skip_ws();
                if self.peek() == Some(b'}') {
                    self.consume();
                }
                // {expr} = zero-or-more; {expr}+ = one-or-more
                let min = if self.peek() == Some(b'+') {
                    self.consume();
                    1
                } else {
                    0
                };
                Some(AstNode::Repetition {
                    element: Box::new(inner),
                    min,
                })
            }

            // Special: ?...?
            b'?' => {
                self.consume();
                let mut text = String::new();
                loop {
                    match self.consume() {
                        Some(b'?') => break,
                        Some(c) => text.push(c as char),
                        None => break,
                    }
                }
                Some(AstNode::Special(text.trim().to_string()))
            }

            // Nonterminal: identifier
            _ if (b as char).is_alphanumeric() || b == b'_' || b == b'-' => {
                let mut ident = String::new();
                while let Some(c) = self.peek() {
                    if (c as char).is_alphanumeric() || c == b'_' || c == b'-' {
                        ident.push(c as char);
                        self.consume();
                    } else {
                        break;
                    }
                }
                // Suffix
                let node = AstNode::NonTerminal(ident);
                match self.peek() {
                    Some(b'+') => {
                        self.consume();
                        Some(AstNode::Repetition {
                            element: Box::new(node),
                            min: 1,
                        })
                    }
                    Some(b'*') => {
                        self.consume();
                        Some(AstNode::Repetition {
                            element: Box::new(node),
                            min: 0,
                        })
                    }
                    Some(b'?') => {
                        self.consume();
                        Some(AstNode::Optional(Box::new(node)))
                    }
                    _ => Some(node),
                }
            }

            _ => {
                // Unknown character — skip it
                self.consume();
                None
            }
        }
    }
}

fn parse_expression(s: &str) -> Option<AstNode> {
    if s.is_empty() {
        return None;
    }
    let mut p = Parser::new(s);
    let node = p.parse_choice()?;
    // If nothing was parsed but there's still input, return a nonterminal of the raw string
    Some(node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_rule() {
        let input = "railroad\n    title My Grammar\n    digit ::= \"0\" | \"1\" | \"2\"\n";
        let d = parse(input).diagram;
        assert_eq!(d.title.as_deref(), Some("My Grammar"));
        assert_eq!(d.rules.len(), 1);
        assert_eq!(d.rules[0].name, "digit");
        match &d.rules[0].definition {
            AstNode::Choice(alts) => assert_eq!(alts.len(), 3),
            other => panic!("Expected Choice, got {:?}", other),
        }
    }

    #[test]
    fn sequence_rule() {
        let input = "railroad\n    expr = term ((\"+\" | \"-\") term)*\n";
        let d = parse(input).diagram;
        assert_eq!(d.rules.len(), 1);
    }

    #[test]
    fn optional_rule() {
        let input = "railroad\n    opt_rule = [\"a\" | \"b\"]\n";
        let d = parse(input).diagram;
        let def = &d.rules[0].definition;
        match def {
            AstNode::Optional(_) => {}
            other => panic!("Expected Optional, got {:?}", other),
        }
    }

    #[test]
    fn repetition_rule() {
        let input = "railroad\n    list = { item }+\n";
        let d = parse(input).diagram;
        match &d.rules[0].definition {
            AstNode::Repetition { min, .. } => assert_eq!(*min, 1),
            other => panic!("Expected Repetition, got {:?}", other),
        }
    }
}
