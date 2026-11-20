//! Core IR corpus: `tests/coreir/ok_*.core` must parse (canonical and
//! well-formed) and round-trip; the example in `spec/core-ir.md` §2 too;
//! the texts in `rejected()` must fail with the given message fragment.

use std::path::Path;

use coreir::{parse, print};

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
}

fn accept(name: &str, src: &str) {
    match parse(src) {
        Ok(p) => assert_eq!(print(&p), src, "{name}: round trip"),
        Err(e) => panic!("{name}:{}: {}", e.line, e.message),
    }
}

#[test]
fn ok_corpus() {
    let dir = root().join("tests/coreir");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "core"))
        .collect();
    files.sort();
    assert!(!files.is_empty());
    for f in files {
        accept(
            &f.display().to_string(),
            &std::fs::read_to_string(&f).unwrap(),
        );
    }
}

#[test]
fn spec_example() {
    let doc = std::fs::read_to_string(root().join("spec/core-ir.md")).unwrap();
    let start =
        doc.find("Core IR text:\n\n```\n").expect("example") + "Core IR text:\n\n```\n".len();
    let len = doc[start..].find("```").expect("end of example");
    accept("spec/core-ir.md §2", &doc[start..start + len]);
}

const FIND: &str = "(quod-core 0)

(fn f
  (params (x u32))
  (ret u32)
  (requires)
  (ensures)
  (body
    (return x)))
";

/// Replace `from` with `to` in a minimal valid program.
fn variant(from: &str, to: &str) -> String {
    assert!(FIND.contains(from), "{from}");
    FIND.replacen(from, to, 1)
}

#[test]
fn rejected() {
    accept("base", FIND);
    let cases: Vec<(String, &str)> = vec![
        // Canonical form (§3.2).
        (FIND.replace("  (ret", "   (ret"), "canonical"),
        (variant("(return x)))\n", "(return x)))"), "canonical"),
        (variant("(requires)", "(requires\n    )"), "canonical"),
        (variant("(return x)", "(return 007u32)"), "canonical"),
        (FIND.replace("(quod-core 0)", "(quod-core 1)"), "version"),
        (variant("(return x)))", "(return x))))"), "unbalanced"),
        // Names.
        (variant("(x u32)", "(x.0 u32)"), "not a valid name"),
        (variant("(x u32)", "(result u32)"), "not a valid name"),
        (
            variant("(return x)", "(let x u32 1u32)\n    (return x)"),
            "more than once",
        ),
        (variant("(return x)", "(return y)"), "not defined"),
        // Types and operations.
        (
            variant("(return x)", "(return 4294967296u32)"),
            "does not fit",
        ),
        (
            variant(
                "(return x)",
                "(let $0 u32 (add.u32 x true))\n    (return $0)",
            ),
            "must have type",
        ),
        (
            variant("(return x)", "(let $0 u32 (not.usize x))\n    (return $0)"),
            "outside the slice",
        ),
        (
            variant(
                "(return x)",
                "(let $0 bool (lt.bool true false))\n    (return x)",
            ),
            "not defined on",
        ),
        (
            variant("(return x)", "(set x 1u32)\n    (return x)"),
            "scalar `var`",
        ),
        (
            variant("(return x)", "(let $0 u32 (call f x))\n    (return $0)"),
            "recursion",
        ),
        (
            variant("(return x)", "(let $0 u32 (call g x))\n    (return $0)"),
            "unknown function",
        ),
        // Control flow.
        (
            variant(
                "(return x)",
                "(if true\n      (then\n        (return x))\n      (else))",
            ),
            "does not end in `return`",
        ),
        (
            variant("(return x)", "(return x)\n    (return x)"),
            "after `return`",
        ),
        (variant("(return x)", "(return)"), "return type"),
        // Spec terms.
        (variant("(requires)", "(requires\n    result)"), "`result`"),
        (
            variant("(requires)", "(requires\n    (= (old x) x))"),
            "`old`",
        ),
        (variant("(ensures)", "(ensures\n    (= x x))"), "compare"),
        (
            variant("(ensures)", "(ensures\n    (< x 1))"),
            "must have type",
        ),
        (
            variant("(ensures)", "(ensures\n    (= (old (old x)) x))"),
            "`old`",
        ),
        (
            variant("(ensures)", "(ensures\n    (forall x 0 1 true))"),
            "shadows",
        ),
        (
            variant(
                "(ensures)",
                "(ensures\n    (= (int (shl.usize 1usize 2usize)) 4))",
            ),
            "not available",
        ),
    ];
    for (src, want) in cases {
        match parse(&src) {
            Ok(_) => panic!("accepted, expected `{want}`:\n{src}"),
            Err(e) => assert!(
                e.message.contains(want),
                "expected `{want}`, got `{}` for:\n{src}",
                e.message
            ),
        }
    }
}
