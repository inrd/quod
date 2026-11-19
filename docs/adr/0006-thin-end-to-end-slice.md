# ADR-0006: Build a thin end-to-end slice before widening phases 1–3

- **Status:** Proposed
- **Date:** 2026-11-19
- **Deciders:** —
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** none. Amends the order of work in [roadmap.md](../roadmap.md); phase exit criteria are unchanged.
- **Depends on:** ADR-0003 (kernel logic and proof format)

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
| S1 Elaboration spike | ~50 hand-written SMT-LIB obligations shaped like corpus code: bounds, `+`/`*` overflow, loop invariants, `&`/`>>`. Run cvc5 with Alethe output; classify every step by rule. No kernel code. | `docs/experiments/0001-alethe-elaboration.md`: share of steps and obligations inside ADR-0003's accepted subset, and which rejected rules appear |
| S2 Slice subset | `u32`, `u64`, `usize`, `bool`, fixed arrays, `&`/`&mut` slices, `while` with `invariant`, `requires`/`ensures`. No structs, no casts. | grammar/typing additions written in `spec/` first (docs-first rule) |
| S3 C emitter | slice subset only, with phase-1 runtime traps | UBSan-clean C on the slice corpus |
| S4 Obligation generator + kernel | Core IR for the slice; kernel rules: propositional, equality, `la_generic`, `qd_int_cut`, `forall_inst`, sequence axioms | kernel checks cvc5-derived proofs end-to-end |
| S5 Minimal oracle | `obligations/list`, goal at an obligation, `proof/submit` | JSON over stdio |
| S6 Agent run | 10 tasks (spec given, body `?h`); same tasks in Dafny or Verus as baseline | phase-9 metrics, informal: success, round trips, tokens, proof size |
| S7 Spec-weakness spike | mutate the 10 verified bodies; count mutants that still verify | evidence for or against the phase-8 approach |

After S7, the phases continue in roadmap order and widen from the slice. Slice code follows every repository rule (determinism, kernel constraints, diagnostics registry). It is not exempt from review because it is a slice.

## Consequences

- Phase 2–3 code (kernel, obligation generator, oracle) appears before phase 1 is complete, **limited to the slice subset**. CLAUDE.md's "current phase only" rule must say so on acceptance.
- **Trusted base:** none added by this ADR. The kernel and obligation generator built in S4 are the same components the roadmap already plans; their trust status is set by ADR-0003 and [ADR-0007](0007-trusted-obligation-generator.md).
- **Agent loop:** gets measured from S6 instead of phase 3–9, so later design choices (oracle requests, proof format details) are judged by data.
- S1 needs cvc5 installed locally; it is not a build dependency.
- Docs to update on acceptance: `roadmap.md` (slice section before phase 1 exit criteria), `CLAUDE.md` (code rule), `docs/experiments/` (new directory for spike reports).

## Validation

- S1 decides whether ADR-0003 stands: if fewer than ~80% of obligations elaborate into the accepted subset without manual steps, or bit-vector obligations mostly fail, open a superseding ADR before S4.
- S6 decides whether the agent loop is promising: no fixed threshold, but a slice that is worse than the baseline on every metric is a reason to re-examine the language surface before widening.
- Revisit this ADR if the slice takes longer than finishing phase 1 as planned would have.
