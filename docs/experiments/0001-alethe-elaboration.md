# Experiment 0001: Do cvc5 certificates fit ADR-0003's accepted rules?

- **Date:** 2026-11-19
- **Step:** S1 of [ADR-0006](../adr/0006-thin-end-to-end-slice.md) (Proposed)
- **Tests:** [ADR-0003](../adr/0003-kernel-logic-and-proof-format.md) (Accepted)
- **Author:** Claude (agent), at the maintainer's request
- **Inputs:** [`0001-alethe-elaboration/`](0001-alethe-elaboration/): 61 obligations, `run.sh`, `classify.sh`, result tables

## Question

ADR-0003 assumes that solver certificates elaborate into a strict, search-free rule subset, and names Alethe (via cvc5) as the source format. ADR-0006's validation rule: if fewer than ~80% of obligations elaborate without manual steps, or bit-vector obligations mostly fail, open a superseding ADR before writing the kernel.

## Setup

| Item | Value |
|---|---|
| Solver | cvc5 1.4.2 (official macOS arm64 static release) |
| Alethe flags | `--dump-proofs --proof-format-mode=alethe --proof-granularity=dsl-rewrite --tlimit=30000` |
| CPC flags | same with `--proof-format-mode=cpc` |
| Corpus | 61 hand-written refutational obligations shaped like phase-1/2 code (each asserts hypotheses and the negated goal; expected `unsat`) |
| Not run | Carcara (checking and elaboration): building it was blocked in this session. No proof was independently checked. |

`--proof-granularity` matters: with the default (`macro`), even a one-line bound check contains `hole "untranslated rewrite"` steps. `dsl-rewrite` replaces them with named RARE rewrites, `poly_simp` and `evaluate`.

| Category | Obligations | Shape |
|---|---|---|
| `b` bounds | 10 | index/slice bounds, loop invariant init/preserve, termination measure (linear `Int`) |
| `o` overflow | 8 | ADR-0002 range checks for `+`, `-`, negation, midpoint, widening |
| `m` nonlinear | 6 | `*` overflow, 2D and strided indexing |
| `d` div/mod | 6 | ring-buffer index, `div`/`mod` by constants and variables |
| `q` sequences, quantifiers, `spec fn` | 13 | quantified loop invariants over `Seq`, one over SMT arrays, frame, sortedness, recursive `spec fn` unfolding |
| `x` bit-vectors | 8 | masks, shifts, `x & (x-1)`, rotate |
| `c` BV ↔ `Int` | 6 | ADR-0002 bridge: masked value as index, add without wrap, shift as multiply |
| `t` datatypes | 4 | `Result` projection, injectivity, distinctness; struct update |

## Rule classes

| Class | Meaning | Rules seen |
|---|---|---|
| **A** | Accepted as listed in ADR-0003 | `resolution`, `reordering`, `contraction`, `refl`, `la_generic`, `la_disequality`, `forall_inst`, `sko_forall`, `bind`, Tseitin clause rules, `subproof` |
| **B** | Premise forms of accepted rules; mechanical elaboration into A | `cong`, `trans`, `symm`, `and_intro` |
| **C** | Local and search-free, but not accepted | 60 distinct: named RARE rewrites (`arith-elim-lt`, `arith-leq-norm`, `bool-double-not-elim`, `bv-ule-eliminate`, …), `evaluate`, `poly_simp`, `poly_simp_rel`, `*_simplify`, `aci_simp`, `ac_simp`, `div_intro`, `distinct_elim`, `la_mult_pos`/`la_mult_sign`/`la_mult_abs_comparison` |
| **D** | Rejected by ADR-0003 | `bv_bitblast_step_*`, `bv_bitwise_slicing`, `hole "failed absorb"` |

`lia_generic` and `all_simplify` never appeared.

## Results

### Coverage

| Outcome | Alethe | CPC |
|---|---|---|
| Solved (`unsat`) | 56 | 56 |
| Certificate produced | **42** | **56** |
| Certificate with zero `hole`/`trust` steps | 37 | **56** |
| Solver failed (timeout/unknown) | 5: `c04`, `c05`, `m03` (timeout); `q01`, `q07` (unknown) | same 5 |

The Alethe printer refused 14 solved obligations:

| Refusal | Obligations |
|---|---|
| `contains operator seq.nth` / `str.update` | all 8 solved `Seq` obligations (the other 2 were not solved) |
| `contains operator DUMMY_SKOLEM` (datatypes) | all 4 `t` |
| `contains Skolem mod_by_zero` / `int_div_by_zero` (`mod`/`div` by a variable) | `d01`, `d02` |

`--proof-alethe-define-skolems` did not help.

### Worst rule class per Alethe proof

| Category | A only | ≤ B | ≤ C | D | no certificate |
|---|---|---|---|---|---|
| `b` bounds | 0 | 0 | 10 | 0 | 0 |
| `o` overflow | 0 | 0 | 8 | 0 | 0 |
| `m` nonlinear | 0 | 0 | 5 | 0 | 1 |
| `d` div/mod | 0 | 0 | 4 | 0 | 2 |
| `q` seq/quant/`spec fn` | 0 | 0 | 3 | 0 | 10 |
| `x` bit-vectors | 0 | 0 | 2 | 6 | 0 |
| `c` BV ↔ `Int` | 0 | 0 | 1 | 3 | 2 |
| `t` datatypes | 0 | 0 | 0 | 0 | 4 |
| **Total (61)** | **0** | **0** | **33** | **9** | **19** |

### Proof size

| | Proofs | Median steps | Max steps | Median size |
|---|---|---|---|---|
| Alethe | 42 | 230 | 1,872 (`x03`) | 17.5 KB |
| CPC | 56 | 127 | 3,540 (`q08`) | — |

Across all Alethe proofs, 47% of steps are class A, 32% class B, 20% class C and 0.4% class D. Class C is a minority of steps, but every proof has some.

## Findings

1. **ADR-0003 as written fails its own validation.** 0 of 61 obligations stay inside the accepted rules. This holds even after elaborating class B. The 80% threshold is not met, so ADR-0006 requires a superseding ADR before kernel work (S4).
2. **The blocker is local rewriting, not search.** Every non-BV proof needs only class A, B and C rules. Class C rules are local and deterministic: each RARE rewrite is a fixed, named equation schema; `poly_simp` is polynomial normalization (the same machinery `la_generic` already needs); `evaluate` folds ground terms. ADR-0003 rejected "broad simplification" because it could hide search. These rules do not. About 60 distinct schemas covered this corpus.
3. **Alethe cannot be the only certificate source.** The cvc5 Alethe printer has no support for sequences, datatypes, or `div`/`mod` by a variable. These are core to Quod's `Seq(S)` sort, `Result`/structs, and ring buffers. CPC (cvc5's native format, checked by Ethos) covered all 56 solved obligations with no trust steps.
4. **Bit-vector reasoning goes through bit-blasting, as ADR-0003 feared.** 9 of the 12 solved obligations involving bit-vectors use bit-blasting or `hole` steps. ADR-0003's alternative, `qd_bv_int` lemma instances, is unvalidated: cvc5 does not produce them, so they would have to come from Quod's own elaborator or from agents.
5. **CPC reasons about sequences through string theory.** Its sequence proofs use word-equation rules (`concat_unify`, `concat_eq`, `string_reduction`), not the small sequence axiom set ADR-0003 plans. To check them directly, the kernel would have to accept those rules. Otherwise an elaborator must rebuild the argument from sequence axioms, which is real proof search, though outside the kernel.
6. **Nonlinear arithmetic is better than expected.** 5 of the 6 `m` obligations were solved, using small local lemmas (`la_mult_sign`, `la_mult_pos`, `la_mult_abs_comparison`).
7. **Solver failures are few and expected.** The 5 failures are quantified existentials, a quantified invariant over a sequence update, and BV ↔ `Int` with arithmetic. In Quod, an agent would supply instantiations or lemmas there. Supplying them is the agent's job, and it is outside the scope of the kernel.

## Recommendation

Open a superseding ADR for ADR-0003 (drafted as [ADR-0008](../adr/0008-kernel-rules-from-real-certificates.md), Proposed). Keep its core: refutational, flat, search-free steps; the kernel builds hypotheses from the obligation; no `hole`, no `lia_generic`, no bit-blasting. Change three things:

| Change | Why |
|---|---|
| Add an **enumerated rewrite-schema rule** (`qd_rewrite <name>`): each schema is listed in the versioned axiom file and printed in the trust report. Also accept `poly_simp`/`poly_simp_rel` (sharing `la_generic`'s normalizer), `evaluate` on ground terms, and the `la_mult_*` sign lemmas. | Finding 2: these are what real certificates contain; each check is a match or a normalization, with no search. |
| Make the **elaborator take CPC as input** as well as Alethe (or CPC only). The kernel's own format stays Quod's flat step list. | Finding 3: Alethe has no sequences or datatypes. |
| Treat **bit-vectors and sequences as open sub-questions** with their own spikes, not as settled. | Findings 4 and 5: neither ADR-0003's `qd_bv_int` route nor its sequence axioms is backed by any solver today. |

**Trusted base:** the first change grows the kernel's rule set by about 60 equation schemas and a polynomial normalizer. It must be stated in the ADR and checked against the 5,000-line budget.

## Limitations

- Proofs were classified, not checked. Carcara, Ethos and a Quod kernel were not run.
- The corpus was written by hand by an agent, with no generator. Obligations are smaller than real VCs, which carry more hypotheses.
- Single solver version and option set. Results may change with other cvc5 versions or `--proof-granularity` modes.
- "Elaborates" is approximated by rule class. Class C was not actually elaborated into class A.

## Reproduce

```bash
docs/experiments/0001-alethe-elaboration/run.sh /path/to/cvc5 /tmp/out > summary.tsv
docs/experiments/0001-alethe-elaboration/classify.sh /tmp/out
```
