//! `quodc`: the Quod compiler (phase 1: front end).
//!
//! Untrusted component (see `docs/architecture.md`). Deterministic: no hash
//! iteration, clocks or randomness reach any output.

#![forbid(unsafe_code)]

pub mod ast;
pub mod check;
pub mod diag;
pub mod dump;
pub mod fmt;
pub mod lexer;
pub mod lower;
pub mod parser;
pub mod types;

pub use diag::{Code, Diagnostic};

/// Parse, check and lower a source file to canonical Core IR text, then
/// re-read that text through `coreir` (§3.4 checks). A failure of the
/// re-read is a lowering bug and is returned as `Err(Ok(message))`.
pub fn lower_source(src: &str) -> Result<String, Result<String, Diagnostic>> {
    let file = parser::parse(src).map_err(Err)?;
    let checked = check::check(&file).map_err(Err)?;
    let program = lower::lower(&file, &checked).map_err(Err)?;
    let text = coreir::print(&program);
    coreir::parse(&text).map_err(|e| {
        Ok(format!(
            "lowering produced invalid Core IR (line {}): {}",
            e.line, e.message
        ))
    })?;
    Ok(text)
}

/// Parse and format a source file canonically.
pub fn format_source(src: &str) -> Result<String, Diagnostic> {
    Ok(fmt::format_file(&parser::parse(src)?))
}
