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
    /// Retired: was `while` before contracts (`spec/contracts.md`). Never emitted.
    E0206,
    /// Unknown name.
    E0301,
    /// Duplicate definition or shadowing.
    E0302,
    /// Type mismatch.
    E0303,
    /// Unsuffixed literal or hole with no expected type.
    E0304,
    /// Constant out of range.
    E0305,
    /// Operator not defined for the operand type.
    E0306,
    /// Invalid cast.
    E0307,
    /// Not assignable.
    E0308,
    /// Wrong number of arguments.
    E0309,
    /// Call target is not a function.
    E0310,
    /// Unknown field.
    E0311,
    /// Struct literal fields missing, duplicated or out of order.
    E0312,
    /// Missing return.
    E0313,
    /// Unreachable statement.
    E0314,
    /// `break`/`continue` outside a loop.
    E0315,
    /// Expression statement is not a call returning `()`.
    E0316,
    /// Recursion (phase 1).
    E0317,
    /// Construct not available in phase 1.
    E0318,
    /// Reference outside a parameter type or call argument.
    E0319,
    /// `&mut` argument aliases another argument.
    E0320,
    /// Unfilled hole.
    E0321,
    /// Invalid array length.
    E0322,
    /// Array literal length mismatch.
    E0323,
    /// Struct contains itself by value.
    E0324,
    /// Range index not directly under `&`/`&mut`.
    E0325,
    /// Spec-only construct in code.
    E0326,
    /// `old` in `requires`, nested, or reading a non-parameter.
    E0327,
    /// `result` outside `ensures` or in a `()` function.
    E0328,
    /// Call in a spec context.
    E0329,
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
            Code::E0301 => "E0301",
            Code::E0302 => "E0302",
            Code::E0303 => "E0303",
            Code::E0304 => "E0304",
            Code::E0305 => "E0305",
            Code::E0306 => "E0306",
            Code::E0307 => "E0307",
            Code::E0308 => "E0308",
            Code::E0309 => "E0309",
            Code::E0310 => "E0310",
            Code::E0311 => "E0311",
            Code::E0312 => "E0312",
            Code::E0313 => "E0313",
            Code::E0314 => "E0314",
            Code::E0315 => "E0315",
            Code::E0316 => "E0316",
            Code::E0317 => "E0317",
            Code::E0318 => "E0318",
            Code::E0319 => "E0319",
            Code::E0320 => "E0320",
            Code::E0321 => "E0321",
            Code::E0322 => "E0322",
            Code::E0323 => "E0323",
            Code::E0324 => "E0324",
            Code::E0325 => "E0325",
            Code::E0326 => "E0326",
            Code::E0327 => "E0327",
            Code::E0328 => "E0328",
            Code::E0329 => "E0329",
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
