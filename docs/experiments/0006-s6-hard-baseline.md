# Experiment 0006: S6 hard baseline — agents on harder tasks in Dafny and Verus

- **Date:** 2026-11-20
- **Step:** S6 of [ADR-0006](../adr/0006-thin-end-to-end-slice.md) (Accepted); follows [experiment 0005](0005-s6-baseline.md)
- **Tests:** nothing by itself. It asks whether a task set exists on which the baseline struggles, which S6 needs to compare Quod against anything
- **Author:** Claude (agent), at the maintainer's request
- **Inputs:** [`0006-s6-hard-baseline/`](0006-s6-hard-baseline/): task files, harness and analysis copied unchanged from experiment 0005 (only the experiment number in comments differs), per-run records

## Question

Experiment 0005's tasks were solved by the baseline on the first verifier call. Do tasks that need permutations, recursive spec functions, ghost witnesses and overflow bounds separate agents from "verified at once"? If the baseline still solves them easily, Quod cannot win S6 on the agent loop and should argue from determinism, stability and trust instead (see 0005's conclusion).

## Protocol (fixed before the first run)

Identical to [experiment 0005's](0005-s6-baseline.md#protocol-fixed-before-the-first-run), including protocol revision 1 of its prompt: same agent, model (`claude-sonnet-5-5`), tools, limits, isolation, verifier versions, success rule, metrics, error classes and stability edits. Differences:

- Tasks are written directly in Dafny and Verus. They need constructs outside Quod's slice subset (last column), so there is no `.qd` version yet; writing one is part of widening the slice.
- Spec helpers (recursive `function`/`spec fn`, node types) sit inside the `SPEC-BEGIN`/`SPEC-END` block, so the success rule protects them too.
- Hand review (success rule 4) applies to every task whose spec allows a degenerate body; none of these specs does (permutation, ghost-witness or exact-value postconditions), so review is a spot check only.

| Task | What it needs | Outside the slice |
|---|---|---|
| h01 `insertion_sort` | nested loops, sortedness, multiset preserved by swaps | multisets |
| h02 `dutch_flag` | three-region invariant, multiset, tuple result | multisets, tuples |
| h03 `count_eq` | head-recursive spec function vs. a forward loop (needs a prefix lemma), counter overflow bound | `spec fn`, recursion |
| h04 `sum_u64` | recursive sum, `u64` accumulator overflow bound (`len < 2^32`) | `spec fn`, casts |
| h05 `two_sum_sorted` | two-pointer correctness: excluded pairs argument | tuples |
| h06 `dedup_sorted` | in-place compaction, set of values preserved | sequence `contains` |
| h07 `pool_list_len` | traverse index links with a ghost node list | ghost parameters |
| h08 `pool_list_reverse` | in-place reversal of index links, frame for nodes outside the list | ghost parameters |
| h09 `ptr_list_reverse` | pointer counterpart of h08 (Dafny: heap objects with a ghost node list; Verus: owned `Option<Box<Node>>` with a recursive view) | heap, recursion |
| h10 `merge` | merge into a new array, sorted and multiset union | multisets, allocation |

## Results

Per-run tables: [`results/`](0006-s6-hard-baseline/results/). Before the stability numbers were taken, two bugs in `analyze.py`'s edit script were fixed (in this copy and in 0005's): E1 renamed the keyword in `let ghost x` instead of `x`, and E2 could swap a Verus invariant clause with the `decreases` line. Neither could have changed 0005's numbers (no 0005 solution has ghost code, and the bad swap always fails, which 0005's Verus results never did).

| | Dafny 4.11.0 | Verus 0.2026.11.02 |
|---|---|---|
| solved | **10/10** | **10/10** |
| verified on the first `./verify` call | 7/10 | 3/10 |
| `./verify` calls: total, max | 21, 9 (h07) | 30, 6 (h06, h10) |
| errors by class | 10 verification | 1 syntax, 5 typing, 14 verification |
| added lines in solved runs: code / annotation / proof step | 93 / 102 / 31 | 131 / 117 / 137 |
| proof steps as a share of non-code lines | 23% | 54% |
| cost, all 10 runs (max per run) | $0.41 ($0.15) | $0.63 ($0.15) |
| wall time per run | 12–70 s | 12–83 s |
| stability edits that broke verification | **8/30**, all E2 (8 of 10 applicable) | **0/29** |
| re-verification, 5 times each (not pre-registered) | 9/10 always pass; **h06 4/5** | 10/10 always pass |

Hand review: every solution was checked for forbidden constructs and spec integrity by `analyze.py`; h09 in both verifiers and h08 in Dafny were read in full and are genuine in-place reversals.

### Findings

1. **Harder tasks still do not stop the baseline.** All 20 cells were solved, at most 9 verifier calls and $0.15 per run. The tasks took more work than 0005's (first-call success fell from 11/12 to 7/10 in Dafny and from 8/12 to 3/10 in Verus; proof steps went from none to a quarter or half of the non-code lines), but the agent got there every time, within about a minute.
2. **Proof steps are needed for these tasks.** Lemmas and `proof { … }` blocks appear in 3 of 10 Dafny and all 10 Verus solutions (multiset, recursive-sum and sequence facts). ADR-0013 expects Quod agents to write annotations, not steps (under 10% steps). The baseline suggests that tasks of this kind need lemma-level reasoning whatever the language; whether Quod's pipeline can turn those into annotations plus solver work is for the Quod half to show.
3. **Dafny verdicts vary on unchanged files again:** h06 passed 4 of 5 re-runs. Together with 0005's t06, 2 of 22 Dafny solutions are flaky; no Verus solution was.
4. **Invariant order breaks Dafny in most solutions here (8/10)**, for the same reason as in 0005 (each invariant's well-formedness is checked using only the ones before it). Verus: 0/29 edits of any kind.
5. **Verus front-end errors persist** (6 of 20 failing calls: syntax and typing), Dafny had none.
6. **Pool vs. pointer:** pool traversal (h07) was the hardest Dafny task (9 calls); pool reversal (h08) and pointer reversal (h09) were solved on the first call in Dafny and in 1–2 calls in Verus. No sign that index-linked structures are harder for agents than pointer-linked ones (ADR-0011 decision 1), on one run each.

## Conclusion

- **At function scale, today's agent with an existing verifier solves what we can specify, quickly and cheaply.** Two task sets (22 specs, 44 runs) produced 43 solved runs and a total cost of about $1.50. The vision's key metric (time from hole to verified, phase 9) has little room left for a new language to win on, at least for single functions.
- **What the baseline does not give**, measured here: deterministic verdicts (Dafny: 2 flaky solutions of 22), stability under harmless edits (Dafny: invariant order), and a front end without host-language traps (Verus: all 6 failing calls in 0005, 6 of 20 here). These are the language-independent parts of Quod's design (kernel, stored certificates, own front end), plus spec review, which this experiment does not test.
- Neither experiment settles an ADR. They are evidence for re-reading the vision's speculative fallback (retarget the kernel, oracle and reports to a verified subset of an existing language) before investing in a wider slice, and for testing at larger scale (multi-function modules, where solver instability and proof maintenance grow) before concluding either way.

## Threats to validity

- One run per cell, one model. Agent behavior varies; a 10/10 could be 8/10 on another day.
- Tasks are single functions of 10–40 lines with specs written for them. Real code adds callee contracts, larger obligations and spec maintenance, none of which is measured.
- The agent's Read tool could reach files outside the run directory; only the prompt forbade it, and tool calls were not logged.
- Line classes come from keyword rules in `analyze.py`; lemma and `proof` blocks were detected correctly in the solutions read by hand (h01, h06, h09).
