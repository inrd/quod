# ADR-0013: Agents write annotations; small obligations give proof stability

- **Status:** Proposed
- **Date:** 2026-11-20
- **Deciders:** —
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** none. Rewords thesis 1 and 3 of [vision.md](../vision.md).
- **Depends on:** [ADR-0009](0009-prove-check-split.md) (`prove`/`check`), [ADR-0007](0007-trusted-obligation-generator.md) (obligation IDs, fingerprints, `vcgen`), [ADR-0008](0008-kernel-rules-from-real-certificates.md) (proof format)
- **Evidence gate:** S6 of [ADR-0006](0006-thin-end-to-end-slice.md) (pending)
- **Revised:** 2026-11-20 (split out of ADR-0009 under [ADR-0012](0012-evidence-before-acceptance.md)'s no-partial-acceptance rule)

## Context

The vision says agents write explicit proofs and the kernel checks them, and that explicit proof artifacts fix the brittleness of SMT-based verifiers. [Experiment 0001](../experiments/0001-alethe-elaboration.md) shows what those proofs look like: a median of ~230 Alethe steps of Farkas coefficients, rewrite instances and congruence chains. No agent will write or repair these by hand at useful speed. In practice the loop is:

```
agent edits annotations ──► vcgen ──► cvc5 (search) ──► elaborator ──► kernel
   ▲   (invariant, assert,                                                │
   │    ghost code, lemma call)                                           │
   └───────────── counterexample / failing obligation ◄───────────────────┘
```

This is the Dafny/Verus loop with a certifying back end ([ADR-0009](0009-prove-check-split.md)). Two claims in the current docs do not survive that:

| Claim | Problem |
|---|---|
| Stored proofs give stability | Any edit to an obligation changes its fingerprint; the solver runs again, and brittleness returns for that obligation. Stability comes from **small obligations**, not from stored certificates. |
| The oracle's main job is step-level proof errors | Agents act on *which annotation is missing*: failing obligation, counterexample, goal and context. Step errors matter only for hand-written fallbacks. |

A third claim needs a qualifier: "verbosity is free" (vision thesis 3). Agents read far more than they write, and the phase-9 metric counts tokens. Redundancy costs context each time a function is read.

## Options considered

### Option A: Keep "agents write proofs" as the model
- For: no doc change; the kernel format already allows hand-written steps.
- Against: describes a loop no agent will run; points oracle design (`proof/submit` errors) at the wrong problem.

### Option B: Annotations are the agent's artifact; stability comes from obligation size
- For: matches what experiment 0001 shows; keeps every soundness property (the kernel still checks every obligation); gives agents a lever they control (where obligations are cut).
- Against: proof stability now depends on obligation size and solver behavior, which Quod does not control; it has to be measured, not assumed.

## Decision

Option B.

1. **The agent's artifact is annotations**: `invariant`, `decreases`, `assert`, ghost code, `lemma` declarations and calls. Proof steps are produced by the untrusted pipeline (solver → elaborator). Hand-written steps through `proof/submit` stay possible as a fallback.
2. **Stability comes from obligation size.** `vcgen` cuts obligations at every function boundary (contracts only, principle 10) and at every `assert`: the asserted fact is one obligation, and a hypothesis for what follows. Agents control obligation size by adding `assert`s and lemmas. The oracle reports obligation size so agents can see when to split. The cut rule is `vcgen` behavior and is restated normatively in `spec/core-ir.md`.
3. **Oracle priorities** (phase 3): `obligations/list`, `goal/at`, `check/counterexample` and `wp/suggest` come first; step-level `proof/submit` diagnostics come last.
4. **Verbosity is cheap to write, not to read.** Redundant statements are kept where the checker checks them and they catch real mistakes (types on `let`, effects, struct field order). Redundancy that only restates what the reader already sees is not added. Tokens read per task count in the phase-9 metric.

## Consequences

- **Trusted base:** none added. The cut at `assert` is part of `vcgen`'s trusted behavior; [ADR-0007](0007-trusted-obligation-generator.md)'s soundness cases cover it (a false `assert` is unprovable even when the code after it is correct).
- **Agent loop:** the agent edits annotations and reads counterexamples; it never needs to read Alethe. Proof size matters for storage and `check` time, not for agent context.
- **Proof stability becomes a measured property**: the share of obligations whose certificate must be regenerated after an unrelated edit (target: none, since unrelated edits do not change their fingerprint), and the share of `prove` runs that fail after a related but semantics-preserving edit.
- Docs to update on acceptance: `vision.md` (thesis 1: the oracle answers about annotations; thesis 3: reading cost), `design-principles.md` §11 (oracle priorities), `oracle-protocol.md` (request order, obligation size field), `prior-art.md` lesson 1 (stability comes from small obligations plus stored certificates).

## Validation

Pass criteria, fixed before S6 runs:

- **Annotations, not steps:** hand-written proof steps are under 10% of classified agent edits across the 10 tasks. If they are more, decision 1's premise is wrong.
- **Stability:** for a fixed set of semantics-preserving edits to the S6 solutions, the share of `prove` runs that fail is no higher than the share of verification failures for the same edits in the Dafny/Verus baseline. If Quod is not at least as stable, the stability claim in the vision goes.
- Revisit if solvers stop producing certificates for a common obligation class, so agents must write steps by hand often.
