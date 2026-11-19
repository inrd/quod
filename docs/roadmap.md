# Roadmap

Phases are ordered by dependency ([ADR-0006](adr/0006-thin-end-to-end-slice.md) proposes a thin end-to-end slice through phases 1–3 first). Each has **exit criteria** that must all hold before the phase is considered done. Later phases are more speculative and will be revised by ADRs.

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
