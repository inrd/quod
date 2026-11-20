# ADR-0010: A small trusted C emitter over Core IR, and no compiler optimizer

- **Status:** Proposed
- **Date:** 2026-11-20
- **Deciders:** —
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** trust-boundary item 2 in [architecture.md](../architecture.md#trust-boundaries) ("compilation preserves semantics"). Narrows [open-questions.md §9](../open-questions.md#9-backend-strategy) (C emission, CompCert-compatible).
- **Depends on:** [ADR-0007](0007-trusted-obligation-generator.md) (Core IR text, trusted `vcgen`), [ADR-0009](0009-annotations-not-proofs.md) (`check` verdict)

## Context

The soundness target is: *if the kernel accepts all obligations, every execution of the emitted program satisfies the spec, modulo the trust report and the C compiler.* ADR-0007 makes the path Core IR → `vcgen` → kernel trusted and small. The path from Core IR to running code is still untrusted:

- the **C emitter** turns Core IR into C;
- the **optimizer** "uses proven facts to drop runtime checks, then transforms code".

A bug in either makes the running program differ from the verified one, possibly with undefined behavior in the emitted C, which C compilers exploit. Then the kernel's verdict says nothing about the binary. Listing an untrusted emitter in the trust report is honest, but it puts "modulo the compiler" back into the claim, which Quod set out to remove.

The optimizer is also redundant with the refinement principle ([design-principles §6](../design-principles.md#6-refinement-workflow)): agents can write the fast version and prove it equal to the simple one. Speed then comes from source the kernel has checked, not from compiler passes nobody verifies.

## Options considered

### Option A: Untrusted emitter and optimizer, listed in the trust report (current plan)
- For: no constraints on either component.
- Against: the verified program and the running program are linked only by untrusted code; an optimizer that removes checks based on "proven facts" can remove them wrongly.

### Option B: Small trusted emitter (`cgen`) over the same Core IR text as `vcgen`; no Quod optimizer
- For: one artifact (Core IR text) feeds both trusted consumers, so the program checked and the program emitted are the same text; `cgen` is a direct, syntax-directed translation, small enough to audit; no optimizer to trust.
- Against: a second trusted crate next to the kernel and `vcgen`; Quod-level optimizations are left to agents (refinement) and to the C compiler.

### Option C: Untrusted emitter plus translation validation per build
- For: smallest trusted code in principle.
- Against: needs a formal C semantics and a validator, which is larger than `cgen` itself.

### Option D: Verified emitter (CompCert-style, e.g. in Lean)
- For: strongest guarantee.
- Against: needs a mechanized Core IR semantics first (ADR-0007 Option D). Long-term only.

## Decision

Option B now; Option D stays the long-term path, together with ADR-0007's.

1. **`cgen/`** is a workspace crate under the kernel's constraints (std-only plus the Core IR types shared with `vcgen`, `#![forbid(unsafe_code)]`, no `HashMap`/`HashSet`, floats, clocks, threads, randomness) with a budget of **≤ 2,000 lines excluding tests**. Its input is Core IR text ([ADR-0007](0007-trusted-obligation-generator.md)); its output is C.
2. **No Quod optimizer.** The "optimizer" component is removed from the architecture. Performance comes from (a) agents writing faster code verified against a reference (refinement) and (b) the C compiler's optimizer on UB-free C.
3. **Check elision is not a decision `cgen` makes.** From phase 2 on, an unproven precondition is a compile error, so a program that passes `check` has every precondition proven and `cgen` emits no traps. During phase 1 (no proofs), `cgen` emits a trap for every precondition that is not statically trivial (ADR-0002). There is no mixed mode in which `cgen` chooses which checks to drop.
4. **Emitted C is a documented subset**, specified in `spec/c-subset.md` before `cgen` code is written: C99, `<stdint.h>` fixed-width types, explicit casts on every conversion, unsigned arithmetic only where Core IR's preconditions make the result exact, no implementation-defined behavior that the subset does not pin down (it lists every such assumption, e.g. two's-complement conversions, and checks it with `_Static_assert` in the emitted file).
5. **CompCert-compatible.** The subset is accepted by CompCert as well as gcc and clang. The trust report names the C compiler and its version; users who want the smallest trusted base compile with CompCert.
6. **The phase-1 emitter is `cgen`.** No separate AST → C emitter is written. The front end lowers to Core IR, and Core IR is emitted to C.

## Consequences

- **Trusted base:** grows by `cgen` (≤ 2,000 lines) and `spec/c-subset.md`; both are listed in every trust report. In exchange, the untrusted emitter and optimizer leave the soundness argument: the claim becomes *"modulo the trust report, `vcgen`, `cgen` and the named C compiler"*.
- **Order of work:** Core IR (`spec/core-ir.md`, ADR-0007) must exist before the first emitter, so ADR-0006's S3 moves after a Core IR step. This avoids writing an AST-based emitter that would be replaced.
- **Performance:** no Quod-level loop transformations or layout changes unless written and proven by agents. The phase-6 exit criterion (measured speedups from refinements) is the test of whether that is enough.
- **Agent loop:** none directly. Agents optimizing code write a second function and prove it equal; [ADR-0011](0011-v1-scope.md) defines `refines` as sugar for that.
- Docs to update on acceptance: `architecture.md` (pipeline: remove optimizer; add `cgen`; trust boundaries item 2), `design-principles.md` §2 (the path to emitted code), §6 (refinement is how code gets faster), `open-questions.md` §9, `roadmap.md` phase 1 (emitter over Core IR; CompCert in the exit criterion), `CLAUDE.md` (crate list and constraints), new `spec/c-subset.md`, new `cgen/README.md`.

## Validation

- `cgen` stays within budget through phase 2.
- On the corpus, emitted C compiles warning-free with gcc, clang and CompCert, and the three binaries give identical outputs on the test inputs; UBSan reports nothing.
- Differential tests: a Core IR interpreter (untrusted, test-only) and the compiled C agree on the corpus.
- Revisit if refinement-only optimization leaves the phase-6 benchmarks far behind hand-written C, or if `cgen` cannot stay within budget.
