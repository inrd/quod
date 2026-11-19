//! Diagnostics with stable codes. See `spec/diagnostics.md`.

use std::fmt;

/// Byte range in the source file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    pub fn to(self, other: Span) -> Span {
        Span {
            start: self.start,
            end: other.end,
        }
    }
}

/// Stable diagnostic codes. Never renumber; retired codes are never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Code {
    /// Unexpected character.
    E0101,
    /// Malformed integer literal.
    E0102,
    /// Malformed hole.
    E0103,
    /// Unexpected token.
    E0201,
    /// Reserved keyword (phase 2+).
    E0202,
    /// Mixed arithmetic operators without parentheses.
    E0203,
    /// Chained comparison.
    E0204,
    /// `&&` and `||` mixed without parentheses.
    E0205,
    /// Construct not available in phase 1.
    E0206,
}

impl Code {
    pub fn as_str(self) -> &'static str {
        match self {
            Code::E0101 => "E0101",
            Code::E0102 => "E0102",
            Code::E0103 => "E0103",
            Code::E0201 => "E0201",
            Code::E0202 => "E0202",
            Code::E0203 => "E0203",
            Code::E0204 => "E0204",
            Code::E0205 => "E0205",
            Code::E0206 => "E0206",
        }
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: Code,
    pub span: Span,
    pub message: String,
}

impl Diagnostic {
    pub fn new(code: Code, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            code,
            span,
            message: message.into(),
        }
    }

    /// Render as `file:line:col: error[CODE]: message`.
    pub fn render(&self, file: &str, src: &str) -> String {
        let (line, col) = line_col(src, self.span.start);
        format!(
            "{file}:{line}:{col}: error[{}]: {}",
            self.code, self.message
        )
    }
}

/// 1-based line and column (in bytes) of a byte offset.
pub fn line_col(src: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(src.len());
    let before = &src.as_bytes()[..offset];
    let line = before.iter().filter(|&&b| b == b'\n').count() + 1;
    let line_start = before
        .iter()
        .rposition(|&b| b == b'\n')
        .map_or(0, |p| p + 1);
    (line, offset - line_start + 1)
}
