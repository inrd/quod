# kernel/ — trusted proof checker

**Empty skeleton.** Implementation starts in phase 2. The only trusted component of Quod.

## What it does

Checks proofs of proof obligations, nothing else ([ADR-0003](../docs/adr/0003-kernel-logic-and-proof-format.md)):

- **Logic:** many-sorted first-order logic with equality; sorts `Bool`, `Int`, `BV(8|16|32|64)`, `Seq(S)`, user datatypes, uninterpreted sorts/functions.
- **Proofs are refutational:** from the obligation's hypotheses and negated goal, derive the empty clause. The kernel builds hypotheses and negated goal from the obligation itself; the proof file supplies only steps.
- **Format:** flat, numbered Alethe steps (SMT-LIB S-expressions), restricted to the accepted rule subset plus Quod rules (`qd_int_cut`, `qd_def_unfold`, `qd_induction`, `qd_lemma`, `qd_bv_int`, datatype and sequence rules).
- **Rejected rules:** `hole`, `lia_generic`, anything that needs search or a decision procedure (e.g. `all_simplify`), bit-blasting (for now). Coarse solver output is elaborated outside the kernel.
- **`spec fn` definitions** are admitted only after their termination obligation is checked; lemma citations must form an acyclic graph.
- **Axioms:** one versioned file listing every theory axiom; printed in every trust report.

## Constraints ([ADR-0001](../docs/adr/0001-implementation-language.md))

- Rust crate with **no dependencies** beyond `core`/`alloc`/`std` (not even other workspace crates).
- `#![forbid(unsafe_code)]`.
- No `HashMap`/`HashSet`, floats, threads, clocks, randomness, environment or file-system access during checking.
- **Line budget: ≤ 5,000 lines excluding tests**, enforced in CI.
- Checking time linear in proof size; no search, no timeouts.
- Identical verdicts on every OS and architecture.

## Later

An independent second checker in Lean 4 for the same proof format, once the format is stable.
