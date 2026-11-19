# ADR-0007: A small trusted obligation generator over a specified Core IR

- **Status:** Proposed
- **Date:** 2026-11-19
- **Deciders:** —
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** trust-boundary item 1 in [architecture.md](../architecture.md#trust-boundaries) ("obligation generation is faithful")
- **Depends on:** ADR-0001 (Rust, kernel constraints), ADR-0003 (kernel logic)

## Context

ADR-0003 makes the kernel build each obligation's hypotheses and negated goal from the obligation itself, so a proof cannot smuggle in assumptions. It does not say where obligations come from. If the obligation generator is wrong (drops a precondition, mis-encodes a loop, forgets an overflow check), the kernel faithfully checks the wrong statements and the soundness claim fails. The generator must be decided before phase 2 fixes the obligation encoding.

A related gap: the front end turns source (including specs) into Core IR. If it mistranslates a spec, what is verified is not what the human reviewed.

## Options considered

### Option A: Untrusted generator, listed in the trust report
- For: no constraints on the generator; fastest to build.
- Against: the largest semantic component becomes a soundness hole; "modulo the VC generator" is close to "modulo the compiler", which is what Quod set out to avoid.

### Option B: Separate trusted crate (`vcgen/`) under kernel-style constraints, over a Core IR with a written semantics
- For: trusted code stays small and auditable; the kernel stays a pure logic checker (reusable, and the planned Lean re-checker can check proofs without re-deriving obligations); Core IR semantics in `spec/` is the reference for audits, tests and a later verified generator.
- Against: two trusted crates instead of one; Core IR must be small and frozen enough to specify.

### Option C: Kernel computes obligations from Core IR
- For: one trusted crate; one boundary.
- Against: puts program semantics inside the logic checker; the kernel's 5,000-line budget must then cover both; every Core IR change touches the kernel, which needs an ADR each time.

### Option D: Untrusted generator plus per-obligation certificates against a formal Core IR semantics (e.g. checked in Lean)
- For: smallest trusted base in the long run.
- Against: needs a mechanized Core IR semantics and a second proof pipeline; out of reach before the semantics is stable.

## Decision

Option B now, with Option D as the long-term path.

- `vcgen/` is a workspace crate with the kernel's constraints (std-only plus the `kernel` crate for term types, `#![forbid(unsafe_code)]`, no `HashMap`/`HashSet`, floats, clocks, threads, randomness, I/O during generation) and its own budget: **≤ 3,000 lines excluding tests**.
- Its input is **Core IR in a canonical text form**, specified in `spec/core-ir.md` before any `vcgen` code is written. The trusted path is: Core IR text → `vcgen` → obligations → kernel.
- The front end (source → Core IR) stays untrusted. To close the spec gap, **human reports are generated from Core IR**, not from source: the human reviews the spec that is actually verified. A round-trip test (source spec → Core IR → rendered spec) catches front-end mistranslation early but is not part of the trusted base.
- Obligation IDs and fingerprints are computed in `vcgen` from Core IR structure, so they are deterministic by construction.

## Consequences

- **Trusted base grows:** `vcgen` (≤ 3,000 lines) and the Core IR semantics document join the kernel. Both are listed in every trust report. This is not new trust: an obligation generator is trusted under every option except D; this ADR makes it small, constrained and visible.
- The kernel stays a pure first-order proof checker; its ADR-0003 rules and budget are unchanged.
- Core IR becomes a stable interface. Changes to its semantics need an ADR, like kernel rule changes.
- Compilation from Core IR to C (trust-boundary item 2) is still open and unaffected.
- **Agent loop:** none directly; agents see the same obligations. Reports from Core IR may read slightly less like the source; the report generator must render them carefully.
- Docs to update on acceptance: `architecture.md` (component table, trust boundaries), `CLAUDE.md` (crate list and constraints), `README.md` layout, new `spec/core-ir.md`, new `vcgen/README.md`.

## Validation

- `vcgen` stays within budget through phase 2.
- Soundness test suite (phase-2 exit criterion) gains generator cases: programs with known bugs whose obligations must be unprovable.
- Revisit if Core IR cannot be kept small enough to specify in one document, or if a mechanized semantics (Option D) becomes cheap.
