# ADR-0002: Mathematical integers in specs, checked machine integers in code

- **Status:** Accepted
- **Date:** 2026-11-19
- **Deciders:** project maintainer (accepted 2026-11-19)
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** [open-questions.md §3](../open-questions.md#3-arithmetic-semantics-in-specs)

## Context

Quod has no undefined behavior ([design-principles.md §3](../design-principles.md#3-zero-undefined-behavior)): every arithmetic operation is total or carries a proven precondition. Specs must also state facts about arithmetic, and they are the part humans read. The question is what `a + b` means in each place, and how the two meanings connect.

Constraints:
- Specs should read the way a human expects (`len + 1` is the next number, not "next number mod 2³²").
- Obligations should land in theories that agents reason about reliably and that the kernel can check with small certificates (see ADR-0003).
- "Nothing implicit" ([design-principles.md §12](../design-principles.md#12-agent-friendly-surface)) argues against hidden conversions.

## Options considered

### Option A: Mathematical integers in specs, machine integers in code (SPARK big-int mode, Dafny, Verus)
- For: specs read naturally; overflow obligations become simple range facts in linear integer arithmetic, which agents handle reliably and the kernel can certify with Farkas-style coefficients; widely used and proven in practice.
- Against: two meanings of `+`, distinguished by context; an implicit widening in spec contexts.

### Option B: Bit-vectors everywhere
- For: one semantics; exactly matches the hardware.
- Against: specs become hard for humans to read and easy to get subtly wrong (`len + 1 > len` is false at the maximum); nonlinear bit-vector reasoning is hard for both agents and solvers; certificates are large (bit-blasting).

### Option C: Explicit lifting everywhere (`int(a) + int(b)` in specs)
- For: fully explicit; one rule per operator.
- Against: specs, the human-facing layer, become noisy; forgetting `int(…)` silently produces a weaker or wrong spec, which is worse than the implicit rule it replaces.

## Decision

Option A, with the following rules.

### Spec contexts

A **spec context** is any expression inside `requires`, `ensures`, `invariant`, `decreases`, `where`, `assert`, `assume`, a `spec fn` body, a `lemma`, or ghost code. The context is always determined by the enclosing keyword, never by inference.

- Spec-only types: `int` (unbounded integers) and `nat` (`int` where value ≥ 0).
- In a spec context, `+ - * / %` and unary `-` on machine integers **lift their operands to `int`** and produce an `int`. They have no overflow.
- Comparisons (`== != < <= > >=`) between machine integers and `int` compare mathematical values.
- Unsuffixed integer literals in a spec context have type `int`.
- `/` and `%` **truncate toward zero**, matching code-side semantics (not Euclidean division), so a spec and its implementation never disagree on negative operands. Division by zero in a spec is an obligation (spec well-formedness), not a defined value.
- Bitwise operators (`& | ^ << >>`) keep their **machine-type, bit-vector meaning** in spec contexts and require both operands of the same machine type. Converting their results to `int` is explicit: `int(x & m)`.
- `x as int` is accepted (redundant but allowed); `e as u32` for an `int` expression in a spec carries a range well-formedness obligation.
- The human report renders spec arithmetic as mathematical (for example "len increases by exactly 1"), so the implicit lifting is made visible where humans read.

### Code contexts

- `a + b`, `a - b`, `a * b` on type `T` carry the precondition `MIN_T <= int(a) op int(b) <= MAX_T`. The result is the exact value.
- `a / b`, `a % b`: precondition `b != 0`; for signed types, also `!(a == MIN_T && b == -1)`. Truncate toward zero.
- `a << n`: precondition `n < bits(T)` and no bits lost (`int(a) * 2^n` fits in `T`). `a >> n`: precondition `n < bits(T)`; arithmetic shift for signed types. `& | ^ !`: total.
- `e as U`: precondition that the value fits in `U`. Widening casts are total and need no proof.
- Wrapping, saturating and checked behavior use named functions, never operators: `add_wrap`, `add_sat`, `add_checked -> Option<T>`, and likewise for `sub`, `mul`, `shl`; `trunc::<U>(e)` for truncation.
- No implicit conversions between machine types in code; mixed-type operands are a type error.

### Phase 1 transition

Phase 1 has no proofs. Until phase 2, each code-side precondition that is not statically trivial is **compiled as a runtime trap** and listed in a build report of "unverified checks". From phase 2 on, the same preconditions become proof obligations and unproven ones are compile errors. This keeps phase-1 C output free of undefined behavior without a verifier.

## Consequences

- Most overflow, bounds and loop obligations land in **linear integer arithmetic**, the easiest theory to certify (ADR-0003). Bit-level code relies on a fixed set of kernel lemmas linking bit-vector operations to `int` (for example `int(x & (2^k - 1)) == int(x) % 2^k` for unsigned `x`).
- Grammar: `int` and `nat` become spec-only type names; literal typing depends on context (spec vs. code). `spec/grammar-draft.md` and the examples need updating.
- The roadmap's phase-1 exit criterion "every potentially-UB operation is rejected unless guarded by an explicit runtime check" is replaced by "compiled as a runtime trap and reported" (above). The trap is not a `panic` effect: it disappears in phase 2.
- **Trusted base:** adds the bit-vector ↔ `int` lemma set to the kernel's axioms (ADR-0003 lists them).
- Agent loop: agents state invariants in plain math (`lo + hi <= 2 * n`) without overflow side conditions inside specs.
- Docs to update on acceptance: `open-questions.md` §3, `design-principles.md` §3, `spec/grammar-draft.md` (`int`, `nat`, literal typing), `roadmap.md` phase 1, `examples/*.qd`.

## Validation

- Phase-2 corpus: count obligations by theory; target a large majority in linear integer arithmetic.
- Seeded-bug test: agents writing specs for overflow-prone functions (binary search midpoint, ring buffer index) produce correct specs without overflow confusion.
- Revisit if: spec/code arithmetic mismatches cause real bugs in the corpus, or human reviewers misread spec arithmetic in report studies (phase 8).
