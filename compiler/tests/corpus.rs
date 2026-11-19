//! Parse corpus: `tests/parse/*.qd`.
//!
//! Each file starts with the provisional-syntax header; line 2 is
//! `// expect: ok` or `// expect: E0xxx`. For `ok` files:
//! - `format(format(x)) == format(x)` (idempotence);
//! - `parse(format(x)) == parse(x)` (AST equality via the structural dump);
//! - `format(x) == x` unless the file name contains `noncanonical`.
//!
//! For error files: parsing fails with exactly that code.

use std::path::{Path, PathBuf};

use quodc::{dump::dump, fmt::format_file, parser::parse};

fn corpus(dir: &str) -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(dir);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", root.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "qd"))
        .collect();
    files.sort();
    files
}

fn expectation(src: &str, path: &Path) -> String {
    let line = src.lines().nth(1).unwrap_or("");
    line.strip_prefix("// expect: ")
        .unwrap_or_else(|| panic!("{}: line 2 must be `// expect: …`", path.display()))
        .trim()
        .to_string()
}

fn check_ok(src: &str, path: &Path) -> Result<(), String> {
    let name = path.display();
    let ast = parse(src).map_err(|d| {
        format!(
            "{name}: unexpected error: {}",
            d.render(&name.to_string(), src)
        )
    })?;
    let once = format_file(&ast);
    let reparsed = parse(&once).map_err(|d| {
        format!(
            "{name}: formatted output does not parse: {}\n{once}",
            d.render("<formatted>", &once)
        )
    })?;
    if dump(&reparsed) != dump(&ast) {
        return Err(format!(
            "{name}: parse(format(x)) != parse(x)\n{}\n---\n{}",
            dump(&ast),
            dump(&reparsed)
        ));
    }
    let twice = format_file(&reparsed);
    if twice != once {
        return Err(format!(
            "{name}: formatter not idempotent\n{once}\n---\n{twice}"
        ));
    }
    let canonical = !path.to_string_lossy().contains("noncanonical");
    if canonical && once != src {
        return Err(format!(
            "{name}: file is not in canonical format; formatted:\n{once}"
        ));
    }
    Ok(())
}

#[test]
fn parse_corpus() {
    let files = corpus("tests/parse");
    assert!(!files.is_empty());
    let mut failures = Vec::new();
    for path in &files {
        let src = std::fs::read_to_string(path).unwrap();
        let expect = expectation(&src, path);
        let result = if expect == "ok" {
            check_ok(&src, path)
        } else {
            match parse(&src) {
                Ok(_) => Err(format!("{}: expected {expect}, parsed ok", path.display())),
                Err(d) if d.code.as_str() == expect => Ok(()),
                Err(d) => Err(format!(
                    "{}: expected {expect}, got {}",
                    path.display(),
                    d.render("", &src)
                )),
            }
        };
        if let Err(e) = result {
            failures.push(e);
        }
    }
    assert!(
        failures.is_empty(),
        "{} failure(s):\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

/// Phase-1 example sketches must parse, round-trip and be canonical.
#[test]
fn phase1_examples() {
    let examples: &[&str] = &["01_core_max.qd"];
    for name in examples {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../examples")
            .join(name);
        let src = std::fs::read_to_string(&path).unwrap();
        if let Err(e) = check_ok(&src, &path) {
            panic!("{e}");
        }
    }
}
