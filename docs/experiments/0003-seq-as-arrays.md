# Experiment 0003: Do sequences encoded as SMT arrays plus a length give checkable certificates?

- **Date:** 2026-11-20
- **Step:** gate for S4 of [ADR-0006](../adr/0006-thin-end-to-end-slice.md) (Proposed)
- **Tests:** [ADR-0008](../adr/0008-kernel-rules-from-real-certificates.md) §5 (Proposed)
- **Author:** Claude (agent), at the maintainer's request
- **Inputs:** [`0003-seq-as-arrays/`](0003-seq-as-arrays/): 17 obligations, `run.sh` and `classify.sh` (copied unchanged from [experiment 0001](0001-alethe-elaboration.md)), result tables

## Question

Experiment 0001 found that cvc5's Alethe printer refuses every `Seq` obligation, and that CPC proves sequence facts with string-theory word-equation rules. ADR-0008 §5 proposes that the obligation generator sends sequences to solvers as **SMT arrays plus an explicit length**. Does this encoding:

1. get the solver to solve at least as many obligations as `Seq` did?
2. produce Alethe certificates with no `hole`, `trust` or other class-D rule?
3. need only array rules that are local, and few enough to list as kernel axioms or rewrite schemas?

## Encoding

| Quod (phase 1–2) | SMT encoding |
|---|---|
| value `a: Seq(S)` | two terms: `a : (Array Int S)` and `a_len : Int`, with `0 <= a_len` |
| `a.len()` | `a_len` |
| `a[k]` | `(select a k)` |
| `a[i := v]` | `(store a i v)`; the length term stays `a_len` |
| view `&a[lo..hi]` | no new array: base `a`, offset `lo`, length `hi - lo`; `s[k]` is `(select a (+ lo k))` |
| view of a view | offsets add: `&s[lo2..hi2]` is base `a`, offset `lo + lo2` |
| `s == t` (sequence equality) | `s_len = t_len` and `∀k. 0 <= k < s_len → s[k] = t[k]`; **never** array `=` |

Indices outside `[0, len)` are unconstrained. Array `=` would also constrain them, so it is too strong as a hypothesis and too weak a match for Quod's meaning of equality.

## Setup

| Item | Value |
|---|---|
| Solver | cvc5 1.4.2 (official macOS arm64 static release), same as experiment 0001 |
| Flags | as experiment 0001 (`--proof-granularity=dsl-rewrite`, `--tlimit=30000`), Alethe and CPC; each also with `--enum-inst` |
| Not run | Carcara, Ethos, a Quod kernel. Proofs were classified, not checked. |

| Obligations | Count | Source |
|---|---|---|
| `q01`–`q09`, `q11` | 10 | experiment 0001's `Seq` obligations, re-encoded one to one |
| `q14` swap, `q15` bounded equality | 2 | new: swap with frame; copy-loop exit proves `b == a` |
| `s01`–`s05` slice views | 5 | new: view bounds, view of a view, write through a view (frame), loop invariant through a view (offset index `s04`, base index `s05`) |

Dropped: `q10` (already the array form of `q01`; now `q01`) and `q12`/`q13` (`spec fn` unfolding, no sequences; already printed in Alethe in experiment 0001).

## Results

### Coverage

| | `Seq` (exp. 0001, `q` with sequences) | arrays, default flags | arrays, `--enum-inst` |
|---|---|---|---|
| Obligations | 10 | 17 | 17 |
| Solved | 8 (`q01`, `q07` unknown) | 15 (`q07`, `s04` unknown) | **17** |
| Alethe certificate | **0** | 15 | **17** |
| Alethe certificate with no class-D rule | 0 | 15 | **17** |
| CPC certificate with no `trust`/`hole` | 8 | 15 | **17** |

With `--enum-inst`, cvc5 also solves experiment 0001's `Seq` `q01` and `q07`. So the encoding does not by itself fix solving; it fixes **certificates**.

The two default-flag failures are quantifier instantiation, not the encoding:

| Obligation | Cause | Fixed by |
|---|---|---|
| `q07` | existential goal; the witness `i` must be found | `--enum-inst` (or an agent-supplied instantiation) |
| `s04` | invariant quantifies over view index `k`, read as `(select a (+ lo k))`; E-matching cannot match through `+` | `--enum-inst`, or the generator quantifying over base indices (`s05`, solved with default flags) |

### Worst rule class per Alethe proof (`--enum-inst`)

Classes as in experiment 0001 (relative to ADR-0003).

| Category | A only | ≤ B | ≤ C | D | no certificate |
|---|---|---|---|---|---|
| `q` re-encoded (10) | 0 | 1 (`q04`) | 9 | 0 | 0 |
| `q` new (2) | 0 | 0 | 2 | 0 | 0 |
| `s` views (5) | 0 | 0 | 5 | 0 | 0 |
| **Total (17)** | **0** | **1** | **16** | **0** | **0** |

Across all steps: 48% class A, 29% B, 24% C, 0% D (experiment 0001: 47 / 32 / 20 / 0.4).

### Array rules used

| Rule (Alethe) | Uses | Statement | Under ADR-0008 |
|---|---|---|---|
| `rare_rewrite array-read-over-write-split` | 9 | `select(store(a,i,v),k) = ite(k = i, v, select(a,k))` | provable from the two axioms below → lemma |
| `rare_rewrite array-read-over-write` | 1 | `select(store(a,i,v),i) = v` | axiom (same index) |
| `arrays_row` | 5 | `k ≠ i ⊢ select(store(a,i,v),k) = select(a,k)` | axiom (different index) |
| `arrays_idx` | 1 | `select(store(a,i,v),i) = v` (theory lemma form) | same axiom as above |

No extensionality (`arrays_ext`) appeared, because sequence equality is bounded and pointwise. CPC used only `arrays_read_over_write*` and the same RARE rules, with **no string or sequence rules**.

Other class-C rules not seen in experiment 0001: `bool-or-taut2` (provable Boolean schema), `qnt_rm_unused` (drop an unused bound variable) and `connective_def` (`∃ = ¬∀¬`). The last two are definitional, and ADR-0008 §3 leaves such steps to the elaborator.

### Proof size

| Obligation | `Seq` CPC steps (exp. 0001) | array CPC steps | array Alethe steps |
|---|---|---|---|
| `q08` copy preserve | 3,540 | 243 | 723 |
| `q11` frame | 3,485 | 8 | 6 |
| `q05` running max | 478 | 458 | 1,035 |
| `q02`, `q03`, `q04`, `q06`, `q09` | 9–135 | 8–159 | 4–321 |
| **Median (all 17)** | — | 119 | 160 |

## Findings

1. **The encoding passes ADR-0008 §5's test.** Every obligation has an Alethe certificate with no class-D rule (17/17 with `--enum-inst`, 15/15 solved without it). Under experiment 0001's `Seq` encoding, no certificates were produced.
2. **Two array axioms are enough.** Read-over-write at the same index and at a different index cover every array step. The `split` form is a provable lemma. Extensionality is never needed if the generator emits sequence equality as bounded pointwise equality.
3. **Length and slice views never reach the solver as theory.** The explicit length is a separate `Int` term, and views are offset arithmetic. `q04` (update preserves length) becomes the trivial `b_len = a_len`, and slice facts become linear arithmetic. ADR-0008 §5's planned kernel axioms "length preserved by update" and "length/index of a sub-range view" were never used. This bookkeeping moves into the obligation generator, which ADR-0007 already makes trusted.
4. **Frame and copy proofs shrink a lot.** `q11` goes from 3,485 CPC steps to 8, and `q08` from 3,540 to 243, because string-theory word equations are replaced by one read-over-write lemma.
5. **Quantifier instantiation is the remaining weak point, not the encoding.** Existential goals (`q07`) and invariants over view indices (`s04`) need `--enum-inst` or a hint. Quantifying over base indices (`s05`) avoids the second case with default flags. This is a generator choice, not a kernel rule.
6. **Equality is a soundness trap for the generator.** Encoding Quod's `==` on sequences as array `=` would assume values at out-of-range indices. As a hypothesis, that is unsound relative to Quod's meaning. The generator must emit bounded pointwise equality.

## Recommendation

Update ADR-0008 §5 (still Proposed) as follows. These changes have been applied to the Proposed ADR, and the maintainer decides whether to accept them.

| Change | Why |
|---|---|
| The kernel sort for sequence contents is `Array(Int, S)`. Its axioms are **only** read-over-write at the same and at a different index. | Findings 1–2 |
| `Seq(S)` becomes an **encoding in the obligation generator**: an array term plus a length term, views as base plus offset, equality as bounded pointwise. Drop the length and sub-range kernel axioms. | Finding 3; this keeps the kernel smaller and puts the rules in the generator, which ADR-0007 already trusts and reviews |
| Generator rule: sequence equality is never array `=`. | Finding 6 |
| Generator guidance (not a kernel rule): quantify over base indices for views; run solvers with `--enum-inst` as a fallback. | Finding 5 |

**Trusted base:** smaller than ADR-0008 §5 as drafted: two array axioms instead of four `Seq` axioms. The trusted generator gains the encoding table above (a few lines per construct). That must be listed under ADR-0007's generator rules.

## Limitations

- Proofs were classified, not checked. Carcara, Ethos and a Quod kernel were not run.
- Elements are `Int`. Machine-integer elements add a range fact per element (`∀k < len. 0 <= a[k] < 2^32`), which was not tested.
- No nested sequences, sequences in struct fields, or sequences passed across calls. Each would need its own length term from the generator.
- The corpus was written by hand by an agent and is small (17). A single solver version was used.
- `--enum-inst` was tried only as a whole-corpus flag; its cost on larger obligations is unknown.

## Reproduce

```bash
docs/experiments/0003-seq-as-arrays/run.sh /path/to/cvc5 /tmp/a --enum-inst > summary.tsv
docs/experiments/0003-seq-as-arrays/classify.sh /tmp/a
docs/experiments/0003-seq-as-arrays/run.sh /path/to/cvc5 /tmp/c --proof-format-mode=cpc --enum-inst
```
