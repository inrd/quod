# Architecture

> **Status:** draft. Component boundaries are proposals. Accepted so far: all components in Rust, kernel as an isolated crate ([ADR-0001](adr/0001-implementation-language.md)); kernel logic and proof format ([ADR-0003](adr/0003-kernel-logic-and-proof-format.md)).

## Pipeline

```
                 human                                agent
                   │ reviews                             │ writes
                   ▼                                     ▼
          ┌─────────────────┐                  ┌─────────────────────┐
          │   SPEC LAYER    │                  │  IMPL + PROOFS      │
          │ types, where,   │                  │ fn bodies, loop     │
          │ requires/ensures│                  │ invariants, ghost   │
          │ spec fn, effects│                  │ code, proof terms,  │
          │ resource bounds │                  │ holes ?h            │
          └────────┬────────┘                  └──────────┬──────────┘
                   │                                      │
                   └──────────────┬───────────────────────┘
                                  ▼
                 ┌────────────────────────────────┐
                 │ FRONT END (untrusted)          │
                 │ lex/parse → canonical format   │
                 │ name resolution, type check    │
                 │ effect check, ownership check  │
                 └───────────────┬────────────────┘
                                 ▼
                 ┌────────────────────────────────┐
                 │ CORE IR (untrusted producer)   │
                 │ small explicit language,       │
                 │ every op annotated             │
                 └───────┬───────────────┬────────┘
                         │               │
                         ▼               ▼
        ┌──────────────────────┐   ┌──────────────────────────┐
        │ OBLIGATION GENERATOR │   │ ORACLE SERVER (untrusted)│◄──► agent
        │ VCs with stable IDs  │   │ holes, goals, counter-   │     (JSON)
        └──────────┬───────────┘   │ examples, task list      │
                   │               └──────────────────────────┘
                   ▼                        ▲ uses
        ┌──────────────────────┐   ┌────────┴─────────┐
        │  KERNEL (TRUSTED)    │   │ SMT + elaborator │
        │  checks Alethe-subset│◄──│ (untrusted,      │
        │  proofs against      │   │  agent-side)     │
        │  obligations         │   └──────────────────┘
        │  deterministic       │
        └──────────┬───────────┘
                   │ all obligations discharged (or listed as trust)
                   ▼
        ┌──────────────────────┐   ┌──────────────────────────┐
        │ OPTIMIZER (untrusted)│──►│ C EMITTER (untrusted)    │──► .c ──► cc
        │ uses proven facts to │   │ no UB in output; checks  │
        │ drop runtime checks  │   │ elided where proven      │
        └──────────────────────┘   └──────────────────────────┘
                   │
                   ▼
        ┌──────────────────────────────────────────────┐
        │ REPORTS: human report, trust report,         │
        │ spec warnings (weakness, vacuity, witnesses) │
        └──────────────────────────────────────────────┘
```

## Components

| Component | Trusted? | Responsibility |
|---|---|---|
| Front end | no | Parse, canonicalize, resolve, type/effect/ownership check. Emits Core IR. |
| Core IR | — | Small explicit language: no sugar, every operation's precondition explicit. Shared input to obligation generator, optimizer and emitter. |
| Obligation generator | see below | Produces proof obligations (VCs) from Core IR, each with a stable ID. |
| Kernel | **yes** | Checks refutational proofs (flat Alethe-subset steps) against obligations, building hypotheses and negated goal from the obligation itself. Many-sorted first-order logic with an enumerated axiom list ([ADR-0003](adr/0003-kernel-logic-and-proof-format.md)). Isolated, dependency-free Rust crate, ≤ 5,000 lines ([ADR-0001](adr/0001-implementation-language.md)). |
| Oracle server | no | Answers agent queries over JSON (holes, goals, counterexamples, task list). |
| SMT solvers | no | Agent-side search helpers (e.g. cvc5 producing Alethe certificates). |
| Proof elaborator | no | Turns coarse solver certificates into the kernel's accepted rule subset (Carcara-style). Bugs here cannot affect soundness. |
| Optimizer | no | Uses proven facts to remove checks, then transforms code. |
| C emitter | no | Emits UB-free C. Phase-1 backend. |
| Report generators | no | Human report, trust report, spec warnings. Errors here mislead humans but don't break soundness. |

## Trust boundaries

Soundness claim (target): *if the kernel accepts all obligations, every execution of the emitted program satisfies the spec, modulo the trust report and the C compiler.*

What this requires beyond the kernel:

1. **Obligation generation is faithful.** If the VC generator is buggy, the kernel may check the wrong statements. Options (open):
   - keep the generator tiny and include it in the trusted base;
   - have the kernel itself compute obligations from Core IR (moves Core IR semantics into the kernel);
   - certify each generated VC against a formal semantics of Core IR.
2. **Compilation preserves semantics.** Front end → Core IR → C. Options (open): translation validation, a verified emitter (CompCert-style, long term), or trusting the emitter and listing it in the trust report.
3. **The C compiler and target.** Trusted and listed in the trust report. A future direct backend may change this ([open-questions.md](open-questions.md#9-backend-strategy)).

> Every trusted item above must appear in the trust report, not only user-written `trust` axioms.

## Data artifacts

| Artifact | Producer | Consumer |
|---|---|---|
| `.qd` source (canonical) | agent / human | front end |
| proofs (Alethe-subset text, keyed by obligation ID + fingerprint) | agent / elaborator | kernel |
| obligation list (JSON) | obligation generator | oracle, agent |
| human report (per module) | report generator | human |
| trust report (per build) | report generator | human, CI |
| `.c` output | C emitter | C compiler |

## Determinism

Every stage must be deterministic given the same inputs and compiler version: same obligations, same IDs, same proof-check verdict, same C output. Reproducibility is a correctness property, not a convenience.
