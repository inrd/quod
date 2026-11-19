//! `quodc`: the Quod compiler (phase 1: front end).
//!
//! Untrusted component (see `docs/architecture.md`). Deterministic: no hash
//! iteration, clocks or randomness reach any output.

#![forbid(unsafe_code)]

pub mod ast;
pub mod diag;
pub mod dump;
pub mod fmt;
pub mod lexer;
pub mod parser;

pub use diag::{Code, Diagnostic};

/// Parse and format a source file canonically.
pub fn format_source(src: &str) -> Result<String, Diagnostic> {
    Ok(fmt::format_file(&parser::parse(src)?))
}
