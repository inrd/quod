# Experiment 0002: Bit-vectors, `qd_bv_int` lemma library or bounded bit-blasting?

- **Date:** 2026-11-20
- **Step:** gate for S4 of [ADR-0006](../adr/0006-thin-end-to-end-slice.md) (Proposed), bit-vector part
- **Tests:** [ADR-0008](../adr/0008-kernel-rules-from-real-certificates.md) §4 (Proposed); the lemma route of [ADR-0002](../adr/0002-arithmetic-semantics.md) (Accepted)
- **Author:** Claude (agent), at the maintainer's request
- **Inputs:** [`0002-bit-vectors/`](0002-bit-vectors/): route (a) obligations and lemma library, route (b) obligations at four widths, lemma checks, `gen.sh`, `run.sh`, `classify.sh`, result tables

## Question

In [experiment 0001](0001-alethe-elaboration.md), 9 of the 12 solved bit-vector obligations (`x`, `c`) used bit-blasting or `hole` steps, and 2 (`c04`, `c05`) timed out. ADR-0008 §4 names two routes:

| Route | Kernel gets |
|---|---|
| **(a) Lemma library** | a fixed list of `qd_bv_int` schemas linking each bitwise operator to `Int`; no bit-level reasoning |
| **(b) Bounded bit-blasting** | bit-blasting rules for widths ≤ 64, with a kernel-side size limit |

For each route, the experiment asks three things:
1. Does it solve the 14 `x`/`c` obligations?
2. Does it give certificates with no class-D rule (`hole`, `trust`, `lia_generic`; for route (a), also no bit-blasting)?
3. What does it add to the kernel?

## Setup

| Item | Value |
|---|---|
| Solver | cvc5 1.4.2 (official macOS arm64 static release), as in experiments 0001 and 0003 |
| Flags | as experiment 0001 (`--proof-granularity=dsl-rewrite`, `--tlimit=30000`); Alethe, and CPC for route (b) |
| Not run | Carcara, Ethos, a Quod kernel. Proofs were classified, not checked. |

### Route (a) encoding

| Quod | SMT encoding |
|---|---|
| value `x: u32` | `x : Int` with the generator's type fact `0 <= x < 2^32` |
| `x & y`, `x \| y`, `x ^ y` | `(and32 x y)`, `(or32 x y)`, `(xor32 x y)`: uninterpreted functions, one per operator and width |
| `x << k`, `x >> k` (unsigned) | `(shl32 x k)`, `(lshr32 x k)` |
| `add_wrap`, `sub_wrap` (code `+`/`-` as bit-vectors in `x`) | `(add32 x y)`, `(sub32 x y)` |
| unsigned `<`, `<=`, `==` | `<`, `<=`, `=` on `Int` |
| `int(e)` (ADR-0002) | `e` itself; there is no separate bit-vector sort |

The bitwise operators get their meaning only from the lemma library (all unsigned, width `w`, `W = 2^w`, constant `k`):

| Schema | Statement | How it was checked |
|---|---|---|
| `and-range`, `and-le` | `0 <= and(x,y) < W`, `and(x,y) <= x`, `and(x,y) <= y` | pure bit-vector form, 32 and 64 bits |
| `or-range`, `or-ge` | `or(x,y) < W`, `or(x,y) >= x`, `or(x,y) >= y` | pure bit-vector form, 32 and 64 bits |
| `xor-self` | `xor(x,x) = 0` | pure bit-vector form, 32 and 64 bits |
| `add`, `sub` | `add(x,y) = (x+y) mod W`, `sub(x,y) = (x−y) mod W` | SMT-LIB's definition of `bvadd`, `bvsub` |
| `shl k` | `shl(x,k) = (x·2^k) mod W` | SMT-LIB's definition of `bvshl` |
| `lshr k` | `lshr(x,k) = x div 2^k` | pure bit-vector form via `bvudiv`, 32 and 64 bits |
| `and-mask k` | `and(x, 2^k−1) = x mod 2^k` | pure bit-vector form via `bvurem`, 32 and 64 bits |
| `or-disjoint k` | `x mod 2^k = 0 ∧ y < 2^k → or(x,y) = x + y` | pure bit-vector form (`bvor = bvadd`, no carry), 32 and 64 bits |

With default flags, cvc5 timed out on 8 of 9 of these statements written with `ubv_to_int` at 32 bits. `--solve-bv-as-int=sum` solved some of them, but still timed out on `and-le` and `or-ge`. So the [lemma checks](0002-bit-vectors/lemma-checks/) use equivalent pure bit-vector forms, which cvc5 solves by bit-blasting (all 12 checks unsat).

Two variants of route (a):

| Variant | Obligations | Who picks the instances |
|---|---|---|
| **a1 instances** | [`lemma-instances/`](0002-bit-vectors/lemma-instances/): each obligation asserts the schema instances it needs (lines ending `; inst`) | the agent or elaborator |
| **a2 library** | [`lemma-library/`](0002-bit-vectors/lemma-library/): the instances are replaced by [`library.smt2`](0002-bit-vectors/library.smt2) as quantified axioms with patterns | the solver (E-matching) |

### Route (b) encoding

[`bitblast/w{8,16,32,64}/`](0002-bit-vectors/bitblast/) contains the 8 `x` obligations in native bit-vectors at each width, with masks and shifts scaled to the width (`gen.sh`).

## Results

### Route (a): lemma library

| Obligation | Instances (a1) | a1 Alethe steps | a2 Alethe steps | Worst class (a1, a2) | Exp. 0001 (`BitVec`) |
|---|---|---|---|---|---|
| `c01` mask as index | 1 | 14 | 168 | C, C | D |
| `c02` hash bucket | 2 | 19 | 180 | C, C | D |
| `c03` top byte | 1 | 238 | 98 | C, C | D |
| `c04` add without wrap | 1 | 719 | 713 | C, C | timeout |
| `c05` shift as multiply | 1 | 603 | 571 | C, C | timeout |
| `c06` range | 0 | 21 | 21 | C, C | C |
| `x01` and mask | 1 | 14 | 168 | C, C | D |
| `x02` shift range | 1 | 238 | 98 | C, C | D |
| `x03` `x & (x-1) < x` | 2 | 466 | 855 | C, C | D |
| `x04` `x << 1 == x + x` | 2 | 88 | 104 | C, C | C |
| `x05` `x \| y >= x` | 1 | 14 | 163 | C, C | D |
| `x06` `x ^ x == 0` (64-bit) | 1 | 1 | 12 | A, A | C |
| `x07` mask index | 1 | 15 | 164 | C, C | D |
| `x08` rotate byte | 6 | **211,217** (19 MB) | 2,565 (231 KB) | C, C | D |
| **Solved / no class D** | | **14 / 14** | **14 / 14** | | 12 / 3 |

- All class-C rules in route (a) already appeared in experiment 0001: arithmetic and Boolean RARE rewrites, `poly_simp`, `evaluate`, `div_intro`, `arith-int-div-total`/`arith-int-mod-total`, `mod-elim`, and the `*_simplify` family. **No bit-vector rule appears.**
- In a2, E-matching found every instance it needed, using 40 `forall_inst` steps in total. No hints were needed.
- `c` and `x` obligations look the same after encoding, because ADR-0002's `int(e)` disappears.

`x08` is the outlier. Its arithmetic, `((256·x mod W) + x div 2^24) div 2^24 = (x div 2^16) mod 256`, sends cvc5's linear integer arithmetic into case splitting on `div`/`mod`. Variants of the hint did not fix it:

| `x08` variant | Alethe steps |
|---|---|
| a1, `shl` instance as `(x·256) mod W` | 211,217 |
| a1, `shl` instance as `256·(x mod 2^24)` | 308,771 |
| a1 plus an agent-supplied byte split of `x` | 250,044 |
| a2 library | 2,565 |

### Route (b): bit-blasting

Alethe steps per obligation; `bb` is the number of bit-blasting steps, `hole` the number of `hole` steps.

| Obligation | w = 8 | w = 16 | w = 32 | w = 64 | bb | hole |
|---|---|---|---|---|---|---|
| `x01` and mask | 86 | 134 | 230 | 422 | 7 | 1 |
| `x02` shift range | 86 | 134 | 230 | 422 | 6 | 0 |
| `x03` `x & (x-1) < x` | 450 | 882 | 1,746 | 3,474 (297 KB) | 7 | 0 |
| `x04` `x << 1 == x + x` | 40 | 40 | 40 | 40 | 0 | 0 |
| `x05` `x \| y >= x` | 141 | 269 | 525 | 1,037 | 4 | 0 |
| `x06` `x ^ x == 0` | 17 | 17 | 17 | 17 | 0 | 0 |
| `x07` mask index | 86 | 118 | 182 | 310 | 7 | 1 |
| `x08` rotate | 191 | 191 | 191 | 191 | 1 | 1 |

- **Growth is linear in width.** Bit-blasting steps are few (≤ 7) and do not grow, but each step's term has one entry per bit, and the steps that follow grow with it.
- **Alethe has `hole "failed absorb"` steps** (a constant-extraction rewrite cvc5 cannot justify) in 3 of 8 obligations at every width. Under ADR-0008 these are rejected regardless of route.
- **CPC has no `trust` steps** at 8 and 64 bits (64-bit sizes match Alethe: `x03` 3,020 steps, 261 KB). It uses `bv_bitblast_step` and `bv_poly_norm`.
- **Bit-blasting cannot do the `Int` bridge.** In native bit-vectors, `c04` (add without wrap) times out even at 8 bits; `c05` solves at 8 and 16 bits but times out at 32. Route (b) does not cover ADR-0002's `int(e)` obligations.

## Findings

1. **Route (a) covers the corpus, route (b) does not.** With the lemma library, all 14 obligations are solved with no rejected rule, including the 2 that bit-vectors could not solve. Route (b) still fails `c04`/`c05`, and its Alethe output has `hole` steps.
2. **The library is small.** It has 9 schema families, parameterized by width and a constant `k`. Six were checked in pure bit-vector form at 32 and 64 bits. The other three (`add`, `sub`, `shl`) restate SMT-LIB's own definitions.
3. **The kernel needs no bit-vector sort.** Machine integers are `Int` with range facts from the generator, and bitwise operators are uninterpreted functions defined only by the library. Proofs use only rules that ADR-0008 already accepts (and its `div`/`mod` axioms).
4. **The schemas are axioms, not lemmas.** The functions are opaque to the kernel, so ADR-0008 §2 puts each schema in the axiom file with its justification, and each appears in the trust report. The bit-vector checks above are evidence, not proofs, and stay outside the trusted base.
5. **Quantified library axioms are the better default.** Solver instantiation (a2) needed no hints. It was 10× larger on trivial obligations (14 → 168 steps) but 80× smaller on the hard one (`x08`). Hand-picked instances (a1) are fragile, and no hint tried brought `x08` below 250k steps.
6. **`div`/`mod` by large constants is the new weak point.** `x08` shows that linear integer arithmetic over `div`/`mod` can explode depending on the form of a fact. ADR-0008's `qd_poly_norm` size bound would reject such proofs. This needs watching on real corpus code, not a kernel change now.
7. **Bit-blasting would add trusted code without new coverage.** The kernel would have to re-blast each step's term for every operator (`and`, `or`, `add`, shifts, comparisons). Proofs stay small up to 64 bits (≤ 300 KB here), but nothing in this corpus needs it.

## Recommendation

Update ADR-0008 §4 (still Proposed) to route (a). These changes have been applied to the Proposed ADR, and the maintainer decides whether to accept them.

| Change | Why |
|---|---|
| Machine integers are `Int` with generator-supplied range facts; bitwise operators and wrapping functions are uninterpreted function symbols per operator and width (`and32`, `shl64`, …); no `BV(n)` sort in the kernel | Findings 1, 3 |
| `qd_bv_int` is the enumerated schema list above, parameterized by width and constant `k`, in the axiom file with justifications; the kernel also evaluates these symbols on literals (`qd_evaluate`) | Findings 2, 4 |
| The obligation generator sends solvers the schemas for the symbols an obligation uses, as quantified axioms | Finding 5 |
| Bit-blasting stays rejected. Revisit if real code needs many new schemas or an operator with no clean `Int` statement (e.g. `popcount`). | Finding 7 |

**Trusted base:** 9 axiom schema families (each a few lines plus a justification) and evaluation of bitwise operators on literals. This replaces the `BV(n)` sort that ADR-0003 planned. No bit-blaster.

## Limitations

- Proofs were classified, not checked. Carcara, Ethos and a Quod kernel were not run.
- Unsigned operators only. Signed comparison, arithmetic shift `>>` on signed types, `!`, `mul_wrap` and rotations as named functions were not tested.
- Library schemas were instantiated for the `k` values the corpus uses; a kernel schema would take `k` as a parameter. Widths 8 and 16 were not tested for route (a).
- The corpus has 14 hand-written obligations, and a single solver version was used.
- The lemma checks show that each schema holds at 32 and 64 bits (and by SMT-LIB definitions for the three definitional ones). They are not proofs for all widths.

## Reproduce

```bash
docs/experiments/0002-bit-vectors/gen.sh
docs/experiments/0002-bit-vectors/run.sh /path/to/cvc5 docs/experiments/0002-bit-vectors/lemma-library /tmp/a2
docs/experiments/0002-bit-vectors/classify.sh /tmp/a2
docs/experiments/0002-bit-vectors/run.sh /path/to/cvc5 docs/experiments/0002-bit-vectors/bitblast/w64 /tmp/b64 --proof-format-mode=cpc
```
