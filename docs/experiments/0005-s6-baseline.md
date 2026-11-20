# Experiment 0005: S6 baseline — agents on the slice tasks in Dafny and Verus

- **Date:** 2026-11-20
- **Step:** baseline half of S6 of [ADR-0006](../adr/0006-thin-end-to-end-slice.md) (Accepted)
- **Tests:** nothing by itself. It supplies the baseline numbers that the Quod half of S6 is compared against for [ADR-0013](../adr/0013-annotations-and-small-obligations.md) (stability, annotations vs. steps) and [ADR-0011](../adr/0011-v1-scope.md) decision 1 (pool vs. pointer tasks)
- **Author:** Claude (agent), at the maintainer's request
- **Inputs:** [`0005-s6-baseline/`](0005-s6-baseline/): task files, harness (`run.sh`, `verify.sh`, `prompt.txt`), analysis (`analyze.py`), per-run records

## Question

How well does an agent do on the S6 tasks in existing verifiers, measured with the S6 metrics (success, verifier round trips, tokens, proof size, edit and error classes) and ADR-0013's stability measure? The Quod half of S6 runs the same tasks, with the same agent and prompt, once S3–S5 exist.

## Protocol (fixed before the first run)

### Tasks

Ten tasks, written first in Quod's slice subset ([contracts.md §7](../../spec/contracts.md#7-slice-subset)) and checked with `quodc check` (only `E0321`, the hole, is reported), then translated to Dafny and Verus by hand. Two extra pointer-linked tasks exist only in the baseline (ADR-0011's gate compares pool tasks in Quod against pointer tasks in the baseline).

| Task | Kind | What it needs |
|---|---|---|
| t01 `max_index` | array scan | invariant with an existential witness index |
| t02 `all_below` | array scan, `bool` result | early return, `exists` postcondition |
| t03 `lower_bound` | binary search | sortedness precondition, overflow-safe midpoint |
| t04 `reverse` | in-place `&mut` | two-pointer swap, `old` frame |
| t05 `copy_at` | two slices | offset arithmetic, frame on both sides |
| t06 `scale` | overflow precondition | multiplication bound carried through the loop |
| t07 `partition` | in-place `&mut` | two-sided invariant; the spec has no permutation clause (no multisets in the slice), so the body is reviewed by hand |
| t08 `insert_sorted` | shift and insert | nested-quantifier sortedness, index shift by one |
| t09 `pool_insert_after` | pool (parallel arrays, index links) | frame over two arrays |
| t10 `pool_alloc` | pool free list | head in a 1-element slice (no `&mut T` in the slice) |
| t09p `ptr_insert_after` | pointer-linked (baseline only) | Dafny: class with `fresh`; Verus: `Option<Box<Node>>` |
| t10p `ptr_alloc` | pointer-linked (baseline only) | pop from a pointer free list |

Translation rules: machine integers are `u32`/`usize` subset types in Dafny (with `xs.Length < 2^64` as a precondition, since a Quod slice length is a `usize`) and native `u32`/`usize` in Verus, which checks overflow natively. Quod slices are Dafny `array`s (with `modifies` and, where two arrays appear, a non-aliasing precondition) and Verus slices. Quantifiers keep their ranges; Verus's sortedness clauses use the idiomatic two-variable form with explicit triggers, which is logically equivalent. Verus needs `final(x)` for the post-state of a `&mut` parameter.

### Runs

| Setting | Value |
|---|---|
| Agent | Claude Code 2.1.326, headless (`claude -p`), `--safe-mode` (no CLAUDE.md, memory, skills, hooks), no MCP servers, no session persistence |
| Model | `claude-sonnet-5-5`, default effort; the Quod half uses the same |
| Tools | Read, Edit, Write; Bash allowed only for `./verify` |
| Limits | 20 `./verify` calls (by instruction), $5 per run (`--max-budget-usd`), 300 s per verifier call |
| Isolation | each run in a fresh temporary directory outside the repository holding only the task file and `./verify` |
| Verifiers | Dafny 4.11.0 (bundled Z3), Verus 0.2026.11.02.8bf68ee (Rust 1.99.1), pinned in [`tools/pins.sh`](../../tools/pins.sh) |
| Repetitions | one run per task and verifier (24 runs). Agent behavior is not deterministic; one run per cell is an informal measurement, as ADR-0006 allows for S6 |

The prompt is [`prompt.txt`](0005-s6-baseline/prompt.txt).

### Success

A run succeeds when all hold, checked by `analyze.py` after the session ends, not by the agent's own claim:

1. The verifier, re-run on the final file, reports no errors.
2. The text between `SPEC-BEGIN` and `SPEC-END` is byte-identical to the task file.
3. No construct that skips verification: Dafny `assume`, `{:axiom}`, `{:verify false}`, `expect`; Verus `assume`, `admit`, `external_body`, `external`, `#[verifier::exec_allows_no_decreases_clause]`.
4. For t07, a hand review confirms the body only swaps elements.

### Metrics

| Metric | Source |
|---|---|
| success | above |
| round trips | number of `./verify` calls |
| tokens, cost, turns, wall time | session JSON |
| proof size | non-blank, non-comment lines in the final file outside the skeleton, split into code, annotation and proof-step lines |

Line classes for the final file, by keyword:

| Class | Dafny | Verus |
|---|---|---|
| annotation | `invariant`, `decreases`, `assert` without `by`, ghost `var`, `lemma`/`predicate`/`function` signatures and `requires`/`ensures` of helpers, lemma calls | `invariant`, `decreases`, `assert` without `by`, `ghost`/`tracked` lets, `proof fn`/`spec fn` signatures and their clauses, lemma calls |
| proof step | lines in a lemma body, `calc`, `assert … by { … }` bodies, `reveal` | lines in a `proof fn` body, `proof { … }` blocks, `assert … by` bodies, `reveal` |
| code | everything else | everything else |

ADR-0013's criterion is about Quod; for the baseline the same split is reported for comparison.

### Error classes

Each failing `./verify` output is classified by its first error:

| Class | Dafny | Verus |
|---|---|---|
| syntax | parse errors | Rust parse errors |
| typing | resolution and type errors, definite assignment | `rustc` type and borrow errors, Verus "unsupported", mode errors |
| false friend | — (no host language) | a Rust construct Verus rejects (`unsupported`, iterator/`for`, closures, std calls without specs) |
| verification | postcondition, invariant, assertion, precondition, subset-type, decreases, timeout | same, plus overflow ("possible arithmetic underflow/overflow") and trigger errors |

### Stability (ADR-0013)

For each successful run, three semantics-preserving edits are applied mechanically to the final file and the verifier is re-run once per edit:

| Edit | Change |
|---|---|
| E1 rename | the first local variable declared in the body is renamed (`x` → `x_q`) everywhere in the body |
| E2 reorder | the first two `invariant` clauses of the first loop swap places (if there are two) |
| E3 unrelated | an unrelated, verified function is appended to the file |

Reported: the share of edits after which verification fails. ADR-0013's pass criterion compares Quod's share against this one.

## Results

Protocol revision 1, made before the counted runs: in a first batch (kept in `runs-v0/`, not counted), two Verus agents combined a file write with `./verify` in one shell command, were refused, and gave up without verifying. [`prompt.txt`](0005-s6-baseline/prompt.txt) now says `./verify` must run on its own. All 24 cells were rerun with it (`runs/`). Per-run tables: [`results/`](0005-s6-baseline/results/).

| | Dafny 4.11.0 | Verus 0.2026.11.02 |
|---|---|---|
| solved (protocol's success rule) | **11/12**: t06 verified for the agent but failed the re-check (see Determinism) | **12/12** |
| verified on the first `./verify` call | 11/12 | 8/12 |
| `./verify` calls, all tasks | 13 | 18 |
| errors by class | 1 verification | 2 syntax, 4 typing, 0 false friend, 0 verification |
| hand-written proof-step lines | 0 | 0 |
| added lines in solved runs (code / annotation) | 58 / 29 | 84 / 50 |
| cost, all 12 runs | $0.21 | $0.26 |
| tokens per run (in, mostly cached / out) | 20–28k / 360–880 | 20–39k / 370–1,560 |
| wall time per run | 9–15 s | 9–20 s |
| stability edits that broke verification | **4/25**, all E2 (4 of 7 applicable) | **0/32** |

### Findings

1. **The tasks do not separate anything.** 23 of 24 cells were solved, mostly on the first call and with no proof steps. Even an ideal Quod would at best tie, so S6 cannot show an advantage with these tasks. The S6 task set needs harder tasks: nested loops with permutation or counting specs, multi-function tasks where callee contracts matter, and pool tasks that traverse links. Several of those need constructs the slice lacks (`spec fn`, ghost parameters, multisets). That is itself a finding: **the slice subset can state only local, easy properties**. Traversing an index-linked list cannot be specified without ghost state, so the pool tasks here are single-step operations.
2. **Verus errors were all front-end errors** (syntax, typing): `set` on `&mut [u32]` needs `use vstd::slice::SliceAdditionalExecFns`, `final(x)` vs `old(x)`, mode errors. This is the cost of a Rust-shaped surface with verifier-specific rules, which [open-questions §14](../open-questions.md#14-surface-syntax-own-or-rust-shaped) asks about. Dafny's single error was a verification error.
3. **Invariant order matters in Dafny (E2, 4/25).** Dafny checks that each invariant is well-formed (here: array index in range) assuming only the invariants before it, so swapping `0 <= i <= n` below a quantified invariant breaks verification. Verus spec indexing is total, so the same edit never failed there (0/8 applicable E2 edits). Quod's spec indexing is also total in spec contexts ([core-ir.md](../../spec/core-ir.md)), so it should behave like Verus.
4. **Pool vs. pointer (ADR-0011):** all four pool cells and all four pointer cells were solved on the first or second call. No difference, but these pool tasks are too local for the gate to mean much (finding 1).

### Determinism (not pre-registered)

Added after the runs, because Dafny t06's final file verified when the agent ran it and failed the protocol's re-check, byte-identical. [`repeat.sh`](0005-s6-baseline/repeat.sh) re-verifies every final solution 10 times in sequence:

| Solutions | Result of 10 sequential re-verifications |
|---|---|
| 23 of 24 (all Verus, Dafny except t06) | 10/10 pass |
| Dafny t06 | **8/10 pass** |

Earlier ad-hoc re-runs of the same t06 file gave 3/6 and 4/5, so 15 of 21 passes over all its re-runs (71%). No other solution ever failed a re-run.

Dafny's verdict on the t06 file is not a function of the file. The solution itself is correct; Z3 sometimes fails to prove the subset-type check for `xs[i] * f` from the quantified precondition. This is the failure mode ADR-0009 (`quodc check` checks stored certificates only) is designed to rule out: an agent's "verified" claim that CI later rejects with no change.

## Conclusion

- **Baseline is strong on easy tasks:** 23/24 solved for about $0.02 a run. S6 must be compared on harder tasks, which need a wider slice or ghost parameters (to be proposed separately, not decided here).
- **Evidence for Quod's design, gathered from the baseline side:** solver non-determinism on an unchanged file (Dafny t06), invariant-order sensitivity (Dafny E2), and front-end errors caused by verifier-specific rules on top of a host language (Verus). Each is a measurement of the baseline only; whether Quod does better is for the Quod half of S6.
- Nothing here settles an ADR. ADR-0013's stability comparison now has its baseline number (Dafny 4/25 with E2, 0/18 without; Verus 0/32).

## Threats to validity

- One run per cell, one model (`claude-sonnet-5-5`). Agent behavior varies between runs.
- The agent's Read tool could reach files outside the run directory; only the prompt forbade it. The run directory held nothing but the task and `./verify`, and the session JSON records no permission denials, but individual tool calls were not logged.
- Line classes come from keyword rules in `analyze.py`. They were spot-checked on t07 and t08 only; with no lemmas or proof blocks in any solution, the proof-step count of 0 does not depend on them.
- Translations to Dafny and Verus are by hand. Verus sortedness clauses use an equivalent two-variable form.
