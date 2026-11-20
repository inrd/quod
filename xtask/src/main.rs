//! Repository checks for Quod. Run with `cargo xtask lint`.
//!
//! Every check enforces a rule already written in `CLAUDE.md` or an accepted
//! ADR. Output is sorted and deterministic; the exit code is 1 if any check
//! fails. Standard library only.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Trusted crates and their line budgets (non-test lines), from accepted ADRs.
/// Add `vcgen` (3,000) and `cgen` (2,000) when ADR-0007 and ADR-0010 are accepted.
const TRUSTED_CRATES: &[(&str, usize)] = &[("kernel", 5_000)]; // ADR-0001

/// Tokens forbidden in trusted crates (ADR-0001: determinism, no I/O, no unsafe).
const TRUSTED_FORBIDDEN: &[&str] = &[
    "HashMap",
    "HashSet",
    "f32",
    "f64",
    "std::thread",
    "std::time",
    "std::env",
    "std::fs",
    "std::net",
    "std::process",
    "unsafe ",
];

const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", ".claude"];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("lint") => lint(),
        _ => {
            eprintln!("usage: cargo xtask lint");
            ExitCode::from(2)
        }
    }
}

struct Report {
    errors: Vec<String>,
}

impl Report {
    fn err(&mut self, path: &Path, line: usize, msg: impl Into<String>) {
        let loc = if line == 0 {
            path.display().to_string()
        } else {
            format!("{}:{}", path.display(), line)
        };
        self.errors.push(format!("{loc}: {}", msg.into()));
    }
}

fn lint() -> ExitCode {
    let root = repo_root();
    if let Err(e) = std::env::set_current_dir(&root) {
        eprintln!("cannot enter {}: {e}", root.display());
        return ExitCode::from(2);
    }
    let files = walk(Path::new("."));
    let mut r = Report { errors: Vec::new() };

    check_links(&files, &mut r);
    check_adr_index(&mut r);
    check_diagnostics(&files, &mut r);
    check_qd_headers(&files, &mut r);
    check_trusted_crates(&files, &mut r);

    r.errors.sort();
    r.errors.dedup();
    for e in &r.errors {
        println!("{e}");
    }
    if r.errors.is_empty() {
        println!("xtask lint: ok");
        ExitCode::SUCCESS
    } else {
        println!("xtask lint: {} problem(s)", r.errors.len());
        ExitCode::FAILURE
    }
}

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is xtask/; the repository root is its parent.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().map(Path::to_path_buf).unwrap_or(manifest)
}

/// All files under `dir`, sorted, without `./` prefix.
fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            if p.is_dir() {
                if !SKIP_DIRS.contains(&name.as_str()) {
                    stack.push(p);
                }
            } else {
                out.push(p.strip_prefix("./").unwrap_or(&p).to_path_buf());
            }
        }
    }
    out.sort();
    out
}

fn has_ext(p: &Path, ext: &str) -> bool {
    p.extension().is_some_and(|e| e == ext)
}

fn read(p: &Path) -> String {
    fs::read_to_string(p).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Markdown links and anchors

/// Lines of a Markdown file outside fenced code blocks, with inline code removed.
fn prose_lines(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut fence: Option<&str> = None;
    for (i, line) in text.lines().enumerate() {
        let t = line.trim_start();
        let marker = if t.starts_with("```") {
            Some("```")
        } else if t.starts_with("~~~") {
            Some("~~~")
        } else {
            None
        };
        match (fence, marker) {
            (None, Some(m)) => {
                fence = Some(m);
                continue;
            }
            (Some(f), Some(m)) if f == m => {
                fence = None;
                continue;
            }
            (Some(_), _) => continue,
            _ => {}
        }
        out.push((i + 1, strip_inline_code(line)));
    }
    out
}

fn strip_inline_code(line: &str) -> String {
    let mut out = String::new();
    let mut in_code = false;
    for c in line.chars() {
        if c == '`' {
            in_code = !in_code;
        } else if !in_code {
            out.push(c);
        }
    }
    out
}

/// GitHub-style heading anchors of a Markdown file, with `-1`, `-2` for duplicates.
fn anchors(text: &str) -> BTreeSet<String> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut out = BTreeSet::new();
    let mut fence = false;
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            fence = !fence;
            continue;
        }
        if fence || !t.starts_with('#') {
            continue;
        }
        let title = t.trim_start_matches('#');
        if !title.starts_with(' ') {
            continue;
        }
        let s = slug(title.trim());
        let n = seen.entry(s.clone()).or_insert(0);
        out.insert(if *n == 0 {
            s.clone()
        } else {
            format!("{s}-{n}")
        });
        *n += 1;
    }
    out
}

fn slug(title: &str) -> String {
    let mut s = String::new();
    for c in title.to_lowercase().chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            s.push(c);
        } else if c == ' ' {
            s.push('-');
        }
    }
    s
}

/// Link targets `](target)` in one line of prose.
fn link_targets(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(i) = rest.find("](") {
        rest = &rest[i + 2..];
        let end = rest.find([')', ' ']).unwrap_or(rest.len());
        out.push(rest[..end].to_string());
        rest = &rest[end..];
    }
    out
}

fn check_links(files: &[PathBuf], r: &mut Report) {
    let mut anchor_cache: BTreeMap<PathBuf, BTreeSet<String>> = BTreeMap::new();
    for f in files.iter().filter(|f| has_ext(f, "md")) {
        let text = read(f);
        let dir = f.parent().unwrap_or(Path::new(""));
        for (ln, line) in prose_lines(&text) {
            for target in link_targets(&line) {
                if target.is_empty()
                    || target.contains("://")
                    || target.starts_with("mailto:")
                    || target.starts_with('<')
                {
                    continue;
                }
                let (path_part, anchor) = match target.split_once('#') {
                    Some((p, a)) => (p, Some(a)),
                    None => (target.as_str(), None),
                };
                let dest = if path_part.is_empty() {
                    f.clone()
                } else {
                    dir.join(path_part)
                };
                if !dest.exists() {
                    r.err(f, ln, format!("broken link `{target}`"));
                    continue;
                }
                let Some(a) = anchor else { continue };
                if !has_ext(&dest, "md") {
                    continue;
                }
                let key = fs::canonicalize(&dest).unwrap_or(dest.clone());
                let set = anchor_cache
                    .entry(key)
                    .or_insert_with(|| anchors(&read(&dest)));
                if !set.contains(a) {
                    r.err(f, ln, format!("broken anchor `{target}`"));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ADR index

struct Adr {
    num: String,
    file: String,
    title: String,
    status: String,
}

fn status_word(s: &str) -> &str {
    s.split(|c: char| !c.is_alphabetic())
        .find(|w| !w.is_empty())
        .unwrap_or("")
}

fn check_adr_index(r: &mut Report) {
    let dir = Path::new("docs/adr");
    let index_path = dir.join("README.md");
    let mut adrs: BTreeMap<String, Adr> = BTreeMap::new();
    for f in walk(dir) {
        let name = f
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if !has_ext(&f, "md") || name == "README.md" || name.starts_with("0000-") {
            continue;
        }
        let num: String = name.chars().take_while(char::is_ascii_digit).collect();
        if num.len() != 4 {
            r.err(&f, 0, "ADR file name must start with a 4-digit number");
            continue;
        }
        let text = read(&f);
        let mut title = None;
        let mut status = None;
        for line in text.lines() {
            if title.is_none() {
                if let Some(t) = line.strip_prefix(&format!("# ADR-{num}: ")) {
                    title = Some(t.trim().to_string());
                }
            }
            if status.is_none() {
                if let Some(s) = line.strip_prefix("- **Status:** ") {
                    status = Some(s.trim().to_string());
                }
            }
        }
        let Some(title) = title else {
            r.err(
                &f,
                1,
                format!("first heading must be `# ADR-{num}: <title>`"),
            );
            continue;
        };
        let Some(status) = status else {
            r.err(&f, 0, "missing `- **Status:** …` line");
            continue;
        };
        if !["Proposed", "Accepted", "Rejected", "Superseded"].contains(&status_word(&status)) {
            r.err(&f, 0, format!("unknown status `{status}`"));
        }
        if status_word(&status) == "Superseded" {
            match status.split("ADR-").nth(1) {
                Some(n) if n.len() >= 4 => {
                    let n = &n[..4];
                    let exists = walk(dir).iter().any(|p| {
                        p.file_name()
                            .is_some_and(|x| x.to_string_lossy().starts_with(&format!("{n}-")))
                    });
                    if !exists {
                        r.err(
                            &f,
                            0,
                            format!("superseded by ADR-{n}, which does not exist"),
                        );
                    }
                }
                _ => r.err(&f, 0, "`Superseded` status must name `ADR-NNNN`"),
            }
        }
        if adrs.contains_key(&num) {
            r.err(&f, 0, format!("duplicate ADR number {num}"));
        }
        adrs.insert(
            num.clone(),
            Adr {
                num,
                file: name,
                title,
                status,
            },
        );
    }

    // Numbers are sequential from 0001.
    for (i, num) in adrs.keys().enumerate() {
        let expected = format!("{:04}", i + 1);
        if *num != expected {
            r.err(
                dir,
                0,
                format!("ADR numbers not sequential: expected {expected}, found {num}"),
            );
            break;
        }
    }

    // Index rows: | [NNNN](file) | Title | Status |
    let index = read(&index_path);
    let mut rows: BTreeMap<String, (usize, String, String, String)> = BTreeMap::new();
    for (i, line) in index.lines().enumerate() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() < 5 || !cells[1].starts_with('[') {
            continue;
        }
        let Some((label, link)) = cells[1].strip_prefix('[').and_then(|c| c.split_once("]("))
        else {
            continue;
        };
        let link = link.trim_end_matches(')');
        rows.insert(
            label.to_string(),
            (
                i + 1,
                link.to_string(),
                cells[2].to_string(),
                cells[3].to_string(),
            ),
        );
    }
    for adr in adrs.values() {
        let Some((ln, link, title, status)) = rows.get(&adr.num) else {
            r.err(
                &index_path,
                0,
                format!("ADR-{} ({}) missing from the index", adr.num, adr.file),
            );
            continue;
        };
        if *link != adr.file {
            r.err(
                &index_path,
                *ln,
                format!("ADR-{} links `{link}`, file is `{}`", adr.num, adr.file),
            );
        }
        if !title.starts_with(&adr.title) {
            r.err(
                &index_path,
                *ln,
                format!(
                    "ADR-{} title `{title}` does not start with the file's title `{}`",
                    adr.num, adr.title
                ),
            );
        }
        if status_word(status) != status_word(&adr.status) {
            r.err(
                &index_path,
                *ln,
                format!(
                    "ADR-{} status `{status}` but the file says `{}`",
                    adr.num, adr.status
                ),
            );
        }
    }
    for (num, (ln, ..)) in &rows {
        if !adrs.contains_key(num) {
            r.err(
                &index_path,
                *ln,
                format!("index lists ADR-{num}, which has no file"),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Diagnostic code registry

fn code_at(s: &str) -> Option<String> {
    let b = s.as_bytes();
    if b.len() >= 5 && b[0] == b'E' && b[1..5].iter().all(u8::is_ascii_digit) {
        Some(s[..5].to_string())
    } else {
        None
    }
}

fn check_diagnostics(files: &[PathBuf], r: &mut Report) {
    let spec_path = Path::new("spec/diagnostics.md");
    let rs_path = Path::new("compiler/src/diag.rs");

    // Spec: first column of table rows, written as `E0101`.
    let mut spec: BTreeMap<String, usize> = BTreeMap::new();
    for (i, line) in read(spec_path).lines().enumerate() {
        if let Some(c) = line.strip_prefix("| `").and_then(code_at) {
            if spec.insert(c.clone(), i + 1).is_some() {
                r.err(spec_path, i + 1, format!("{c} listed twice"));
            }
        }
    }
    // Code: variants of `enum Code`, one per line as `    E0101,`.
    let mut code: BTreeMap<String, usize> = BTreeMap::new();
    let mut in_enum = false;
    for (i, line) in read(rs_path).lines().enumerate() {
        if line.starts_with("pub enum Code") {
            in_enum = true;
            continue;
        }
        if in_enum && line.starts_with('}') {
            break;
        }
        if in_enum {
            if let Some(c) = code_at(line.trim()) {
                code.insert(c, i + 1);
            }
        }
    }
    if code.is_empty() {
        r.err(rs_path, 0, "no variants found in `pub enum Code`");
    }
    for (c, ln) in &code {
        if !spec.contains_key(c) {
            r.err(
                rs_path,
                *ln,
                format!("{c} is not registered in spec/diagnostics.md"),
            );
        }
    }
    for (c, ln) in &spec {
        if !code.contains_key(c) {
            r.err(
                spec_path,
                *ln,
                format!("{c} has no variant in compiler/src/diag.rs"),
            );
        }
    }
    // Test inputs named err_EXXXX… must use a registered code.
    for f in files.iter().filter(|f| has_ext(f, "qd")) {
        let name = f.file_name().unwrap_or_default().to_string_lossy();
        if let Some(c) = name.strip_prefix("err_").and_then(code_at) {
            if !spec.contains_key(&c) {
                r.err(f, 0, format!("test expects {c}, which is not registered"));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// `.qd` provisional-syntax header

fn check_qd_headers(files: &[PathBuf], r: &mut Report) {
    for f in files.iter().filter(|f| has_ext(f, "qd")) {
        let text = read(f);
        let first = text.lines().next().unwrap_or("");
        if !(first.starts_with("//") && first.to_lowercase().contains("provisional")) {
            r.err(
                f,
                1,
                "first line must be a comment saying the syntax is provisional",
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Trusted crates: line budget and forbidden constructs

fn check_trusted_crates(files: &[PathBuf], r: &mut Report) {
    for (krate, budget) in TRUSTED_CRATES {
        let src = Path::new(krate).join("src");
        let rs: Vec<&PathBuf> = files
            .iter()
            .filter(|f| f.starts_with(&src) && has_ext(f, "rs"))
            .collect();
        if rs.is_empty() {
            continue; // crate not started yet
        }
        let mut total = 0;
        for f in &rs {
            for (i, line) in read(f).lines().enumerate() {
                if line.trim_start().starts_with("#[cfg(test)]") {
                    break; // inline test modules sit at the end of the file
                }
                total += 1;
                let code = line.split("//").next().unwrap_or("");
                for tok in TRUSTED_FORBIDDEN {
                    if contains_token(code, tok) {
                        r.err(
                            f,
                            i + 1,
                            format!(
                                "`{}` is forbidden in the trusted crate `{krate}`",
                                tok.trim()
                            ),
                        );
                    }
                }
            }
        }
        if total > *budget {
            r.err(
                &src,
                0,
                format!("{total} non-test lines, budget is {budget}"),
            );
        }
        let lib = src.join("lib.rs");
        if !read(&lib).contains("#![forbid(unsafe_code)]") {
            r.err(&lib, 0, "missing `#![forbid(unsafe_code)]`");
        }
        let manifest = read(&Path::new(krate).join("Cargo.toml"));
        if let Some(deps) = manifest.split("[dependencies]").nth(1) {
            let section = deps.split("\n[").next().unwrap_or("");
            if section
                .lines()
                .any(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
            {
                r.err(
                    &Path::new(krate).join("Cargo.toml"),
                    0,
                    "trusted crates take no dependencies",
                );
            }
        }
    }
}

/// `tok` occurs in `line` not as part of a longer identifier.
fn contains_token(line: &str, tok: &str) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    line.match_indices(tok).any(|(i, _)| {
        let before = line[..i].chars().next_back();
        let after = line[i + tok.len()..].chars().next();
        let starts_inside_ident = before.is_some_and(ident);
        let ends_inside_ident = tok.ends_with(ident) && after.is_some_and(ident);
        !starts_inside_ident && !ends_inside_ident
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_match_github() {
        assert_eq!(
            slug("14. Surface syntax: own or Rust-shaped"),
            "14-surface-syntax-own-or-rust-shaped"
        );
        assert_eq!(
            slug("1. Two strata: spec and impl + proofs"),
            "1-two-strata-spec-and-impl--proofs"
        );
        assert_eq!(
            slug("`proof/submit` — kernel check"),
            "proofsubmit--kernel-check"
        );
    }

    #[test]
    fn duplicate_headings_get_suffixes() {
        let a = anchors("# A\n## A\n```\n# A\n```\n");
        assert!(a.contains("a") && a.contains("a-1") && !a.contains("a-2"));
    }

    #[test]
    fn links_skip_inline_code() {
        let line = strip_inline_code("see [x](a.md#b) and `[y](c.md)`");
        assert_eq!(link_targets(&line), vec!["a.md#b".to_string()]);
    }

    #[test]
    fn tokens_respect_identifiers() {
        assert!(contains_token("let x: f64 = 0;", "f64"));
        assert!(!contains_token("let buf64 = 0;", "f64"));
        assert!(!contains_token("#![forbid(unsafe_code)]", "unsafe "));
        assert!(contains_token("unsafe { x }", "unsafe "));
    }
}
