# ADR-0001: Rust for the bootstrap compiler and the kernel

- **Status:** Accepted
- **Date:** 2026-11-19
- **Deciders:** project maintainer (accepted 2026-11-19)
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** [open-questions.md §1](../open-questions.md#1-implementation-language)

## Context

Phase 1 cannot start until the implementation language is chosen ([CLAUDE.md](../../CLAUDE.md) forbids compiler code until then). The code will be written almost entirely by AI agents, so the deciding factor is how reliably agents produce, debug and maintain code in each language, not human preference. Two components have different requirements:

- **Compiler, oracle, emitter, report generators** (untrusted, large): need a solid ecosystem (JSON-RPC, parser testing, fuzzing), cross-platform builds, and determinism.
- **Kernel** (trusted, small): needs auditability, minimal dependencies, bit-for-bit determinism, and a credible path to being verified or independently re-checked ([design-principles.md §2](../design-principles.md#2-small-trusted-kernel-de-bruijn-criterion)).

## Options considered

### Option A: Rust for everything, kernel as an isolated crate
- For: agents write Rust with high reliability; the type and ownership system catches agent errors at compile time; mature ecosystem (`serde`, `proptest`, `cargo-fuzz`, `insta` for golden tests); single static binary; easy C interop for testing emitted C; no GC; Kani and Verus can verify parts of the kernel later.
- Against: verifying a Rust kernel end-to-end is still immature; slower to iterate on IR passes than in an ML-family language; default `HashMap` iteration order is randomized (must be avoided for determinism).

### Option B: OCaml for everything
- For: the traditional compiler language (CompCert, Rocq, F\* lineage); algebraic data types and pattern matching suit IR passes; a kernel could be extracted from Rocq.
- Against: smaller ecosystem for servers and tooling; agents are somewhat less fluent; no advantage over Rust that matters for this project.

### Option C: Lean 4 for everything
- For: compiler and kernel in a language that can prove properties of its own kernel; compiles to C.
- Against: agent output in Lean 4 is markedly less reliable (Lean 3 / Lean 4 confusion in training data, fast-moving Mathlib APIs), which is exactly the hallucination risk Quod is designed to avoid; weaker ecosystem for JSON servers; toolchain churn.

### Option D: Rust compiler + kernel in Lean or Rocq
- For: compiler gets Rust's ecosystem; kernel gets a proof assistant.
- Against: two toolchains from day one; the kernel is the piece agents must iterate on most carefully while its format (ADR-0003) stabilizes, and that is where Lean's reliability cost bites hardest.

## Decision

Use **Rust** for the bootstrap compiler, oracle, emitter and report generators, and for the **kernel as a separate crate** under `kernel/` with these constraints:

- no dependencies outside `core`/`alloc`/`std` (not even other workspace crates);
- `#![forbid(unsafe_code)]`;
- no `HashMap`/`HashSet` (use `BTreeMap`/`BTreeSet` or vectors); no floats, threads, clocks, randomness, environment or file-system access inside checking;
- a line budget (initial target: ≤ 5,000 lines excluding tests), recorded in `kernel/README.md` and enforced in CI.

Independent assurance comes later from a **second kernel implementation in Lean 4** (or another proof assistant), checking the same proof format. Writing it becomes cheap once the format is stable and the kernel is small.

## Consequences

- Phase 1 can start: Cargo workspace with `compiler/`, `kernel/`, `oracle/` crates.
- **Trusted base:** the Rust kernel crate plus `rustc` and the platform it runs on. Listed in the trust report. The future Lean re-checker would reduce reliance on any single implementation.
- **Agent loop:** compile-time errors from `rustc` catch many agent mistakes in the toolchain itself before tests run.
- Self-hosting (writing `quodc` in Quod) remains a long-term option and would need its own ADR.
- Docs to update on acceptance: `open-questions.md` §1, `architecture.md` (component table), `kernel/README.md` (budget and constraints), `CLAUDE.md` (lift the no-code rule for phase 1).

## Validation

- Kernel stays within its line budget through phase 2.
- Identical kernel verdicts on the test corpus across at least two OS/architecture pairs (phase-2 exit criterion).
- Revisit if: agents repeatedly struggle with Rust for IR transformations (measured by iteration counts on compiler tasks), or a Lean re-checker turns out cheap enough to become the primary kernel.
