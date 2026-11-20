# Roadmap

Phases are ordered by dependency. Before widening phase 1, work follows a thin end-to-end slice through phases 1–3 ([ADR-0006](adr/0006-thin-end-to-end-slice.md); see [Thin slice](#thin-slice-adr-0006)). Each has **exit criteria** that must all hold before the phase is considered done. Later phases are more speculative and will be revised by ADRs.

The key project metric (phase 9) should be measured informally from phase 3 onward, so design choices can be judged by their effect on the agent loop.

---

## Phase 0 — Scaffold

Repo layout, docs, ADR process, example sketches, test layout.

**Exit criteria**
- [x] `README.md`, `CLAUDE.md`, all `docs/*.md` listed in the README exist.
- [x] ADR template exists; workflow documented in `CLAUDE.md`.
- [x] Provisional phase-1 grammar draft in `spec/`.
- [x] Example sketches in `examples/`, each marked provisional.
- [x] Accepted ADRs for: implementation language ([ADR-0001](adr/0001-implementation-language.md)), arithmetic semantics ([ADR-0002](adr/0002-arithmetic-semantics.md)), kernel logic and proof format ([ADR-0003](adr/0003-kernel-logic-and-proof-format.md)).

## Thin slice (ADR-0006)

The narrowest path from source to a kernel-checked function, built before phase 1's exit criteria are pursued in full. It tests the riskiest assumptions early: certificates fit the kernel rules, the agent loop beats an existing verifier, and spec review tooling works. Slice code is limited to the slice subset and follows every repository rule. Steps S2b–S4a build to Proposed ADRs 0007, 0008 and 0010 as their evidence-gate experiments ([ADR-0012](adr/0012-evidence-before-acceptance.md)).

| Step | Scope | Status |
|---|---|---|
| S1 | Elaboration spike: cvc5 certificates against kernel rules | done ([experiment 0001](experiments/0001-alethe-elaboration.md); ADR-0003 superseded by Proposed ADR-0008) |
| S2 | Slice subset in `spec/`: `u32`, `u64`, `usize`, `bool`, fixed arrays, `&`/`&mut` slices, `while` with `invariant`, `requires`/`ensures` | done ([contracts.md](../spec/contracts.md); parser, formatter, type checker) |
| S2b | Core IR for the slice and its canonical text ([core-ir.md](../spec/core-ir.md)); front end lowers to it | done: spec, `coreir` crate, lowering (`quodc lower`) |
| S3a | C subset written down ([c-subset.md](../spec/c-subset.md), required by ADR-0010 before `cgen`) | done: spec; in-place `mut` slices need Proposed [ADR-0014](adr/0014-alias-free-views-in-core-ir.md) |
| S3 | `cgen` over Core IR, phase-1 traps; C accepted by gcc, clang, CompCert (gates for ADR-0010, ADR-0014) | next |
| S4a | `vcgen`, elaborator prototype, stand-in checkers (Carcara, Ethos); experiment 0004 (gates for ADR-0007, ADR-0008) | — |
| S4b | Kernel rules for the slice; replaces the stand-in checker | — |
| S5 | Minimal oracle over JSON stdio | — |
| S6 | Agent run: 10 tasks against a Dafny/Verus baseline (gates for ADR-0011, ADR-0013) | baseline half run on three task sets ([experiments 0005](experiments/0005-s6-baseline.md), [0006](experiments/0006-s6-hard-baseline.md), [0007](experiments/0007-s6-change-baseline.md): easy, hard, spec changes): solved by the baseline in 59 of 60 runs; Quod half pending |
| S7 | Spec-review spike on existing Dafny/Verus programs; off the critical path | may start at any time |

After S6 and S7, phases continue in the order below, widening from the slice.

## Phase 1 — Minimal core

Integers with explicit widths (`u8`…`u64`, `i8`…`i64`, `usize`), `bool`, structs, fixed arrays `[T; N]`, slices `&[T]`, functions, bounded loops. Parser, type checker, C emitter, canonical formatter. Implemented in Rust as a Cargo workspace ([ADR-0001](adr/0001-implementation-language.md)).

**Exit criteria**
- Grammar is LL(1) (checked by a tool) and matches `spec/grammar-draft.md`, promoted to `spec/grammar.md` by ADR.
- Formatter is idempotent and `parse(format(x)) == parse(x)` on the whole test corpus.
- Type checker rejects every implicit conversion and overloading attempt in a negative test suite.
- Every operation precondition that is not statically trivial (overflow, bounds, division, shifts, casts) is compiled as a runtime trap and listed in an "unverified checks" build report ([ADR-0002](adr/0002-arithmetic-semantics.md#phase-1-transition)). Proof obligations replace these traps in phase 2.
- C output compiles warning-free with `-Wall -Wextra -Werror` under `-fsanitize=undefined` and passes the test corpus with zero sanitizer reports.
- Diagnostics have stable codes.

## Phase 2 — Contracts and obligations

`requires`/`ensures`/`old`, type invariants (`where`), loop invariants, `decreases`, `assert`, spec-only `int`/`nat`. Proof obligation generation. Kernel checking flat Alethe-subset proofs ([ADR-0003](adr/0003-kernel-logic-and-proof-format.md)). Untrusted elaborator from solver certificates.

**Exit criteria**
- Every phase-1 runtime trap is replaced by a proof obligation; unproven obligations are compile errors and the "unverified checks" report is empty on the corpus.
- All rules accepted in ADR-0003 implemented; the trusted axiom list lives in one versioned file and is printed in the trust report.
- Kernel crate respects the ADR-0001 constraints and its line budget (≤ 5,000 lines excluding tests), enforced in CI.
- Kernel checking is deterministic: identical verdicts across ≥ 2 OSes/architectures on the corpus.
- cvc5 Alethe certificate → elaborator → kernel works end-to-end on the corpus.
- Soundness test suite: known-false obligations, plus malformed applications of every accepted rule, all rejected.

## Phase 3 — Oracle server

JSON protocol ([oracle-protocol.md](oracle-protocol.md)), typed holes, counterexamples, obligation task lists, stable diagnostic IDs.

**Exit criteria**
- All requests in the protocol draft implemented, with a versioned JSON Schema.
- Obligation IDs stable across a defined set of unrelated edits (tested).
- Counterexamples are concrete and replayable on the corpus.
- An agent harness can drive a function from `?h` to verified using only the protocol (no free-text parsing).

## Phase 4 — Effects and capabilities

`effects { … }`, effect checking, capability values, effect changes in reports.

**Exit criteria**
- Calls with undeclared effects are rejected.
- `io`, `alloc`, `panic`, `blocking`, `unsafe`, `time` implemented; effect polymorphism design accepted by ADR.
- Effect diff tool reports effect changes between two versions of a module.

## Phase 5 — Memory model

Ownership plus explicit permissions (separation-logic style), regions/arenas, `trust` axioms for FFI.

**Exit criteria**
- Formal description of the permission model in `spec/`.
- Frame reasoning works: callee contracts specify footprint; unchanged locations need no proof.
- Region allocator library verified in Quod.
- C FFI via `trust fn` with provenance; all appear in the trust report.

## Phase 6 — Refinement

`spec fn`, `impl fn … refines`, equivalence proofs.

**Exit criteria**
- `spec fn` may use non-executable constructs (mathematical integers, quantifiers) and generate no code.
- At least three non-trivial refinements verified (e.g. SIMD sum, unrolled memcpy, bit-twiddling popcount) with measured speedup over the reference.

## Phase 7 — Resource contracts

`cost`, `stack`, `noalloc`, `mem`.

**Exit criteria**
- `stack` bounds proven and validated against measured stack use for the C output on at least one target.
- `noalloc` ≡ absence of `alloc` effect, checked both ways.
- Cost model chosen by ADR.

## Phase 8 — Spec tooling

Mutation-based weakness detection, witness generation, human report generator, trust report.

**Exit criteria**
- Human report generated per module: contracts in plain language, examples, effects, resource bounds, trust assumptions, spec warnings.
- Mutation tool finds seeded spec weaknesses on a benchmark of deliberately weak specs (target recall set by ADR).
- Trust report lists every `trust`, `assume` and toolchain trust item, transitively; diffable in CI.
- User study (even small): reviewers detect spec/intent mismatches faster with the report than with source.

## Phase 9 — Agent loop benchmark

Measure how fast an agent drives functions from holes to verified on a fixed task corpus. **This is the project's key metric.**

**Exit criteria**
- Fixed, versioned task corpus (spec given, body = `?h`), spanning arithmetic, arrays, data structures, parsing, numeric kernels.
- Metrics defined and reported: success rate, wall time, oracle round-trips, tokens, proof size, trust items introduced.
- Baselines: same tasks in at least one existing verifier (e.g. Dafny or Verus).
- Benchmark runs reproducibly in CI.
