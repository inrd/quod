//! Lowering corpus: `tests/lower/*.qd`.
//!
//! Line 2 is `// expect: ok` or `// expect: E0xxx`. An `ok` file lowers to
//! exactly its sibling `.core` file, which `coreir` accepts. The `find`
//! example also matches `spec/core-ir.md` §2 byte for byte.

use std::path::{Path, PathBuf};

use quodc::lower_source;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn lower_ok(path: &Path) -> String {
    let src = std::fs::read_to_string(path).unwrap();
    match lower_source(&src) {
        Ok(text) => text,
        Err(Err(d)) => panic!("{}", d.render(&path.display().to_string(), &src)),
        Err(Ok(internal)) => panic!("{}: {internal}", path.display()),
    }
}

#[test]
fn lower_corpus() {
    let mut files: Vec<PathBuf> = std::fs::read_dir(root().join("tests/lower"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "qd"))
        .collect();
    files.sort();
    assert!(!files.is_empty());
    for path in files {
        let src = std::fs::read_to_string(&path).unwrap();
        let expect = src
            .lines()
            .nth(1)
            .and_then(|l| l.strip_prefix("// expect: "));
        let golden = path.with_extension("core");
        match expect {
            Some("ok") => {
                let want = std::fs::read_to_string(&golden)
                    .unwrap_or_else(|_| panic!("missing {}", golden.display()));
                assert_eq!(lower_ok(&path), want, "{}", path.display());
            }
            Some(code) => {
                assert!(
                    !golden.exists(),
                    "{}: error case with a .core file",
                    path.display()
                );
                match lower_source(&src) {
                    Err(Err(d)) => assert_eq!(d.code.as_str(), code, "{}", path.display()),
                    other => panic!(
                        "{}: expected {code}, got {:?}",
                        path.display(),
                        other.map(|_| ())
                    ),
                }
            }
            None => panic!("{}: line 2 must be `// expect: …`", path.display()),
        }
    }
}

#[test]
fn spec_example() {
    let doc = std::fs::read_to_string(root().join("spec/core-ir.md")).unwrap();
    let marker = "Core IR text:\n\n```\n";
    let start = doc.find(marker).expect("example") + marker.len();
    let len = doc[start..].find("```").expect("end of example");
    assert_eq!(
        lower_ok(&root().join("tests/lower/ok_find.qd")),
        doc[start..start + len]
    );
}
