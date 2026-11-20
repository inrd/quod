//! Lexer for the phase-1 grammar (`spec/grammar-draft.md` §1).
//!
//! Comments are not tokens: they are collected separately so the formatter can
//! reattach them (`spec/format.md`).

use crate::diag::{Code, Diagnostic, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntSuffix {
    U8,
    U16,
    U32,
    U64,
    Usize,
    I8,
    I16,
    I32,
    I64,
    Isize,
}

impl IntSuffix {
    pub fn as_str(self) -> &'static str {
        match self {
            IntSuffix::U8 => "u8",
            IntSuffix::U16 => "u16",
            IntSuffix::U32 => "u32",
            IntSuffix::U64 => "u64",
            IntSuffix::Usize => "usize",
            IntSuffix::I8 => "i8",
            IntSuffix::I16 => "i16",
            IntSuffix::I32 => "i32",
            IntSuffix::I64 => "i64",
            IntSuffix::Isize => "isize",
        }
    }

    fn parse(s: &str) -> Option<IntSuffix> {
        Some(match s {
            "u8" => IntSuffix::U8,
            "u16" => IntSuffix::U16,
            "u32" => IntSuffix::U32,
            "u64" => IntSuffix::U64,
            "usize" => IntSuffix::Usize,
            "i8" => IntSuffix::I8,
            "i16" => IntSuffix::I16,
            "i32" => IntSuffix::I32,
            "i64" => IntSuffix::I64,
            "isize" => IntSuffix::Isize,
            _ => return None,
        })
    }
}

/// Integer literal: exact value, optional suffix, and the source text
/// (kept so the formatter preserves base and underscores).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntLit {
    pub value: u64,
    pub suffix: Option<IntSuffix>,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kw {
    Fn,
    Type,
    Const,
    Let,
    Mut,
    If,
    Else,
    For,
    In,
    While,
    Return,
    True,
    False,
    As,
    Effects,
    Pub,
    Mod,
    Use,
    Break,
    Continue,
    // Contracts and spec contexts (`spec/contracts.md`).
    Requires,
    Ensures,
    Invariant,
    Decreases,
    Old,
    Result,
    Forall,
    Exists,
}

impl Kw {
    pub fn as_str(self) -> &'static str {
        match self {
            Kw::Fn => "fn",
            Kw::Type => "type",
            Kw::Const => "const",
            Kw::Let => "let",
            Kw::Mut => "mut",
            Kw::If => "if",
            Kw::Else => "else",
            Kw::For => "for",
            Kw::In => "in",
            Kw::While => "while",
            Kw::Return => "return",
            Kw::True => "true",
            Kw::False => "false",
            Kw::As => "as",
            Kw::Effects => "effects",
            Kw::Pub => "pub",
            Kw::Mod => "mod",
            Kw::Use => "use",
            Kw::Break => "break",
            Kw::Continue => "continue",
            Kw::Requires => "requires",
            Kw::Ensures => "ensures",
            Kw::Invariant => "invariant",
            Kw::Decreases => "decreases",
            Kw::Old => "old",
            Kw::Result => "result",
            Kw::Forall => "forall",
            Kw::Exists => "exists",
        }
    }

    fn parse(s: &str) -> Option<Kw> {
        Some(match s {
            "fn" => Kw::Fn,
            "type" => Kw::Type,
            "const" => Kw::Const,
            "let" => Kw::Let,
            "mut" => Kw::Mut,
            "if" => Kw::If,
            "else" => Kw::Else,
            "for" => Kw::For,
            "in" => Kw::In,
            "while" => Kw::While,
            "return" => Kw::Return,
            "true" => Kw::True,
            "false" => Kw::False,
            "as" => Kw::As,
            "effects" => Kw::Effects,
            "pub" => Kw::Pub,
            "mod" => Kw::Mod,
            "use" => Kw::Use,
            "break" => Kw::Break,
            "continue" => Kw::Continue,
            "requires" => Kw::Requires,
            "ensures" => Kw::Ensures,
            "invariant" => Kw::Invariant,
            "decreases" => Kw::Decreases,
            "old" => Kw::Old,
            "result" => Kw::Result,
            "forall" => Kw::Forall,
            "exists" => Kw::Exists,
            _ => return None,
        })
    }
}

/// Keywords reserved for phase 2+ (`spec/grammar-draft.md` §1).
pub const RESERVED: &[&str] = &[
    "assert",
    "where",
    "spec",
    "impl",
    "refines",
    "trust",
    "assume",
    "ghost",
    "proof",
    "lemma",
    "cost",
    "stack",
    "noalloc",
    "mem",
    "provenance",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Punct {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    ColonColon,
    Dot,
    DotLBrace,
    DotDot,
    Arrow,
    Eq,
    EqEq,
    Implies,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,
    Pipe,
    Caret,
    Shl,
    Shr,
    AndAnd,
    OrOr,
    Bang,
    PlusEq,
    MinusEq,
    StarEq,
}

impl Punct {
    pub fn as_str(self) -> &'static str {
        match self {
            Punct::LParen => "(",
            Punct::RParen => ")",
            Punct::LBrace => "{",
            Punct::RBrace => "}",
            Punct::LBracket => "[",
            Punct::RBracket => "]",
            Punct::Comma => ",",
            Punct::Semi => ";",
            Punct::Colon => ":",
            Punct::ColonColon => "::",
            Punct::Dot => ".",
            Punct::DotLBrace => ".{",
            Punct::DotDot => "..",
            Punct::Arrow => "->",
            Punct::Eq => "=",
            Punct::EqEq => "==",
            Punct::Implies => "==>",
            Punct::Ne => "!=",
            Punct::Lt => "<",
            Punct::Le => "<=",
            Punct::Gt => ">",
            Punct::Ge => ">=",
            Punct::Plus => "+",
            Punct::Minus => "-",
            Punct::Star => "*",
            Punct::Slash => "/",
            Punct::Percent => "%",
            Punct::Amp => "&",
            Punct::Pipe => "|",
            Punct::Caret => "^",
            Punct::Shl => "<<",
            Punct::Shr => ">>",
            Punct::AndAnd => "&&",
            Punct::OrOr => "||",
            Punct::Bang => "!",
            Punct::PlusEq => "+=",
            Punct::MinusEq => "-=",
            Punct::StarEq => "*=",
        }
    }
}

/// Punctuation, longest first so maximal munch works by linear scan.
const PUNCTS: &[Punct] = &[
    Punct::Implies,
    Punct::ColonColon,
    Punct::DotLBrace,
    Punct::DotDot,
    Punct::Arrow,
    Punct::EqEq,
    Punct::Ne,
    Punct::Le,
    Punct::Ge,
    Punct::Shl,
    Punct::Shr,
    Punct::AndAnd,
    Punct::OrOr,
    Punct::PlusEq,
    Punct::MinusEq,
    Punct::StarEq,
    Punct::LParen,
    Punct::RParen,
    Punct::LBrace,
    Punct::RBrace,
    Punct::LBracket,
    Punct::RBracket,
    Punct::Comma,
    Punct::Semi,
    Punct::Colon,
    Punct::Dot,
    Punct::Eq,
    Punct::Lt,
    Punct::Gt,
    Punct::Plus,
    Punct::Minus,
    Punct::Star,
    Punct::Slash,
    Punct::Percent,
    Punct::Amp,
    Punct::Pipe,
    Punct::Caret,
    Punct::Bang,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokKind {
    Ident(String),
    Int(IntLit),
    Hole(String),
    Kw(Kw),
    Reserved(String),
    Punct(Punct),
    Eof,
}

impl TokKind {
    /// Human description for "expected X, found Y" messages.
    pub fn describe(&self) -> String {
        match self {
            TokKind::Ident(s) => format!("identifier `{s}`"),
            TokKind::Int(l) => format!("integer literal `{}`", l.text),
            TokKind::Hole(s) => format!("hole `?{s}`"),
            TokKind::Kw(k) => format!("keyword `{}`", k.as_str()),
            TokKind::Reserved(s) => format!("reserved keyword `{s}`"),
            TokKind::Punct(p) => format!("`{}`", p.as_str()),
            TokKind::Eof => "end of file".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokKind,
    pub span: Span,
    /// 0-based line of the token's first byte.
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    /// Text after `//`, trailing whitespace removed.
    pub text: String,
    pub span: Span,
    /// 0-based line.
    pub line: usize,
    /// True if only whitespace precedes the comment on its line.
    pub own_line: bool,
    /// True if a blank line follows the comment.
    pub blank_after: bool,
}

pub struct Lexed {
    pub tokens: Vec<Token>,
    pub comments: Vec<Comment>,
}

pub fn lex(src: &str) -> Result<Lexed, Diagnostic> {
    let b = src.as_bytes();
    let mut i = 0;
    let mut line = 0;
    let mut line_has_code = false;
    let mut tokens = Vec::new();
    let mut comments: Vec<Comment> = Vec::new();

    while i < b.len() {
        let c = b[i];
        if c == b'\n' {
            line += 1;
            line_has_code = false;
            i += 1;
            continue;
        }
        if c == b' ' || c == b'\t' || c == b'\r' {
            i += 1;
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'/') {
            let start = i;
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            let text = src[start + 2..i].trim_end().to_string();
            comments.push(Comment {
                text,
                span: Span::new(start, i),
                line,
                own_line: !line_has_code,
                blank_after: blank_line_follows(b, i),
            });
            continue;
        }
        line_has_code = true;
        let start = i;
        let kind = if c.is_ascii_alphabetic() || c == b'_' {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            let word = &src[start..i];
            if let Some(k) = Kw::parse(word) {
                TokKind::Kw(k)
            } else if RESERVED.contains(&word) {
                TokKind::Reserved(word.to_string())
            } else {
                TokKind::Ident(word.to_string())
            }
        } else if c.is_ascii_digit() {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            TokKind::Int(int_lit(&src[start..i], Span::new(start, i))?)
        } else if c == b'?' {
            i += 1;
            let id_start = i;
            if i < b.len() && (b[i].is_ascii_alphabetic() || b[i] == b'_') {
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                TokKind::Hole(src[id_start..i].to_string())
            } else {
                return Err(Diagnostic::new(
                    Code::E0103,
                    Span::new(start, i),
                    "`?` must be followed by an identifier (typed hole, e.g. `?h1`)",
                ));
            }
        } else if let Some(p) = PUNCTS.iter().find(|p| src[i..].starts_with(p.as_str())) {
            i += p.as_str().len();
            TokKind::Punct(*p)
        } else {
            let ch = src[i..].chars().next().unwrap_or('?');
            return Err(Diagnostic::new(
                Code::E0101,
                Span::new(i, i + ch.len_utf8()),
                format!("unexpected character `{}`", ch.escape_default()),
            ));
        };
        tokens.push(Token {
            kind,
            span: Span::new(start, i),
            line,
        });
    }
    tokens.push(Token {
        kind: TokKind::Eof,
        span: Span::new(b.len(), b.len()),
        line,
    });
    Ok(Lexed { tokens, comments })
}

/// After a comment ending at `i` (at `\n` or EOF): is the next line blank?
fn blank_line_follows(b: &[u8], mut i: usize) -> bool {
    if i >= b.len() {
        return false;
    }
    i += 1; // skip '\n'
    while i < b.len() && (b[i] == b' ' || b[i] == b'\t' || b[i] == b'\r') {
        i += 1;
    }
    i < b.len() && b[i] == b'\n'
}

fn int_lit(text: &str, span: Span) -> Result<IntLit, Diagnostic> {
    let bad = |msg: String| Diagnostic::new(Code::E0102, span, msg);
    let (radix, body) = if let Some(r) = text.strip_prefix("0x") {
        (16u32, r)
    } else if let Some(r) = text.strip_prefix("0b") {
        (2u32, r)
    } else {
        (10u32, text)
    };
    // Digits: first must be a digit of the radix, then digits or '_'.
    let digits_end = body
        .char_indices()
        .find(|&(idx, ch)| !(ch.is_digit(radix) || (ch == '_' && idx > 0)))
        .map_or(body.len(), |(idx, _)| idx);
    let (digits, suffix_text) = body.split_at(digits_end);
    if digits.is_empty() {
        return Err(bad(format!("integer literal `{text}` has no digits")));
    }
    let suffix = if suffix_text.is_empty() {
        None
    } else {
        Some(IntSuffix::parse(suffix_text).ok_or_else(|| {
            bad(format!("invalid integer suffix or digit in `{text}` (suffixes: u8 … u64, usize, i8 … i64, isize)"))
        })?)
    };
    let mut value: u64 = 0;
    for ch in digits.chars().filter(|&c| c != '_') {
        let d = u64::from(ch.to_digit(radix).unwrap_or(0));
        value = value
            .checked_mul(u64::from(radix))
            .and_then(|v| v.checked_add(d))
            .ok_or_else(|| bad(format!("integer literal `{text}` exceeds 2^64 - 1")))?;
    }
    Ok(IntLit {
        value,
        suffix,
        text: text.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokKind> {
        lex(src)
            .unwrap()
            .tokens
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn literals() {
        let lits: Vec<(u64, Option<IntSuffix>)> = lex("0xffu8 1_000u32 0b101i8 7")
            .unwrap()
            .tokens
            .into_iter()
            .filter_map(|t| match t.kind {
                TokKind::Int(l) => Some((l.value, l.suffix)),
                _ => None,
            })
            .collect();
        assert_eq!(
            lits,
            vec![
                (255, Some(IntSuffix::U8)),
                (1000, Some(IntSuffix::U32)),
                (5, Some(IntSuffix::I8)),
                (7, None)
            ]
        );
    }

    #[test]
    fn bad_literals() {
        for src in [
            "0x",
            "0x_1",
            "12abc",
            "0b102",
            "18446744073709551616",
            "1u128",
        ] {
            assert_eq!(lex(src).err().map(|d| d.code), Some(Code::E0102), "{src}");
        }
        assert_eq!(lex("18446744073709551615").unwrap().tokens.len(), 2);
    }

    #[test]
    fn puncts_and_dot_brace() {
        assert_eq!(
            kinds("a.{ x.y .. ::"),
            vec![
                TokKind::Ident("a".into()),
                TokKind::Punct(Punct::DotLBrace),
                TokKind::Ident("x".into()),
                TokKind::Punct(Punct::Dot),
                TokKind::Ident("y".into()),
                TokKind::Punct(Punct::DotDot),
                TokKind::Punct(Punct::ColonColon),
                TokKind::Eof
            ]
        );
    }

    #[test]
    fn range_after_suffix() {
        assert_eq!(
            kinds("1usize..n"),
            vec![
                TokKind::Int(IntLit {
                    value: 1,
                    suffix: Some(IntSuffix::Usize),
                    text: "1usize".into()
                }),
                TokKind::Punct(Punct::DotDot),
                TokKind::Ident("n".into()),
                TokKind::Eof
            ]
        );
    }

    #[test]
    fn implies_longest_match() {
        assert_eq!(
            kinds("a ==> b == c"),
            vec![
                TokKind::Ident("a".into()),
                TokKind::Punct(Punct::Implies),
                TokKind::Ident("b".into()),
                TokKind::Punct(Punct::EqEq),
                TokKind::Ident("c".into()),
                TokKind::Eof
            ]
        );
    }

    #[test]
    fn errors() {
        assert_eq!(lex("a $ b").err().map(|d| d.code), Some(Code::E0101));
        assert_eq!(lex("? x").err().map(|d| d.code), Some(Code::E0103));
        assert_eq!(lex("é").err().map(|d| d.code), Some(Code::E0101));
    }

    #[test]
    fn comments() {
        let l = lex("// head\n\nx // tail\n").unwrap();
        assert_eq!(l.comments.len(), 2);
        assert!(l.comments[0].own_line && l.comments[0].blank_after);
        assert!(!l.comments[1].own_line && !l.comments[1].blank_after);
        assert_eq!(l.comments[1].text, " tail");
    }
}
