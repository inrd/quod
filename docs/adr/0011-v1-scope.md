# ADR-0011: v1 scope — value-level memory, no general heap, `refines` as a contract, resource contracts deferred

- **Status:** Proposed
- **Date:** 2026-11-20
- **Deciders:** —
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** [open-questions.md §13](../open-questions.md#13-memory-model-encoding-in-obligations) (value-level encoding) for v1. Defers [§6](../open-questions.md#6-concurrency-model) (concurrency) past v1 without closing it.
- **Depends on:** ADR-0002, [ADR-0007](0007-trusted-obligation-generator.md) (sequence encoding), [ADR-0010](0010-trusted-c-emitter-from-core-ir.md) (no optimizer)
- **Evidence gate:** S3 of [ADR-0006](0006-thin-end-to-end-slice.md) (frame-size reports, for decision 3) and S6 (pool tasks, for decision 1) (both pending)
- **Revised:** 2026-11-20 (`refines` restricted to `spec fn` references; static storage marked as a v1 target outside the slice; frame-size source per compiler; evidence gate)

## Context

The roadmap has nine phases. Several of them are open research problems on their own: a separation-logic memory model (phase 5), equivalence proofs as a language mechanism (phase 6), cost models (phase 7). Taken together they make "a C alternative" a very large target, and they pull design decisions in the early phases toward features that may never ship.

Three observations narrow the target without giving up the thesis:

| Observation | Source |
|---|---|
| Quod's own heritage forbids dynamic allocation after initialization (Power of 10, rule 3) and multi-level indirection (rule 9). Safety-critical C that follows these rules has little need for a general heap. | [power-of-10-mapping.md](../power-of-10-mapping.md) |
| Phase-1 references are already second-class: no stored references, no reference fields, no mutable aliasing at calls. Under these rules a value-level encoding of memory is exact. | [spec/typing.md §8](../../spec/typing.md), open-questions §13 |
| Sequences encoded as arrays plus lengths already produce clean certificates. Index-linked structures (pools of nodes addressed by index) use only that encoding. | [experiment 0003](../experiments/0003-seq-as-arrays.md) |

SPARK shows that a language in this shape (values, arrays, no general pointers) covers a large share of verified embedded and algorithmic code in industrial use.

`refines` ([design-principles §6](../design-principles.md#6-refinement-workflow)) looks like a separate mechanism, but `impl fn f refines g` says no more than `ensures result == g(args)` (and, for a `&mut` parameter, that its final value is the one `g` computes). A dedicated equivalence mechanism would duplicate contracts.

That reading needs care. Verification is compositional: a call to an `impl fn` is known only through its contract. If `g` were an `impl fn`, `g(args)` in `f`'s spec would mean "some value satisfying `g`'s `ensures`", which is weaker than "what `g` computes" unless that contract determines the result. Only a `spec fn` has a definition the kernel can unfold (`qd_def_unfold`).

## Options considered

### Option A: Keep the full roadmap scope for v1
- For: no change; keeps the general "C alternative" claim.
- Against: phase 5 alone is a multi-year research project (Viper, Prusti, Creusot); early phases must anticipate a heap model that does not exist yet.

### Option B: Narrow v1 to a SPARK-like core
- For: every v1 feature uses theories already validated by experiments 0001–0003; the obligation shape is fixed now and stays fixed; matches the safety-critical niche the project's principles come from.
- Against: no general pointer-linked data structures in v1; the vision's claim must be narrowed; a later heap model is a retrofit (but onto a value-level core that Creusot-style prophecies extend).

### Option C: Narrow scope but keep a heap encoding in obligations "for later"
- For: avoids a later Core IR change.
- Against: every obligation carries heap terms that v1 never needs; this is open-questions §13's Option B with no user.

## Decision

Option B. **Quod v1** is defined as follows.

1. **Memory: value-level encoding, no general heap.**
   - Obligations contain no heap sort. `&T` is a value; `&mut T` is copy-in/copy-out; slices are views as in ADR-0007.
   - Storage is local, static (module-level, written only by code holding an `init` capability; a v1 target, not part of the slice, since it needs modules, which ADR-0004 excludes in phase 1, and phase-4 capabilities) or in **pools**: fixed-capacity arrays created at initialization, addressed by index. A pool handle is an index with a range invariant; linked structures link by index. Pools are a library on top of arrays, not a language mechanism.
   - Raw pointers exist only in `trust fn` signatures for FFI, with explicit permission contracts in the trust report.
   - Phase 5 becomes "static storage, pools and FFI permissions". A general heap with separation logic is out of v1; it needs its own ADR and builds on the value-level core (prophecies for stored `&mut`, as in Creusot).
2. **`refines g` is a contract, not a mechanism.**
   - `g` must be a `spec fn`. To show that an existing `impl fn h` and a faster `f` agree, write the reference as a `spec fn g` and prove that both refine it.
   - `f` has exactly one output: a result, or one `&mut` parameter `q`. `g` takes `f`'s parameters by value (`q` by its initial value) and returns that output.
   - `impl fn f(p…) refines g` is defined as `ensures result == g(p…)`, or `ensures *q == g(…, old(*q), …)` for a `&mut` output.
   - `f`'s `requires` must imply `g`'s `requires`; this is an obligation at `f`'s entry. Without it, `g(p…)` could be applied outside its domain.
   - `quodc fmt` keeps the `refines` spelling; the human report renders it as "behaves exactly like `g`". Phase 6 is reduced to defining this expansion and showing three measured speedups (its existing exit criterion).
3. **Resource contracts.** `stack <= n` stays in v1: with no recursion in v1 code and an acyclic call graph, it is a bound computed from per-function frame sizes. These come from the C compiler that builds the binary (gcc and clang: `-fstack-usage`), which is listed in the trust report. With a compiler that gives no per-function frame report, a build of a program with any `stack` clause fails; it is never skipped. Whether CompCert gives such a report is recorded at S3. `cost` and `mem` are moved out of v1 and marked speculative. `noalloc` stays as a spelling of "no `alloc` effect".
4. **Single-threaded.** v1 programs have one thread. Interrupt handlers and concurrency are out of v1; open-questions §6 stays open.
5. **Recursion.** v1 keeps ADR-0004's ban on recursion beyond phase 1 unless a later ADR lifts it with `decreases` and a stack bound. This matches Power of 10 rule 1.

## Consequences

- **Vision:** the "C alternative" claim becomes: *a verified systems language for embedded, safety-critical and algorithmic code, as a first target; general heap-based programs later.* The success criteria are unchanged.
- **Obligations and Core IR:** fixed by ADR-0007's encodings; no heap term will be added in v1. Stored proofs survive phases 2–5.
- **Trusted base:** none added by the scope itself. `stack` adds the C compiler's frame-size report as a listed trust item. FFI permission contracts are `trust` items, as before.
- **Agent loop:** obligations stay in linear integer arithmetic, arrays and uninterpreted bit-operation symbols, the theories agents and solvers handle best (experiments 0001–0003). Agents write index-linked structures instead of pointer-linked ones; the training-data cost of that is measured in S6.
- **Roadmap:** phase 5 shrinks (pools, static storage, FFI); phase 6 shrinks (expansion of `refines`); phase 7 keeps `stack`, defers `cost` and `mem`.
- Docs to update on acceptance: `vision.md` (what Quod is), `design-principles.md` §4 (no recursion in v1), §6 (refinement as a contract), §7 (resource contracts in v1), §8 (memory model), `power-of-10-mapping.md` rules 1, 3, 9, `roadmap.md` phases 5–7, `open-questions.md` §6 and §13, `examples/02_ring_buffer.qd` and `examples/03_refinement_sum.qd`.

## Validation

- Phase-9 corpus: every task is expressible in v1 (no task needs a general heap). If a large share of the planned corpus (parsers, data structures) cannot be written with pools, v1 is too narrow.
- Pools: a verified pool library with an index-linked list and a free list, with obligations only in the theories above.
- `refines`: the three phase-6 refinements check with the expansion and nothing else.
- **Gate, decision 3 (S3):** gcc and clang give per-function frame sizes for every function in the slice corpus's C; CompCert's support is recorded either way.
- **Gate, decision 1 (S6):** at least 2 of the 10 S6 tasks use index-linked structures in a pool. Pass: agents solve them at no worse a rate than the other S6 tasks. If they solve the pointer-based versions in the baseline but not the pool versions in Quod, decision 1 is revisited before acceptance.
- Revisit if agents in S6 or phase 9 consistently fail on index-linked structures where they succeed on pointer-linked ones in the baseline.
