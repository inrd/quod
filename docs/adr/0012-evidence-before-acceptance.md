# ADR-0012: Trusted-base and behavior ADRs stay Proposed until their experiment has run

- **Status:** Proposed
- **Date:** 2026-11-20
- **Deciders:** —
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** none. Amends the ADR workflow in [adr/README.md](README.md) and `CLAUDE.md`.

## Context

ADR-0003 (kernel rules) was accepted on 2026-11-19. On 2026-11-20 its own validation criterion failed outright: 0 of 61 obligations stayed inside its accepted rules ([experiment 0001](../experiments/0001-alethe-elaboration.md)), and its sequence and bit-vector designs matched no solver's output. ADR-0008 now supersedes it. The decision was sound in shape but wrong in detail, and the detail was exactly what the experiment measured. Running the experiment first would have cost one day and saved an accepted-then-superseded ADR.

Most Quod ADRs are drafted by an agent and accepted by one maintainer. That makes cheap, external evidence more valuable, not less: it is the only check that does not depend on the author's own judgment.

Not every decision needs an experiment. ADR-0004 (phase-1 typing) records choices whose consequences are visible in the spec itself.

## Options considered

### Option A: Keep the current workflow
- For: fastest acceptance.
- Against: repeats ADR-0003's failure mode for every trusted-base decision.

### Option B: Require an experiment report before accepting ADRs of two kinds
- For: cheap; the experiments directory and format already exist; makes "Validation" sections binding rather than aspirational.
- Against: slows acceptance of some ADRs by days; needs judgment on which ADRs fall under it.

### Option C: Require experiments for every ADR
- For: one rule.
- Against: most front-end and process ADRs have nothing to measure.

## Decision

Option B.

An ADR is **evidence-gated** if it does either of these:

1. changes the **trusted base**: kernel rules, axioms or sorts; `vcgen` encodings; `cgen` output; Core IR semantics; or anything printed in the trust report as a toolchain item;
2. rests on a **claim about solver or agent behavior** (what certificates look like, what agents can write, what humans catch in review).

An evidence-gated ADR:

- names, in its Validation section, at least one experiment that could show it wrong, with a pass criterion stated before the experiment runs;
- stays `Proposed` until that experiment has a report in `docs/experiments/` and the ADR cites the result;
- if the experiment fails its criterion, is revised or rejected, not accepted with a note.

Other ADRs follow the existing workflow. The ADR's header states which applies: `- **Evidence gate:** <experiment> (pending | passed)` or `- **Evidence gate:** none (front-end/process decision)`.

## Consequences

- Current Proposed ADRs:

  | ADR | Gate | State |
  |---|---|---|
  | 0006 thin slice | none (process) | — |
  | 0007 trusted `vcgen` | trusted base | pending: `spec/core-ir.md` for the slice and the generator soundness cases it lists |
  | 0008 kernel rules | trusted base | passed for its §4 and §5 (experiments 0002, 0003); its first criterion (≥ 80% of experiment 0001 obligations check) needs the elaborator and kernel, so it is checked again before phase 2 exits |
  | 0009 annotations | agent behavior | pending: ADR-0006 S6 |
  | 0010 trusted `cgen` | trusted base | pending: CompCert, gcc and clang accept S3's output and agree on the slice corpus |
  | 0011 v1 scope | none (scope) | — |

- **Trusted base:** none; this is a process rule that protects it.
- **Agent loop:** none.
- Docs to update on acceptance: `docs/adr/README.md` (workflow step 2), `docs/adr/0000-template.md` (Evidence gate line), `CLAUDE.md` (Workflow item 2), `docs/experiments/README.md` (experiments may be named by an ADR before they run).

## Validation

- No evidence-gated ADR accepted after this one is superseded within a phase because its own validation criterion failed.
- Revisit if the gate blocks decisions for weeks because experiments cannot be run (missing tools), which would point to tooling, not to the rule.
