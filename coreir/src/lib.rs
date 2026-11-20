//! Core IR for the thin slice (`spec/core-ir.md`): types, canonical text
//! parser and printer, and well-formedness checks (§3.4).
//!
//! Trusted if ADR-0007 is accepted: std only, no hash collections, no I/O.

#![forbid(unsafe_code)]

pub mod ast;
pub mod check;
pub mod text;

pub use text::{parse, print};

/// A rejected Core IR text. `line` is 1-based, or 0 when the error concerns
/// the program as a whole (well-formedness).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub line: usize,
    pub message: String,
}

impl Error {
    pub fn new(line: usize, message: impl Into<String>) -> Error {
        Error {
            line,
            message: message.into(),
        }
    }
}
