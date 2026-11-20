# ADR-0009: Separate `quodc prove` and `quodc check`; certificates are cached artifacts

- **Status:** Accepted
- **Date:** 2026-11-20
- **Deciders:** project maintainer (accepted 2026-11-20)
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** none. Narrows [open-questions.md §12](../open-questions.md#12-role-of-the-smt-solver) (the solver stays agent-side; adds a `prove`/`check` split).
- **Depends on:** [ADR-0008](0008-kernel-rules-from-real-certificates.md) (proof format), [ADR-0007](0007-trusted-obligation-generator.md) (obligation IDs and fingerprints)
- **Evidence gate:** none (tool architecture: no trusted-base change and no claim about solver or agent behavior)
- **Revised:** 2026-11-20 (split: the decisions that rest on agent behavior moved to [ADR-0013](0013-annotations-and-small-obligations.md))

## Context

Checking must be deterministic (CLAUDE.md design invariants): no timeouts, no randomness, no solver calls inside the kernel. Finding a proof is not deterministic. Solver results depend on resource limits, versions and options, and [experiment 0001](../experiments/0001-alethe-elaboration.md) shows that certificates are large (median ~230 Alethe steps), produced by cvc5 plus an elaborator.

If one command both searched for and checked proofs, the build verdict would depend on solver behavior. A CI build could then fail on a timeout without any source change. Dafny and Verus work this way.

## Options considered

### Option A: One command that runs the solver on every build
- For: no stored artifacts; the familiar Dafny/Verus model.
- Against: the build verdict depends on solver version, limits and luck. That breaks the determinism invariant.

### Option B: Two commands; certificates stored and reused
- For: the verdict depends only on source, stored certificates and compiler version; the solver's non-determinism is confined to one command that is never part of the trusted path.
- Against: certificates must be stored and kept in step with the source.

### Option C: Trust solver verdicts for some fragments
- For: fastest loop.
- Against: grows the trusted base; already rejected by ADR-0003 and ADR-0008.

## Decision

Option B.

1. **Two commands, two guarantees.**

   | Command | Does | Deterministic | Trusted |
   |---|---|---|---|
   | `quodc prove` | runs `vcgen`, the solver and the elaborator on obligations whose stored certificate is missing or stale; writes certificates | no (solver search, resource limits) | no |
   | `quodc check` | runs `vcgen` and the kernel on stored certificates; never calls a solver | yes | kernel + `vcgen` |

   CI and release builds run `check` only. A missing or stale certificate is a `check` error naming the obligation ID, never a timeout.
2. **Certificates are cache entries**, stored next to the source and keyed by obligation ID and fingerprint ([ADR-0007](0007-trusted-obligation-generator.md)). A certificate whose fingerprint still matches is reused without running the solver. A hand-written proof submitted through `proof/submit` is stored the same way. The file format and location are specified in `spec/` before S4a.

## Consequences

- **Trusted base:** none added. `prove` and its solver and elaborator stay outside; `check` is the same kernel + `vcgen` path as before.
- **Determinism:** the build verdict depends only on source, stored certificates and compiler version.
- **Agent loop:** agents run `prove` while editing and read its failures; `check` tells them exactly which obligations lack a certificate. What agents edit, and how stable certificates are across edits, is [ADR-0013](0013-annotations-and-small-obligations.md).
- Stored certificates become part of the repository a Quod project ships.
- Docs to update on acceptance: `design-principles.md` §2 (certificates as cached artifacts, `prove`/`check`), `architecture.md` (data artifacts: certificates; two commands), `open-questions.md` §12.

## Validation

- At S4b: `check` gives byte-identical output on the slice corpus on two OSes or architectures, from the same stored certificates.
- Revisit if stored certificates become too large to keep next to the source.
