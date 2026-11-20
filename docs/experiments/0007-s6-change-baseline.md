# Experiment 0007: S6 change baseline — agents updating verified code in Dafny and Verus

- **Date:** 2026-11-20
- **Step:** S6 of [ADR-0006](../adr/0006-thin-end-to-end-slice.md) (Accepted); follows experiments [0005](0005-s6-baseline.md) and [0006](0006-s6-hard-baseline.md)
- **Tests:** nothing by itself. It measures proof maintenance, the claim the vision makes most strongly against existing verifiers and that 0005–0006 did not test
- **Author:** Claude (agent), at the maintainer's request
- **Inputs:** [`0007-s6-change-baseline/`](0007-s6-change-baseline/): `build_tasks.py` (builds the tasks from 0006's solutions), task files, harness and analysis copied from 0006 (only the experiment number differs), `prompt.txt` (new), per-run records

## Question

When the specification of verified code changes, how much work does an agent need to make it verify again in Dafny and Verus, compared with writing it from scratch (experiment 0006)? Does the old proof help, get in the way, or get thrown away?

## Protocol (fixed before the first run)

As in [experiment 0005](0005-s6-baseline.md#protocol-fixed-before-the-first-run) (agent, model `claude-sonnet-5-5`, tools, limits, isolation, verifier versions, success rule, metrics, error classes, stability edits), with these differences:

- **Start files.** Each task is the agent's verified 0006 solution in the same verifier, with the description replaced by a `// CHANGE:` note and the `SPEC` block replaced by a changed specification. The old body, invariants, lemmas and proofs stay. `build_tasks.py` builds them. The Dafny and Verus start files therefore differ (each is its own 0006 solution).
- **Prompt.** [`prompt.txt`](0007-s6-change-baseline/prompt.txt): the first paragraph says the file verified before the spec changed and asks the agent to update it; the rules are 0006's.
- **Checks before the runs.** Every new spec was checked alone, with a stub body, in both verifiers (no errors). Every start file fails: in Dafny 4 by verification, 4 by resolution (renamed spec function, changed return shape); in Verus 3 by verification, 5 by typing.
- **Extra metrics.** The share of the start file's lines after `SPEC-END` that survive in the final file (`difflib` matching), and the cost of each change relative to the 0006 run that wrote the start file from scratch.

| Task | From | Change | Kind |
|---|---|---|---|
| c01 `sort_desc` | h01 | ascending → descending | flips every order fact in invariants and lemmas |
| c02 `four_flag` | h02 | 3 values, 2 bounds → 4 values, 3 bounds | new region, new return shape |
| c03 `count_lt` | h03 | count `== key` → count `< key`; spec function renamed | lemmas refer to the old spec function |
| c04 `sum_sq` | h04 | sum → sum of squares; new bound `xs[k] < 2^16` | spec function and overflow bound change |
| c05 `two_diff` | h05 | `xs[i] + xs[j]` → `xs[j] - xs[i]` | different two-pointer movement and exclusion argument |
| c06 `pool_find` | h07 | list length → position of first node holding `key`; new `val` array | early exit, new parameter |
| c07 `pool_rev_len` | h08 | also return the list length | small change; the old proof should mostly survive |
| c08 `merge_into` | h10 | allocate the result → write into a caller-provided array | aliasing precondition, `modifies` frame |

### Predictions (written before the first run)

| Prediction | Value |
|---|---|
| success | at least 14 of 16 |
| hardest | c02 and c05 in both verifiers; c01 in Verus (multiset lemmas around order) |
| easiest | c07 in both (1–2 calls) |
| `./verify` calls | median 2–3 per run, similar to 0006's from-scratch runs; total cost at most $3 |
| old proof | mostly kept for c03, c06, c07, c08; mostly rewritten for c01, c02, c05 |
| Dafny instability | at least one of the 8 final Dafny solutions flaky on 5 re-runs; no Verus solution |

If changing is clearly cheaper than writing from scratch in both verifiers, the maintenance claim against them weakens. If it costs about the same or more, the claim stands for these verifiers.

## Results

Per-run tables: [`results/`](0007-s6-change-baseline/results/) (`runs.tsv`, `errors.tsv`, `stability.tsv`, `survival.md`, `repeat.tsv`).

| | Dafny 4.11.0 | Verus 0.2026.11.02 |
|---|---|---|
| solved | **8/8** | **8/8** |
| verified on the first `./verify` call | 5/8 | 4/8 |
| `./verify` calls: total, max | 14, 5 (c07) | 14, 3 (c05, c07) |
| same functions written from scratch in 0006: calls | 18 | 25 |
| cost, all 8 runs (same functions in 0006) | $0.33 ($0.35) | $0.34 ($0.46) |
| start-file lines kept in the final file, median (range) | 88% (42–100%) | 93% (44–100%) |
| errors by class | 5 verification | 1 typing, 5 verification |
| proof steps as a share of non-code added lines | 43% | 42% |
| stability edits that broke verification | **5/24**, all E2 (invariant order) | **0/24** |
| re-verification, 5 times each (not pre-registered) | 7/8 always pass; **c08 2/5** | 8/8 always pass |

### Predictions vs. outcome

| Prediction | Outcome |
|---|---|
| at least 14 of 16 succeed | **held**: 16/16 |
| hardest: c02, c05; c01 in Verus | **mostly wrong**: c05 cost the most calls in Verus (3, tied with c07); c02 and c01 took 1–2 calls. The hardest task was c07 in both verifiers |
| easiest: c07 (1–2 calls) | **wrong**: c07 was the hardest (Dafny 5 calls, Verus 3), see finding 2 |
| median 2–3 calls, similar to 0006; total cost ≤ $3 | **partly**: medians were 1 (Dafny) and 1.5 (Verus), below 0006's for the same functions (1 and 3); total $0.67 |
| old proof mostly kept for c03, c06, c07, c08; mostly rewritten for c01, c02, c05 | **half held**: kept for c03, c06, c07, c08 (79–100%); rewritten only for c05 (42–44%); c01 and c02 kept 70–92% |
| at least one flaky Dafny solution, no flaky Verus one | **held**: Dafny c08 passed 2 of 5 re-runs of the unchanged file (it passed the protocol's single re-check, so it counts as solved); no Verus solution failed a re-run |

### Findings

1. **Updating verified code after a spec change was easy for the agent.** All 16 changes verified within 5 calls and $0.09 per run. In both verifiers the change took fewer calls than writing the same function from scratch did in 0006 (14 vs. 18 and 14 vs. 25), and the agent kept most of the old body and proof (median 88% in Dafny, 93% in Verus). The pre-registered rule says the maintenance claim against these verifiers weakens only if changing is *clearly* cheaper in both: it is clearly cheaper in Verus (25% lower cost, 44% fewer calls) and about the same in Dafny. Either way, nothing here shows maintenance pain at function scale.
2. **Proof cost does not follow the size of the spec change.** c07 adds one returned value, `len == |order|`. Counting into a `usize` needs a proof that the counter cannot overflow, i.e. that a list of distinct node indices is no longer than the pool (a pigeonhole argument). Dafny's agent wrote 3 set-cardinality lemmas (about 55 lines), Verus's 13 proof lines. This is a property of checked machine integers ([ADR-0002](../adr/0002-arithmetic-semantics.md)), not of either verifier, so Quod would meet the same obligation; it bears on how the oracle should report "which fact is missing" rather than on the choice of language.
3. **Dafny verdicts vary on an unchanged file again.** The c08 solution (merge into a caller array, multiset postcondition) passed 2 of 5 re-runs. Over 0005–0007, 3 of 30 Dafny solutions are flaky (t06, h06, c08), 0 of 30 Verus ones.
4. **Invariant order again breaks Dafny (5/24) and never Verus (0/24)**, for the same well-formedness reason as in 0005 and 0006.
5. **Old proofs hardly got in the way.** One call in 16 runs went on a stale name (Verus c04: an invariant still used the old `sum`); every other failing call was a verification error about the new behavior.

## Conclusion

Across three experiments (0005–0007: 30 specifications, 60 runs, about $2.20 of agent time) the baseline agent solved 59 runs. It writes, and now also updates, verified single functions in Dafny and Verus quickly and cheaply. The remaining measured gaps are the ones 0005 and 0006 found: verdicts that vary on an unchanged file (Dafny: 3 of 30 solutions), sensitivity to invariant order (Dafny), and front-end errors from a host language (Verus). Proof maintenance is not one of them at this scale.

Untested: changes that cross function boundaries (a callee contract change that ripples into callers), modules where the agent writes the internal contracts (variant 1), and code that people rather than agents changed. Nothing here settles an ADR.

## Threats to validity

- One run per cell, one model; start files are the agent's own 0006 solutions, so it updates proofs written in its own style.
- Line survival uses `difflib` over stripped, non-comment lines; a moved block counts as kept only where the order is preserved.
- The agent's Read tool could reach files outside the run directory; only the prompt forbade it.
