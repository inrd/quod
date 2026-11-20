# Architecture decision records

Design decisions for Quod are recorded here, one file per decision: `NNNN-short-title.md`, numbered sequentially.

1. Copy [0000-template.md](0000-template.md) to the next free number.
2. Status starts as **Proposed**. Agents may draft ADRs; only a human maintainer sets **Accepted** or **Rejected**.
3. On acceptance, update [../open-questions.md](../open-questions.md) and any affected doc.
4. Accepted ADRs are immutable; a later ADR can supersede one, and the old one's status is set to `Superseded by ADR-NNNN`.

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-implementation-language.md) | Rust for the bootstrap compiler and the kernel | Accepted |
| [0002](0002-arithmetic-semantics.md) | Mathematical integers in specs, checked machine integers in code | Accepted |
| [0003](0003-kernel-logic-and-proof-format.md) | First-order kernel logic with flat, Alethe-based proof steps | Accepted |
| [0004](0004-phase-1-typing-choices.md) | Phase-1 typing choices | Accepted (decision 1 partly superseded by 0005) |
| [0005](0005-portable-static-checks-for-pointer-sized-integers.md) | Portable static checks for pointer-sized integers | Accepted |
| [0006](0006-thin-end-to-end-slice.md) | Build a thin end-to-end slice before widening phases 1–3 | Proposed |
| [0007](0007-trusted-obligation-generator.md) | A small trusted obligation generator over a specified Core IR | Proposed |
| [0008](0008-kernel-rules-from-real-certificates.md) | Kernel rules fitted to real solver certificates (supersedes 0003) | Proposed |
| [0009](0009-annotations-not-proofs.md) | Agents write annotations; solvers produce certificates; the kernel checks them | Proposed |
| [0010](0010-trusted-c-emitter-from-core-ir.md) | A small trusted C emitter over Core IR, and no compiler optimizer | Proposed |
| [0011](0011-v1-scope.md) | v1 scope — value-level memory, no general heap, `refines` as a contract, resource contracts deferred | Proposed |
| [0012](0012-evidence-before-acceptance.md) | Trusted-base and behavior ADRs stay Proposed until their experiment has run | Proposed |
