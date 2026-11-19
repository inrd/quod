//! Type-check corpus: `tests/typecheck/*.qd`.
//!
//! Line 2 is `// expect: ok` or `// expect: E03xx`. For `ok` files, line 3 is
//! `// traps: <kinds in source order>` or `// traps: none`, checked against
//! the unverified-checks report. Error files must parse and fail checking
//! with exactly that code.

use std::path::{Path, PathBuf};

use quodc::{check::check, parser::parse};

fn corpus() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/typecheck");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", root.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "qd"))
        .collect();
    files.sort();
    files
}

fn header<'a>(src: &'a str, line: usize, prefix: &str, path: &Path) -> &'a str {
    src.lines()
        .nth(line)
        .and_then(|l| l.strip_prefix(prefix))
        .unwrap_or_else(|| {
            panic!(
                "{}: line {} must start with `{prefix}`",
                path.display(),
                line + 1
            )
        })
        .trim()
}

fn run(path: &Path) -> Result<(), String> {
    let name = path.display().to_string();
    let src = std::fs::read_to_string(path).unwrap();
    let expect = header(&src, 1, "// expect: ", path);
    let file =
        parse(&src).map_err(|d| format!("{name}: parse error: {}", d.render(&name, &src)))?;
    match (expect, check(&file)) {
        ("ok", Ok(checked)) => {
            let want = header(&src, 2, "// traps: ", path);
            let got: Vec<&str> = checked.traps.iter().map(|t| t.kind.as_str()).collect();
            let got = if got.is_empty() {
                "none".to_string()
            } else {
                got.join(" ")
            };
            if got != want {
                return Err(format!(
                    "{name}: traps differ\n  want: {want}\n  got:  {got}\n{}",
                    quodc::check::render_report(&name, &src, &checked.traps)
                ));
            }
            Ok(())
        }
        ("ok", Err(d)) => Err(format!(
            "{name}: unexpected error: {}",
            d.render(&name, &src)
        )),
        (code, Ok(_)) => Err(format!("{name}: expected {code}, checked ok")),
        (code, Err(d)) if d.code.as_str() == code => Ok(()),
        (code, Err(d)) => Err(format!(
            "{name}: expected {code}, got {}",
            d.render(&name, &src)
        )),
    }
}

#[test]
fn typecheck_corpus() {
    let files = corpus();
    assert!(!files.is_empty());
    let failures: Vec<String> = files.iter().filter_map(|p| run(p).err()).collect();
    assert!(
        failures.is_empty(),
        "{} failure(s):\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

/// Every type-checker code in `spec/diagnostics.md` has at least one negative test.
#[test]
fn every_code_tested() {
    let names: Vec<String> = corpus()
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    let missing: Vec<String> = (301..=325)
        .map(|n| format!("E0{n}"))
        .filter(|c| !names.iter().any(|f| f.starts_with(&format!("err_{c}"))))
        .collect();
    assert!(missing.is_empty(), "codes without a test: {missing:?}");
}

/// The phase-1 example type-checks.
#[test]
fn example_01_checks() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/01_core_max.qd");
    let src = std::fs::read_to_string(path).unwrap();
    let checked = check(&parse(&src).unwrap()).unwrap();
    let kinds: Vec<&str> = checked.traps.iter().map(|t| t.kind.as_str()).collect();
    assert_eq!(kinds, ["bounds", "bounds", "bounds", "cast"]);
}
