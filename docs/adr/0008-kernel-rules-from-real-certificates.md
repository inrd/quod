# ADR-0008: Kernel rules fitted to real solver certificates

- **Status:** Proposed
- **Date:** 2026-11-20
- **Deciders:** —
- **Author:** Claude (agent), at the maintainer's request
- **Supersedes:** [ADR-0003](0003-kernel-logic-and-proof-format.md) (on acceptance, ADR-0003's status becomes `Superseded by ADR-0008`)
- **Closes:** none new; keeps [open-questions.md §2](../open-questions.md#2-kernel-logic-and-proof-term-format) resolved; no sub-questions left open: bit-vectors are settled in §4 by experiment 0002, sequences in §5 by experiment 0003
- **Depends on:** ADR-0001, ADR-0002, [experiment 0001](../experiments/0001-alethe-elaboration.md)

## Context

ADR-0003 chose a many-sorted first-order kernel checking flat, refutational, search-free proof steps, with Alethe from cvc5 as the source of certificates and a strict accepted-rule list. [Experiment 0001](../experiments/0001-alethe-elaboration.md) (61 obligations, cvc5 1.4.2) tested that choice:

| Finding | Consequence for ADR-0003 |
|---|---|
| 0 of 61 obligations stay inside the accepted rules, even after trivial elaboration | fails its own validation criterion |
| Every non-bit-vector proof needs only accepted rules, premise forms (`cong`, `trans`, `symm`, `and_intro`) and **local rewrites**: ~60 named RARE equation schemas, `poly_simp`, `evaluate`, `la_mult_*` | the gap is local rewriting, not search |
| cvc5's Alethe printer refuses sequences, datatypes and `div`/`mod` by a variable (14 of 56 solved obligations); cvc5's native CPC format proves all 56 with no trust steps | Alethe cannot be the only certificate source |
| 9 of 12 solved bit-vector obligations use bit-blasting or `hole` | the `qd_bv_int` route is unvalidated (validated later by [experiment 0002](../experiments/0002-bit-vectors.md), §4) |
| CPC proves sequence facts with string-theory word-equation rules; the only array-encoded obligation (`q10`) printed fine in Alethe | the planned "sequence axioms" match no solver's output |

What held up: no `lia_generic`, no `all_simplify`, no search inside any step, and nonlinear multiplication handled by small local lemmas. The design is sound; the rule list and certificate source were wrong.

## Options considered

### Option A: Keep ADR-0003; put everything in the elaborator
- For: no kernel change.
- Against: the elaborator would have to re-prove every rewrite, normalization and evaluation step from accepted rules, multiplying proof size (already median 230 steps) for no soundness gain; it would still have no Alethe certificates for sequences and datatypes.

### Option B: Accept the local rules directly, trusted as axioms
- For: certificates check almost as emitted.
- Against: ~60 rewrite schemas become trusted axioms, much of which is provable; RARE's rule set is cvc5's to change, not Quod's.

### Option C: Accept local rules, but justify rewrite schemas as kernel-checked lemmas; widen the certificate sources; split off bit-vectors and sequences
- For: proofs stay close to solver output; rewrite schemas that are provable from the core rules add no trust; the ones that are not are enumerated and visible; unsupported theories are handled by encoding choices and later experiments rather than guesses.
- Against: more kernel rules than ADR-0003 (normalizer, evaluator, schema instantiation); two certificate formats for the elaborator to read.

## Decision

Option C. Everything in ADR-0003 that is not changed below stays: logic and sorts, refutational proofs whose hypotheses and negated goal the kernel builds from the obligation, Alethe S-expression syntax as the **kernel's** proof format, `spec fn` definitional extension with termination check, defunctionalization, acyclic lemma citations, one versioned axiom file printed in every trust report.

### 1. Accepted rules

| Group | Rules | Change from ADR-0003 |
|---|---|---|
| Propositional | `resolution`, `th_resolution`, Tseitin clause rules, `contraction`, `reordering`, `true`, `false`, `not_not`, `subproof` | `subproof`, `true`, `false` made explicit |
| Equality | `refl`, `eq_reflexive`, `eq_symmetric`, `eq_transitive`, `eq_congruent`, `eq_congruent_pred`, **`cong`, `trans`, `symm`, `and_intro`** | premise forms added (32% of steps in experiment 0001; each is a local check) |
| Linear arithmetic | `la_generic` with explicit Farkas coefficients, `la_disequality`, `la_totality`, `la_tautology`, `qd_int_cut` | unchanged |
| Nonlinear lemmas | **`la_mult_pos`, `la_mult_neg`, `la_mult_sign`, `la_mult_abs_comparison`** | new; each checks one sign/monotonicity instance |
| Normalization | **`qd_poly_norm`** (Alethe `poly_simp`, `poly_simp_rel`): both sides normalize to the same polynomial over `Int` with rational coefficients, using `la_generic`'s normalizer | new; the normalizer is already needed by `la_generic` |
| Evaluation | **`qd_evaluate`** (Alethe `evaluate`): a ground term over `Bool`/`Int` literals, including the §4 bitwise function symbols applied to literals, equals its value | new; needs arbitrary-precision integers, which `la_generic` needs anyway |
| Rewrite schemas | **`qd_rewrite <name>`**: the step is an instance of a named, enumerated equation schema (see §2) | new |
| Quantifiers | `forall_inst` with explicit terms, `sko_forall`, `sko_ex`, `bind` | unchanged |
| Datatypes | constructor injectivity, distinctness, projection, exhaustiveness | unchanged (still Quod rules) |
| Quod-specific | `qd_def_unfold`, `qd_induction`, `qd_lemma`, `qd_bv_int` | unchanged; `qd_bv_int` instantiates the §4 schemas |

`qd_poly_norm` must not blow up: the kernel rejects a step whose normalized polynomial exceeds a fixed size bound (set in `kernel/`), so checking time stays bounded by proof size.

Still rejected: `hole`, `trust`, `lia_generic`, `all_simplify`, broad `*_simplify` rules as kernel rules, and bit-blasting (`bv_bitblast_*`, `bv_bitwise_slicing`).

### 2. Rewrite schemas are lemmas first, axioms only when they must be

- A schema is a fixed-arity equation `∀x̄. l = r` with a name, stored in `kernel/rewrites` (versioned).
- A schema whose universal closure can be **proven from the accepted rules** (most arithmetic and Boolean ones, e.g. `arith-elim-lt`, `bool-double-not-elim`, `ite-true-cond`) ships with that proof. The kernel checks it once when loading the file; using it adds **no trust**.
- A schema that cannot be proven that way (e.g. those relying on `div`/`mod` totality, ADR-0002's conventions) goes into the **axiom file**, is printed in the trust report, and needs a one-line justification.
- Schemas with list variables (RARE's n-ary rules, e.g. over `and` with any number of arguments) are **not** schemas. The elaborator rewrites those steps into fixed-arity instances or propositional reasoning.
- Initial list: the rewrites that experiment 0001 needed, excluding bit-vector ones (§4 replaces them). A provable schema is a lemma, not a new rule, so it can be added by PR with its proof; this is how CLAUDE.md's "rule or axiom changes need an ADR" applies to schemas. An unprovable schema is an axiom and needs an ADR, as before.

### 3. Certificate sources

- The **untrusted elaborator** accepts cvc5 certificates in **Alethe and CPC**. CPC is the default for obligations involving datatypes or `div`/`mod` by a variable, which Alethe does not print.
- The elaborator rewrites `*_simplify`, `aci_simp`, `ac_simp`, `distinct_elim` and similar steps into accepted rules. These are propositional or definitional equivalences over the step's own atoms, so small, local proofs exist; producing them is elaborator work, outside the trusted base.
- The kernel's format does not change: it never reads CPC.

### 4. Bit-vectors: `Int` plus a `qd_bv_int` schema library

Evidence: [experiment 0002](../experiments/0002-bit-vectors.md). With this encoding, all 14 `x`/`c` obligations were solved and gave certificates with no rejected rule and no bit-vector rule. Bit-blasting still failed 2 of them and left `hole` steps in 3.

- **No `BV(n)` sort in the kernel.** This changes ADR-0003's sort list. A machine integer is an `Int`, and the obligation generator ([ADR-0007](0007-trusted-obligation-generator.md)) supplies its range fact (`0 <= x < 2^w` unsigned). ADR-0002's `int(e)` is the identity in obligations.
- Bitwise operators and wrapping functions are **uninterpreted function symbols**, one per operator and width (`and32`, `or64`, `shl16`, `add_wrap32`, …). They get their meaning only from `qd_bv_int` schemas.
- **`qd_bv_int` schemas** are axioms in the axiom file (§2), each with a justification, and each appears in the trust report. They are parameterized by width `w` and, where needed, a constant `k`. Initial list, unsigned: `and-range`/`and-le`, `or-range`/`or-ge`, `xor-self`, `add` and `sub` (`mod 2^w`), `shl k` (`(x·2^k) mod 2^w`), `lshr k` (`x div 2^k`), `and-mask k` (`x mod 2^k`), `or-disjoint k`. A new schema needs its bit-vector check (at least 32 and 64 bits) recorded with it and, being an axiom, an ADR.
- `qd_evaluate` evaluates these symbols on literals.
- The generator sends solvers the schemas for the symbols an obligation uses, as **quantified axioms**. Hand-picked instances proved fragile in experiment 0002 (`x08`: 211k steps vs 2.6k).
- Bit-blasting stays rejected. Revisit if real code needs many new schemas, or an operator with no clean `Int` statement (e.g. `popcount`).
- Signed operators (signed comparison, arithmetic `>>`) and `mul_wrap` are not yet covered. Their schemas follow the same pattern and need the same checks before use.

### 5. Sequences: encode as arrays

- Sequence contents use the kernel sort `Array(Int, S)`. Its only axioms are read-over-write at the same index and at a different index. There is no extensionality axiom.
- `Seq(S)` keeps ADR-0003's meaning (length + indexing), but it is an **encoding in the obligation generator** ([ADR-0007](0007-trusted-obligation-generator.md)), not a kernel sort. A sequence is an array term plus an `Int` length term (`0 <= len`); update is `store`, and the length term stays the same; a sub-range view is the base array plus an offset, and views of views add offsets. Sequence equality is equal lengths plus pointwise equality on `[0, len)`, never array `=`, because array `=` would also constrain out-of-range indices.
- Solvers receive the same encoding, so no string theory is involved. Phases 1–2 only need length, indexing, update and sub-range slicing.
- Generator guidance (not a kernel rule): quantifiers over a view range over base indices, so E-matching triggers contain no arithmetic.
- Evidence: [experiment 0003](../experiments/0003-seq-as-arrays.md). All 17 obligations got Alethe certificates with no rejected rule (2 needed `--enum-inst`), and only the two read-over-write axioms were used.

## Consequences

- **Trusted base grows:** kernel code for `qd_poly_norm`'s size bound, `qd_evaluate`, `qd_rewrite` instantiation, the four `la_mult_*` checks, and the premise-form equality rules; axiom-file entries for unprovable rewrite schemas, `div`/`mod` definitions (the latter were needed under ADR-0003 too) and the two array read-over-write axioms (replacing ADR-0003's sequence axioms), and the 9 `qd_bv_int` schema families of §4 (replacing ADR-0003's `BV(n)` sort; no bit-blaster). The trusted obligation generator (ADR-0007) gains the sequence encoding of §5. Provable rewrite schemas add no trust. The kernel's 5,000-line budget (ADR-0001) still applies; if this does not fit, that is a reason to revisit before raising the budget.
- **Proof size** stays close to solver output (median ~230 Alethe steps in experiment 0001) instead of growing with re-proven rewrites.
- **Agent loop:** kernel errors still name one step. A failing `qd_rewrite` names the schema, which an agent can look up in `kernel/rewrites`.
- **Elaborator** is larger (two input formats) but stays untrusted.
- **Slice (ADR-0006):** S4 can start on linear arithmetic, Booleans, quantifiers and arrays-as-sequences (experiment 0003 passed) and unsigned bit-vector operations through `qd_bv_int` (experiment 0002 passed).
- Docs to update on acceptance: ADR-0003 status, `docs/adr/README.md`, `kernel/README.md` (rules, rewrites file, size bound), `architecture.md` (component table: elaborator inputs), `oracle-protocol.md` (`proof/submit` still takes Alethe text; certificates may come from either format), `open-questions.md` §2 and §12.

## Validation

- Re-run experiment 0001's corpus through the elaborator and kernel once built: **≥ 80% of solved non-bit-vector obligations** check without manual steps.
- Every schema in `kernel/rewrites` is either kernel-proven at load or listed in the axiom file; CI fails otherwise.
- Kernel within budget with all rules above.
- Experiments 0002 (bit-vectors) and 0003 (sequences as arrays) completed, each followed by an ADR or an update to this one before acceptance. Both done: §4 and §5 updated.
- Revisit if cvc5 changes RARE or CPC so that the schema list churns between releases, or if `qd_poly_norm`'s size bound rejects common obligations.
