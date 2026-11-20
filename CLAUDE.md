# CLAUDE.md — working in the Quod repository

Quod is an experimental, formally verified systems language (`quodc`, `.qd`). Phase 0 (design scaffold) is complete; **phase 1 (minimal core) is in progress**: `quodc` has a lexer, LL(1) parser, canonical formatter and type checker (with the unverified-checks report), including contracts and `while` loops for the thin slice; the C emitter is not written yet.

## Accepted decisions

| ADR | Decision |
|---|---|
| [0001](docs/adr/0001-implementation-language.md) | Rust for compiler, oracle and kernel; kernel is an isolated crate with strict constraints |
| [0002](docs/adr/0002-arithmetic-semantics.md) | Mathematical integers in spec contexts; checked machine integers in code; runtime traps during phase 1 only |
| [0003](docs/adr/0003-kernel-logic-and-proof-format.md) | Many-sorted first-order kernel logic; flat proofs in a strict Alethe subset plus Quod rules |
| [0004](docs/adr/0004-phase-1-typing-choices.md) | Phase-1 typing: pointer-width `usize`, `&p[lo..hi]` slicing, one-level implicit deref, same-type shifts, ordered struct literals, no dead code/recursion/modules |
| [0005](docs/adr/0005-portable-static-checks-for-pointer-sized-integers.md) | Static checks treat `usize`/`isize` conservatively: values may need 64 bits, stored values must fit 32 bits; target-independent |
| [0006](docs/adr/0006-thin-end-to-end-slice.md) | Build a thin end-to-end slice (S1–S7) through phases 1–3 before widening; slice code limited to the slice subset |
| [0009](docs/adr/0009-prove-check-split.md) | `quodc prove` (solver, untrusted, non-deterministic) writes certificates; `quodc check` (kernel + `vcgen`, deterministic, used by CI) checks stored certificates only |
| [0012](docs/adr/0012-evidence-before-acceptance.md) | Trusted-base and behavior ADRs stay Proposed until a named experiment passes; gated designs may be built as the experiment; no partial acceptance |

Read the relevant ADR before working on the area it covers.

## Repository layout

| Path | Contents |
|---|---|
| `README.md` | What Quod is, status, reading order |
| `docs/vision.md` | Why the language exists; core thesis |
| `docs/design-principles.md` | The principles every design must respect |
| `docs/architecture.md` | Pipeline and trust boundaries |
| `docs/power-of-10-mapping.md` | JPL "Power of 10" rules → language mechanisms |
| `docs/oracle-protocol.md` | Draft JSON protocol between agents and the compiler |
| `docs/roadmap.md` | Phases 0–9 with exit criteria |
| `docs/open-questions.md` | Undecided questions, with options and trade-offs |
| `docs/prior-art.md` | What to take or avoid from related systems |
| `docs/adr/` | Architecture decision records (`NNNN-title.md`) |
| `spec/grammar-draft.md` | Provisional EBNF for the phase-1 core |
| `spec/typing.md` | Phase-1 name resolution and type checking (draft) |
| `spec/core-ir.md` | Core IR for the slice: canonical text, operations, obligation generation, lowering (draft; text and checks implemented in `coreir/`) |
| `spec/contracts.md` | Contracts, `while` loops, spec contexts and the thin-slice subset (draft; implemented in the front end, not yet lowered or verified) |
| `spec/format.md` | Canonical format emitted by `quodc fmt` (provisional) |
| `spec/diagnostics.md` | Diagnostic code registry (provisional) |
| `Cargo.toml` | Workspace root; members: `compiler`, `coreir`, `xtask` |
| `examples/*.qd` | Provisional syntax sketches |
| `compiler/` | `quodc` crate: lexer, parser, AST dump, formatter, type checker, CLI |
| `coreir/` | Core IR types, canonical text parser/printer, well-formedness checks ([spec/core-ir.md](spec/core-ir.md)); trusted if ADR-0007 is accepted; kernel-style constraints, ≤ 1,500 lines |
| `kernel/`, `oracle/` | Empty skeletons (phases 2, 3), each with a README stating its future scope |
| `tests/` | Test corpus; `parse/` and `typecheck/` are run by `compiler/tests/`, `coreir/` (`.core` files) by `coreir/tests/` |
| `tools/` | Pinned external tools for experiments (cvc5, Ethos, CPC signature, Carcara): `tools/fetch.sh` installs them into git-ignored `tools/bin/`; `tools/check-cpc.sh` checks one obligation's CPC proof |
| `xtask/` | Repository checks (`cargo xtask lint`): links and anchors, ADR index and trusted-base statements, diagnostics registry, `.qd` headers, trusted-crate budgets, no `HashMap`/`HashSet` in `compiler/`. Tooling, not part of the compiler |
| `.claude/` | Agent setup: permission allowlist, Stop hook (`hooks/check.sh`: fmt check + `cargo xtask lint` when the tree changed), `/accept-adr` skill |

## Workflow: docs first, decisions as ADRs

1. **Docs before code.** Any new feature or component starts as a change to the relevant doc (or a new section in `spec/`). Code follows only once the design is written down.
2. **Decisions are ADRs.** A design decision (anything that closes an item in `docs/open-questions.md`, changes a principle, or picks a technology) is recorded as an ADR:
   - Copy `docs/adr/0000-template.md` to `docs/adr/NNNN-short-title.md` with the next free number.
   - Status starts as `Proposed`. Only a human maintainer moves it to `Accepted` or `Rejected`.
   - **Evidence gate** ([ADR-0012](docs/adr/0012-evidence-before-acceptance.md)): an ADR that changes the trusted base (kernel rules, axioms, sorts; `vcgen` encodings; `cgen` output; Core IR semantics; trust-report toolchain items) or rests on a claim about solver or agent behavior names an experiment with a pass criterion fixed in advance. It stays `Proposed` until the report in `docs/experiments/` shows a pass; on a fail it is revised or rejected. Every ADR header has an `Evidence gate:` line. No partial acceptance: split the ADR instead.
   - When accepted, update `docs/open-questions.md` (mark the question resolved and link the ADR) and any doc it affects. The `/accept-adr NNNN` skill walks through these edits.
   - ADRs are never edited after acceptance except to set `Superseded by NNNN`.
3. **Open questions stay open until a human decides.** Agents may add options, evidence and trade-offs to `docs/open-questions.md` and may draft a `Proposed` ADR, but must not treat a question as settled.
4. **Code follows the roadmap phase.** All implementation code is Rust ([ADR-0001](docs/adr/0001-implementation-language.md)). Only write code for the current roadmap phase (now: phase 1), and only for designs already written down in `docs/` or `spec/`. Two exceptions:
   - **Thin slice** ([ADR-0006](docs/adr/0006-thin-end-to-end-slice.md)): code for its steps S2–S6 (Core IR, `cgen`, `vcgen`, elaborator, kernel, minimal oracle) may be written before phase 1 is complete, **limited to the slice subset**. Slice code follows every repository rule.
   - **Building to a gated design** (ADR-0012): code that a Proposed, evidence-gated ADR's experiment needs may follow that ADR's design. It does not settle the ADR, and it is revised or removed if the ADR is rejected. No fallback alternative is built in parallel.

## Rust and kernel rules (ADR-0001)

- Before finishing a change: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo xtask lint` (the Stop hook runs the first and last automatically).
- New diagnostics get a new code in `spec/diagnostics.md` and `compiler/src/diag.rs`; never reuse or renumber a code.

- One Cargo workspace at the repo root; crates live in `compiler/`, `coreir/`, `kernel/`, `oracle/`.
- `coreir/` follows the kernel's constraints below (no dependencies, `#![forbid(unsafe_code)]`, no hash collections, floats, threads, clocks, randomness, environment or file-system access), with a budget of ≤ 1,500 lines excluding tests; `cargo xtask lint` enforces both.
- **Determinism everywhere:** no `HashMap`/`HashSet` iteration where order can reach output (use `BTreeMap`/`BTreeSet` or `Vec`); outputs, obligation IDs and C code must be byte-identical across runs and machines.
- **The kernel crate** (`kernel/`) is special and must stay:
  - dependency-free (only `core`/`alloc`/`std`; no other workspace crates);
  - `#![forbid(unsafe_code)]`;
  - free of `HashMap`/`HashSet`, floats, threads, clocks, randomness, environment and file-system access during checking;
  - within its line budget (≤ 5,000 lines excluding tests).
- Changing the kernel's accepted rules or axiom list ([ADR-0003](docs/adr/0003-kernel-logic-and-proof-format.md)) requires a new ADR. Prefer putting logic in the untrusted elaborator instead.

## Writing rules

- Keep docs concise and technical. Prefer tables and short lists over prose.
- Mark speculative material explicitly (`> **Speculative:** …`).
- **All syntax is provisional.** Every `.qd` file must start with a comment saying so, and every syntax snippet in docs is non-normative unless it lives in `spec/` and an ADR has accepted it.
- Use the project vocabulary consistently: *spec layer*, *impl layer*, *kernel*, *proof obligation*, *oracle*, *effect*, *capability*, *trust report*, *human report*.
- Keep the trusted computing base small. Any proposal that adds to the trusted base must say so explicitly and justify it.

## Design invariants (do not violate without an ADR)

- No undefined behavior. Every operation is total or carries a precondition that must be proven.
- Only the kernel is trusted. Compiler, optimizer, SMT solvers, the proof elaborator and agents are untrusted.
- Arithmetic follows ADR-0002: exact math in spec contexts, a proven precondition for every code-side operator, named functions for wrapping/saturating behavior.
- Checking is deterministic. No timeouts, no randomness, no solver calls inside the kernel.
- Nothing implicit: no implicit conversions, no overloading, no textual macros, one canonical format.
- Every `trust`/`assume` has provenance and appears in the trust report.
- Verification is compositional: a function is checked against its own contract and its callees' contracts only.

## Git

Do not commit, push or rewrite history unless a maintainer explicitly asks.
