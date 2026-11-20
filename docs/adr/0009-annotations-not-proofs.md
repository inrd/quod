# ADR-0009: Agents write annotations; solvers produce certificates; the kernel checks them

- **Status:** Proposed
- **Date:** 2026-11-20
- **Deciders:** —
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** none. Narrows [open-questions.md §12](../open-questions.md#12-role-of-the-smt-solver) (solver stays agent-side; adds a `prove`/`check` split). Rewords thesis 1 and 3 of [vision.md](../vision.md).
- **Depends on:** ADR-0003 / [ADR-0008](0008-kernel-rules-from-real-certificates.md) (proof format), [ADR-0007](0007-trusted-obligation-generator.md) (obligation IDs and fingerprints)

## Context

The vision says agents write explicit proofs and the kernel checks them, and that explicit proof artifacts fix the brittleness of SMT-based verifiers. [Experiment 0001](../experiments/0001-alethe-elaboration.md) shows what those proofs look like: a median of ~230 Alethe steps of Farkas coefficients, rewrite instances and congruence chains. No agent will write or repair these by hand at useful speed. In practice the loop is:

```
agent edits annotations ──► vcgen ──► cvc5 (search) ──► elaborator ──► kernel
   ▲   (invariant, assert,                                                │
   │    ghost code, lemma call)                                           │
   └───────────── counterexample / failing obligation ◄───────────────────┘
```

This is the Dafny/Verus loop with a certifying back end. Two claims in the current docs do not survive that:

| Claim | Problem |
|---|---|
| Stored proofs give stability | Any edit to an obligation changes its fingerprint; the solver runs again, and brittleness returns for that obligation. Stability comes from **small obligations**, not from stored certificates. |
| The oracle's main job is step-level proof errors | Agents act on *which annotation is missing*: failing obligation, counterexample, goal and context. Step errors matter only for hand-written fallbacks. |

A third claim needs a qualifier: "verbosity is free" (vision thesis 3). Agents read far more than they write, and the phase-9 metric counts tokens. Redundancy costs context each time a function is read.

## Options considered

### Option A: Keep "agents write proofs" as the model
- For: no doc change; the kernel format already allows hand-written steps.
- Against: describes a loop no agent will run; points oracle design (`proof/submit` errors) at the wrong problem.

### Option B: Annotations are the agent's artifact; certificates are generated and cached
- For: matches what experiment 0001 shows; keeps every soundness property (the kernel still checks every obligation); makes the deterministic part (checking) and the non-deterministic part (search) separate commands.
- Against: proof stability now depends on obligation size and solver behavior, which Quod does not control; it has to be measured, not assumed.

### Option C: Trust solver verdicts for some fragments
- For: fastest loop.
- Against: grows the trusted base; already rejected by ADR-0003 for now.

## Decision

Option B.

1. **The agent's artifact is annotations**: `invariant`, `decreases`, `assert`, ghost code, `lemma` declarations and calls. Proof steps are produced by the untrusted pipeline (solver → elaborator). Hand-written steps through `proof/submit` stay possible as a fallback.
2. **Two commands, two guarantees.**

   | Command | Does | Deterministic | Trusted |
   |---|---|---|---|
   | `quodc prove` | runs `vcgen`, the solver and the elaborator on obligations whose stored certificate is missing or stale; writes certificates | no (solver search, resource limits) | no |
   | `quodc check` | runs `vcgen` and the kernel on stored certificates; never calls a solver | yes | kernel + `vcgen` |

   CI and release builds run `check` only. A missing certificate is a `check` error, never a timeout.
3. **Certificates are cache entries**, stored next to the source, keyed by obligation ID and fingerprint ([ADR-0007](0007-trusted-obligation-generator.md)). A certificate whose fingerprint still matches is reused without running the solver.
4. **Stability comes from obligation size.** `vcgen` cuts obligations at every function boundary (contracts only, principle 10) and at every `assert`: the asserted fact is one obligation, and a hypothesis for what follows. Agents control obligation size by adding `assert`s and lemmas. The oracle reports obligation size so agents can see when to split.
5. **Oracle priorities** (phase 3): `obligations/list`, `goal/at`, `check/counterexample` and `wp/suggest` come first; step-level `proof/submit` diagnostics come last.
6. **Verbosity is cheap to write, not to read.** Redundant statements are kept where the checker checks them and they catch real mistakes (types on `let`, effects, struct field order). Redundancy that only restates what the reader already sees is not added. Tokens read per task count in the phase-9 metric.

## Consequences

- **Trusted base:** none added. `prove` and its solver/elaborator stay outside; `check` is the same kernel + `vcgen` path as before.
- **Agent loop:** the agent edits annotations and reads counterexamples; it never needs to read Alethe. Proof size matters for storage and `check` time, not for agent context.
- **Determinism:** the build verdict depends only on source + stored certificates + compiler version. Solver non-determinism is confined to `prove`.
- **Proof stability becomes a measured property**: the share of obligations whose certificate must be regenerated after an unrelated edit (target: none, since unrelated edits do not change their fingerprint), and the share of `prove` runs that fail after a related but semantics-preserving edit.
- Docs to update on acceptance: `vision.md` (thesis 1: the oracle answers about annotations; thesis 3: reading cost), `design-principles.md` §2 (certificates as cached artifacts, `prove`/`check`), §11 (oracle priorities), `architecture.md` (data artifacts: certificates; two commands), `oracle-protocol.md` (request order, obligation size field), `open-questions.md` §12, `prior-art.md` lesson 1 (stability comes from small obligations plus stored certificates).

## Validation

- ADR-0006 S6: classify every agent edit (annotation, code, hand-written proof step). Expect hand-written steps to be rare; if they are common, this ADR's premise is wrong.
- S6: count `prove` failures after semantics-preserving edits, versus the same edits in the Dafny/Verus baseline. If Quod is not more stable, the stability claim in the vision must go.
- Revisit if solvers stop producing certificates for a common obligation class, so agents must write steps by hand often.
