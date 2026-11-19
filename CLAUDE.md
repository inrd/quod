# CLAUDE.md — working in the Quod repository

Quod is an experimental, formally verified systems language (`quodc`, `.qd`). Phase 0 (design scaffold) is complete; **phase 1 (minimal core) is in progress**: `quodc` has a lexer, LL(1) parser and canonical formatter; type checker and C emitter are not written yet.

## Accepted decisions

| ADR | Decision |
|---|---|
| [0001](docs/adr/0001-implementation-language.md) | Rust for compiler, oracle and kernel; kernel is an isolated crate with strict constraints |
| [0002](docs/adr/0002-arithmetic-semantics.md) | Mathematical integers in spec contexts; checked machine integers in code; runtime traps during phase 1 only |
| [0003](docs/adr/0003-kernel-logic-and-proof-format.md) | Many-sorted first-order kernel logic; flat proofs in a strict Alethe subset plus Quod rules |

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
| `spec/format.md` | Canonical format emitted by `quodc fmt` (provisional) |
| `spec/diagnostics.md` | Diagnostic code registry (provisional) |
| `Cargo.toml` | Workspace root; members: `compiler` |
| `examples/*.qd` | Provisional syntax sketches |
| `compiler/` | `quodc` crate: lexer, parser, AST dump, formatter, CLI |
| `kernel/`, `oracle/` | Empty skeletons (phases 2, 3), each with a README stating its future scope |
| `tests/` | `.qd` test corpus; `parse/` is run by `compiler/tests/corpus.rs` |

## Workflow: docs first, decisions as ADRs

1. **Docs before code.** Any new feature or component starts as a change to the relevant doc (or a new section in `spec/`). Code follows only once the design is written down.
2. **Decisions are ADRs.** A design decision (anything that closes an item in `docs/open-questions.md`, changes a principle, or picks a technology) is recorded as an ADR:
   - Copy `docs/adr/0000-template.md` to `docs/adr/NNNN-short-title.md` with the next free number.
   - Status starts as `Proposed`. Only a human maintainer moves it to `Accepted` or `Rejected`.
   - When accepted, update `docs/open-questions.md` (mark the question resolved and link the ADR) and any doc it affects.
   - ADRs are never edited after acceptance except to set `Superseded by NNNN`.
3. **Open questions stay open until a human decides.** Agents may add options, evidence and trade-offs to `docs/open-questions.md` and may draft a `Proposed` ADR, but must not treat a question as settled.
4. **Code follows the roadmap phase.** All implementation code is Rust ([ADR-0001](docs/adr/0001-implementation-language.md)). Only write code for the current roadmap phase (now: phase 1), and only for designs already written down in `docs/` or `spec/`.

## Rust and kernel rules (ADR-0001)

- Before finishing a change: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- New diagnostics get a new code in `spec/diagnostics.md` and `compiler/src/diag.rs`; never reuse or renumber a code.

- One Cargo workspace at the repo root; crates live in `compiler/`, `kernel/`, `oracle/`.
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
