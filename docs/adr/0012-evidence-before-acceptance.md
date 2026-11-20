# ADR-0012: Trusted-base and behavior ADRs stay Proposed until their experiment has run

- **Status:** Accepted
- **Date:** 2026-11-20
- **Deciders:** project maintainer (accepted 2026-11-20)
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** none. Amends the ADR workflow in [adr/README.md](README.md) and `CLAUDE.md`.
- **Evidence gate:** none (process decision)
- **Revised:** 2026-11-20 (building to a gated design; gates restated for 0007, 0008, 0011; 0009 split into 0009 and 0013)

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

**Building to a gated design.** Often the experiment *is* building the component, e.g. `cgen` emitting C for the slice corpus. Code that a gated ADR's experiment needs may be written to that ADR's design while it is still `Proposed`. That code:

- follows every repository rule (constraints, budgets, determinism, review); it is not a prototype exempt from them;
- does not make the ADR accepted, and nothing else may treat the design as settled;
- is revised or removed with the ADR if the experiment fails or the ADR is rejected.

No throwaway alternative is built "in case the ADR is rejected". The gate's experiment either confirms the design or the design changes.

**No partial acceptance.** An ADR is accepted only when every gate it names has passed. If some decisions in a draft are ungated, or would be ready long before the others, the draft is split into separate ADRs, so each can be accepted when its own evidence is in. Accepted ADRs stay immutable, so accepting part of one is not used.

## Consequences

- Current Proposed ADRs:

  | ADR | Gate | State |
  |---|---|---|
  | 0006 thin slice | none (process) | — |
  | 0007 trusted `vcgen` | trusted base | pending: S4a, where `vcgen` for the slice is within budget and rejects every generator soundness case in its Validation |
  | 0008 kernel rules | trusted base | §4 and §5 passed (experiments 0002, 0003); §1–§3 pending: experiment 0004 at S4a (elaborated certificates use only §1 rules and §2 schemas). The kernel re-check stays a phase-2 exit criterion, not a gate, because the kernel is built after acceptance |
  | 0009 `prove`/`check` split | none (tool architecture) | — |
  | 0010 trusted `cgen` | trusted base | pending: CompCert, gcc and clang accept S3's output and agree on the slice corpus |
  | 0011 v1 scope | trusted base (frame-size report for `stack`) and agent behavior (index-linked structures) | pending: S3 (frame-size reports) and S6 (pool tasks) |
  | 0013 annotations, small obligations | agent and solver behavior | pending: ADR-0006 S6 |

  None of these needs a circular order. S2b–S4a build to the designs of 0007, 0008 and 0010 under the rule above, and each ADR is accepted when its step's report is in.

- **Trusted base:** none; this is a process rule that protects it.
- **Agent loop:** none.
- Docs to update on acceptance: `docs/adr/README.md` (workflow step 2), `docs/adr/0000-template.md` (Evidence gate line), `CLAUDE.md` (Workflow item 2; "code follows the roadmap phase" gains the building-to-a-gated-design exception), `docs/experiments/README.md` (experiments may be named by an ADR before they run).

## Validation

- No evidence-gated ADR accepted after this one is superseded within a phase because its own validation criterion failed.
- Revisit if the gate blocks decisions for weeks because experiments cannot be run (missing tools), which would point to tooling, not to the rule.
- Revisit if code built to a gated design is often thrown away. That would mean the gates are placed too late in the work.
