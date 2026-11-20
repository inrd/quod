# ADR-0006: Build a thin end-to-end slice before widening phases 1–3

- **Status:** Proposed
- **Date:** 2026-11-19
- **Deciders:** —
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** none. Amends the order of work in [roadmap.md](../roadmap.md); phase exit criteria are unchanged.
- **Depends on:** [ADR-0008](0008-kernel-rules-from-real-certificates.md) (kernel logic and proof format; supersedes ADR-0003); S2b–S4b build to [ADR-0007](0007-trusted-obligation-generator.md), ADR-0008 and [ADR-0010](0010-trusted-c-emitter-from-core-ir.md) (all Proposed) under [ADR-0012](0012-evidence-before-acceptance.md)'s rule for gated designs
- **Evidence gate:** none (process decision)
- **Revised:** 2026-11-20 (S2b, S4a/S4b split with a stand-in checker, S6 error classes, S7 moved off the critical path; later the same day: S1 result, no fallback emitter, pool tasks in S6)

## Context

The roadmap is ordered by dependency: each phase is completed in full before the next starts. Two of the project's make-or-break assumptions are only tested late:

| Assumption | Tested in | If false |
|---|---|---|
| Solver certificates elaborate into ADR-0003's strict rule subset for realistic obligations (overflow with `*`, bitwise code, loop invariants) | phase 2 | ADR-0003 must be revisited; kernel rules and proof format change |
| An agent drives holes to verified code faster than in an existing verifier | phase 9 (informally from phase 3) | the reason for a new language weakens ([vision.md](../vision.md#why-a-new-language)) |
| Tooling can help humans judge spec ↔ intent | phase 8 | Quod is a stricter Dafny with no training data |

Phase 1 already has a parser, formatter and type checker. Widening phase 1 further (emitter for every construct, LL(1) tool check, full UBSan corpus) delays all three tests without reducing their risk. CLAUDE.md currently allows code for the current phase only, so a slice needs a decision.

## Options considered

### Option A: Keep strict phase order
- For: simplest rule; no partially built components.
- Against: the riskiest assumptions are tested after most of the work is done; a negative result in phase 2 or 8 invalidates design built on top of it.

### Option B: Thin slice through phases 1–3 (plus small spikes for 8 and 9), then widen
- For: tests the three assumptions within weeks; gives the key metric an early, real number; results shape the full phase designs (which rules the kernel needs, which oracle requests agents actually use).
- Against: slice code may be rewritten when phases are completed properly; risk of the slice quietly becoming the architecture.

### Option C: Spikes only (no slice code in the workspace)
- For: no throwaway code in the repo.
- Against: tests the elaboration assumption, but not the agent loop, which needs a working kernel, obligation generator and oracle.

## Decision

Option B. Before widening phase 1, build the narrowest path from source to a kernel-checked function, in this order:

| Step | Scope | Output |
|---|---|---|
| S1 Elaboration spike (done: 61 obligations) | Hand-written SMT-LIB obligations shaped like corpus code: bounds, `+`/`*` overflow, loop invariants, `&`/`>>`. Run cvc5 with Alethe output; classify every step by rule. No kernel code. | `docs/experiments/0001-alethe-elaboration.md`: share of steps and obligations inside ADR-0003's accepted subset, and which rejected rules appear |
| S2 Slice subset | `u32`, `u64`, `usize`, `bool`, fixed arrays, `&`/`&mut` slices, `while` with `invariant`, `requires`/`ensures`. No structs, no casts. | grammar/typing additions written in `spec/` first (docs-first rule) |
| S2b Core IR | Core IR for the slice subset and its canonical text form ([ADR-0007](0007-trusted-obligation-generator.md)); front end lowers to it | `spec/core-ir.md` (slice part), lowering in `compiler/` |
| S3 C emitter | `cgen` over Core IR ([ADR-0010](0010-trusted-c-emitter-from-core-ir.md)), slice subset only, with phase-1 runtime traps | UBSan-clean C on the slice corpus, accepted by gcc, clang and CompCert |
| S4a Obligation generator + stand-in checker | `vcgen` for the slice; elaborator prototype; certificates from cvc5 checked by an **existing external checker** (Carcara for Alethe, Ethos for CPC) | end-to-end path from source to checked certificates, without Quod's kernel; experiment 0004 (ADR-0008's gate) |
| S4b Kernel | kernel rules for the slice ([ADR-0008](0008-kernel-rules-from-real-certificates.md)); replaces the stand-in checker | kernel checks the same certificates as S4a |
| S5 Minimal oracle | `obligations/list`, goal at an obligation, counterexample, `proof/submit` | JSON over stdio |
| S6 Agent run | 10 tasks (spec given, body `?h`), at least 2 of them on index-linked structures in a fixed-capacity pool ([ADR-0011](0011-v1-scope.md)); same tasks in Dafny or Verus as baseline | phase-9 metrics, informal: success, round trips, tokens, proof size; every agent edit classified (annotation, code, hand-written proof step) and every agent error classified (syntax, "false friend" from Rust or C, typing, verification) |
| S7 Spec-review spike | **independent of S2–S6; may start at once.** On existing Dafny or Verus programs: seed spec weaknesses, run mutation and witness generation, count how many are caught; measure spec size against implementation size per function | `docs/experiments/`: evidence for or against the phase-8 approach and the "specs stay small" premise |

S4b and S5–S6 are independent: S6 may run on the stand-in checker. Results that depend on the stand-in checker are measurements of the agent loop, not soundness claims, and are labeled so. After S6 and S7, the phases continue in roadmap order and widen from the slice. Slice code follows every repository rule (determinism, kernel constraints, diagnostics registry). It is not exempt from review because it is a slice.

## Consequences

- Phase 2–3 code (kernel, obligation generator, oracle) appears before phase 1 is complete, **limited to the slice subset**. CLAUDE.md's "current phase only" rule must say so on acceptance.
- **Trusted base:** none added by this ADR. The kernel and obligation generator built in S4 are the same components the roadmap already plans; their trust status is set by ADR-0008 and [ADR-0007](0007-trusted-obligation-generator.md).
- **Agent loop:** gets measured from S6 instead of phase 3–9, so later design choices (oracle requests, proof format details) are judged by data.
- S1 needs cvc5 installed locally; it is not a build dependency. S4a needs Carcara and Ethos the same way: tools for the slice measurement, never build dependencies and never in the trusted base.
- S2b, S3 and S4a are built to the designs of ADR-0007, ADR-0008 and ADR-0010 while those are `Proposed`. That code is their evidence-gate experiment ([ADR-0012](0012-evidence-before-acceptance.md)); each ADR is accepted when its step's report is in. No fallback (such as an untrusted emitter from the AST) is built in parallel. If one of those ADRs is rejected, the code built to it is revised to the decision that replaces it.
- Docs to update on acceptance: `roadmap.md` (slice section before phase 1 exit criteria), `CLAUDE.md` (code rule), `docs/experiments/` (new directory for spike reports).

## Validation

- S1 decided ADR-0003: 0 of 61 obligations stayed inside its accepted rules ([experiment 0001](../experiments/0001-alethe-elaboration.md)), so ADR-0008 supersedes it. Experiment 0004 (S4a) applies the same ≥ 80% test to ADR-0008's rules.
- S6 decides whether the agent loop is promising: no fixed threshold, but a slice that is worse than the baseline on every metric is a reason to re-examine the language surface before widening. If syntax and "false friend" errors are a large share of agent errors, that is evidence for [open-questions.md §14](../open-questions.md#14-surface-syntax-own-or-rust-shaped).
- S7 decides whether spec review is worth phase 8: if mutation and witness tools catch few seeded weaknesses, or specs are not much smaller than implementations, the vision's thesis 2 must be revisited before widening.
- Revisit this ADR if the slice takes longer than finishing phase 1 as planned would have.
