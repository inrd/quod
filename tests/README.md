# tests/

Each directory will hold `.qd` inputs with expected outputs (golden files) or expected diagnostics (by stable code).

| Directory | Purpose | Phase |
|---|---|---|
| `parse/` | grammar acceptance/rejection, formatter idempotence (`format(format(x)) == format(x)`) | 1 |
| `typecheck/` | positive and negative cases; every implicit conversion/overload attempt must be rejected | 1 |
| `emit/` | C output compiles with `-Wall -Wextra -Werror`, runs clean under UBSan/ASan | 1 |
| `obligations/` | expected obligation lists and stable IDs per input; ID stability under unrelated edits | 2–3 |
| `kernel-soundness/` | known-false obligations with bogus proofs: all must be rejected | 2 |
| `oracle/` | request/response transcripts validated against the protocol schema | 3 |
| `bench/` | agent-loop benchmark corpus (spec + `?h` body) and metrics | 9 |

Conventions (provisional): one case per file. Line 1 is the provisional-syntax header required for every `.qd` file; line 2 states the expected outcome (`// expect: ok`, `// expect: E0203`).

`parse/` is run by `cargo test` (`compiler/tests/corpus.rs`). For `ok` cases it checks parse success, formatter idempotence, `parse(format(x)) == parse(x)`, and that the file is already canonical (unless its name contains `noncanonical`). For error cases it checks the first diagnostic's code.
