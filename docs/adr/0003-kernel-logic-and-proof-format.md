# ADR-0003: First-order kernel logic with flat, Alethe-based proof steps

- **Status:** Accepted
- **Date:** 2026-11-19
- **Deciders:** project maintainer (accepted 2026-11-19)
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** [open-questions.md §2](../open-questions.md#2-kernel-logic-and-proof-term-format); constrains §12 (role of the SMT solver)
- **Depends on:** ADR-0001 (Rust kernel), ADR-0002 (arithmetic semantics)

## Context

The kernel is Quod's only trusted component ([design-principles.md §2](../design-principles.md#2-small-trusted-kernel-de-bruijn-criterion)). It must check explicit proofs deterministically, with no search and no timeouts. Agents write the proofs, usually with help from SMT solvers, so the format decides how fast the agent loop runs.

Facts that shape the choice:
- Quod's proof obligations are mostly **first-order**: range facts, equalities over struct fields and array elements, quantified invariants, unfolding of `spec fn` definitions. ADR-0002 puts most of them in linear integer arithmetic.
- SMT solvers already emit proof certificates. **Alethe** (produced by cvc5 and veriT) is the best documented, and **Carcara** is an existing Rust checker and elaborator for it.
- Agents repair proofs best when an error points to one local step ("step 14: expected X, got Y") rather than deep inside a nested term.

## Options considered

### Option A: Dependent type theory (Lean/Rocq-style proof terms)
- For: mature, proven small kernels; very expressive; higher-order specs come for free.
- Against: heavyweight for first-order obligations; proof terms are large and nested, so errors are hard for agents to locate and fix; turning SMT output into terms needs a full reconstruction layer; definitional unfolding makes checking time hard to predict.

### Option B: Many-sorted first-order logic with flat, explicit proof steps (Alethe subset plus Quod rules)
- For: matches the shape of obligations; SMT certificates convert almost directly; flat step lists are easy for agents to generate, read and patch; each rule check is a local, linear-time operation; Carcara gives a reference design and a possible untrusted elaborator.
- Against: theory axioms (integers, bit-vectors, sequences) must be trusted and enumerated; higher-order specs need defunctionalization; Alethe has rules too coarse for a strict kernel, so a subset must be defined.

### Option C: LCF-style (abstract theorem type, small rule set; HOL Light)
- For: smallest kernels known; strong track record.
- Against: proofs are programs that construct theorems, not data. Storing them as artifacts needs a logged replay format, which is Option B with extra steps. Higher-order logic is more than Quod needs.

## Decision

Option B.

### Logic

Many-sorted first-order logic with equality and these sorts:

| Sort | Purpose |
|---|---|
| `Bool` | formulas |
| `Int` | `int`/`nat` in specs (ADR-0002); `nat` is `Int` plus a range fact |
| `BV(n)` for n ∈ {8, 16, 32, 64} | machine integers under bitwise operations |
| `Seq(S)` | arrays and slices (length + indexing; no memory model) |
| user datatypes | structs (products), `Option`/`Result`/enums (sums) |
| uninterpreted sorts and functions | abstract types, `trust` declarations |

Memory and permissions (phase 5) are encoded on top of these sorts by the obligation generator; they add no kernel sorts unless a later ADR says so.

### Definitions and lemmas

- A `spec fn` becomes a **definitional extension**: a fresh function symbol plus its defining equation. The kernel admits the equation only after its termination obligation (the measure decreases) has been checked. That keeps the logic consistent.
- Higher-order arguments are **defunctionalized**: `fold(xs, 0, add)` becomes a first-order symbol `fold_add` with its own definition. Spec lambdas are restricted to named spec functions (see open-questions §4).
- A proven `lemma` can be cited by name in other proofs. The kernel checks that lemma dependencies form an acyclic graph.

### Proof format

- A proof is **refutational**: from the obligation's hypotheses and the negation of its goal, derive the empty clause. **The kernel builds the hypotheses and the negated goal from the obligation itself**; the proof file only supplies steps. A proof cannot smuggle in assumptions.
- Concrete syntax is **Alethe** (SMT-LIB S-expressions): `(step t14 (cl ...) :rule ... :premises (...) :args (...))`, with `anchor`/subproofs for binders. A proof is a flat, numbered list of steps.
- Proofs live in files next to the source, keyed by obligation ID and fingerprint ([oracle-protocol.md](../oracle-protocol.md)). File layout is decided in phase 2.

### Accepted rules

Only rules whose check is local, deterministic and does no search:

| Group | Rules (Alethe names where they exist) |
|---|---|
| Propositional | `resolution`, `th_resolution`, the Tseitin-style clause rules, `contraction`, `reordering` |
| Equality | `refl`, `eq_reflexive`, `eq_symmetric`, `eq_transitive`, `eq_congruent`, `eq_congruent_pred` |
| Linear arithmetic | `la_generic` **with explicit Farkas coefficients**, `la_disequality`, `la_totality`, `la_tautology` |
| Integer reasoning | Quod rule `qd_int_cut` (Chvátal–Gomory rounding with explicit coefficients) |
| Quantifiers | `forall_inst` with explicit terms, `sko_forall`, `sko_ex`, `bind` |
| Datatypes, sequences | Quod rules for constructor injectivity/distinctness and the sequence axioms |
| Quod-specific | `qd_def_unfold` (spec fn definition), `qd_induction` (over `nat` and sequence length, explicit predicate), `qd_lemma` (cite a proven lemma), `qd_bv_int` (instances of the fixed bit-vector ↔ `int` lemmas from ADR-0002) |

Rejected, even if a solver emits them:
- `hole`, `lia_generic`, and any rule that amounts to "trust this step";
- rules whose check needs a decision procedure or rewriting search (for example `all_simplify` and other broad simplification rules);
- bit-blasting rules, at least initially (large certificates; bit-level facts go through `qd_bv_int` lemmas instead).

Coarse solver output is turned into accepted rules by an **untrusted elaborator** outside the kernel (Carcara-style). The elaborator can be buggy without affecting soundness: the kernel re-checks whatever it produces.

### Trusted axioms

The kernel contains a fixed, enumerated list of theory axioms (integer arithmetic via the rules above, sequence axioms, datatype axioms, bit-vector ↔ `int` lemmas). The list lives in one file in `kernel/`, is versioned, and is printed in every trust report.

## Consequences

- **Agent loop:** solver certificate → elaborator → kernel works from phase 2; kernel errors name one step; agents can also write short step lists by hand for small obligations.
- **SMT role (open-questions §12):** solvers stay on the agent side only, consistent with this ADR. §12 can be closed in favor of its first option, or kept open for the "partly trusted fragment" option.
- **Trusted base:** kernel rule implementations + the axiom list + the obligation generator (the latter is still open; see [architecture.md](../architecture.md#trust-boundaries)).
- **Proof size:** large (Farkas coefficients, explicit instantiations). Acceptable: storage is cheap and agents produce it. Compression is a later concern.
- **Expressiveness ceiling:** specs requiring real higher-order reasoning (for example quantifying over functions) are out of reach. The planned Lean re-checker from ADR-0001 is the escape hatch if that ceiling becomes a problem.
- **Kernel validation:** the soundness test suite (phase-2 exit criterion) includes, for each accepted rule, malformed applications that must be rejected.
- Docs to update on acceptance: `open-questions.md` §2 (and §12 if closed), `architecture.md`, `oracle-protocol.md` (`proof/submit` payload is Alethe text), `kernel/README.md`, `examples/05_holes_and_proofs.qd`.

## Validation

- On the phase-2 corpus: fraction of obligations whose cvc5 proofs elaborate into the accepted subset without manual steps.
- Kernel checking time is linear in proof size on the corpus (no outliers from search).
- Kernel stays within the ADR-0001 line budget with all accepted rules implemented.
- Revisit if: a significant share of corpus obligations needs rejected rules (especially bit-vector reasoning), or defunctionalization makes common specs unreadable in the human report.
