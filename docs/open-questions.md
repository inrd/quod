# Open questions

Each question lists options and trade-offs. **None is decided.** A question is closed only by an accepted ADR; when that happens, mark it `Resolved by ADR-NNNN` and keep the analysis for history.

---

## 1. Implementation language

> **Resolved by [ADR-0001](adr/0001-implementation-language.md):** Rust for compiler and kernel (kernel as an isolated crate); a Lean re-checker later. Analysis kept below for history.

Covers the bootstrap compiler (front end, obligation generator, oracle, emitter) and the kernel. They may differ.

| Option | For | Against |
|---|---|---|
| **Rust** | Fast, memory-safe, strong ecosystem (parsers, JSON, LSP-style servers), good C interop, agents write it well. Verus/Prusti/Kani available to verify parts. | Complex type system slows quick iteration; kernel verification in Rust is still immature. |
| **OCaml** | The classic compiler language (CompCert extraction, Coq, F\* lineage); ADTs and pattern matching ideal for IR passes; small runtime. Kernel can be extracted from Coq/Rocq. | Smaller ecosystem; agents somewhat less fluent; GC (acceptable for a compiler). |
| **Lean 4** | Kernel and compiler in one language that can *prove its own kernel properties*; compiles to C; strong metaprogramming. Makes a verified kernel a realistic goal. | Smaller ecosystem for servers/JSON tooling; toolchain churn; build times; fewer engineers. |

Mixed options: Rust or OCaml for the compiler + a separate tiny kernel (in the same language or in Lean/Rocq with extraction). Self-hosting is a long-term option once Quod is usable.

## 2. Kernel logic and proof term format

> **Resolved by [ADR-0003](adr/0003-kernel-logic-and-proof-format.md):** many-sorted first-order logic; flat proofs in a strict Alethe subset plus Quod rules. Analysis kept below for history.

| Option | For | Against |
|---|---|---|
| **Type-theoretic** (CIC/Lean-like dependent type theory) | Mature design, proven small kernels (Lean, Rocq), expressive. | Heavyweight for first-order program VCs; proof terms are large; agents must learn a proof language. |
| **First-order logic + theories with explicit proof steps** (resolution, rewriting, arithmetic lemmas) | Matches VC shape; close to SMT output; small checker. | Need trusted theory axioms for bit-vectors, arrays; less expressive for higher-order specs. |
| **LCF-style** (abstract theorem type, small set of inference rules; HOL Light) | Tiny kernel; proven track record. | Proofs are programs, not data (unless logged); replay needed. |

Certificate formats to reuse: **Alethe** (cvc5, veriT), **LFSC** (cvc5), **DRAT/LRAT** (SAT), Z3 proof logs. Reusing Alethe would give an agent-side SMT → kernel path early. Trade-off: the kernel must check the certificate's rules, which grows the trusted base.

## 3. Arithmetic semantics in specs

> **Resolved by [ADR-0002](adr/0002-arithmetic-semantics.md):** mathematical integers in spec contexts, checked machine integers in code. Analysis kept below for history.

| Option | For | Against |
|---|---|---|
| **Mathematical integers in specs, bit-vectors in code** (SPARK, Dafny, Verus style) | Specs read naturally (`a + b` means the real sum); overflow obligations are simple `0 <= a+b <= MAX`. | Two arithmetics; every code operation needs a mapping lemma; risk of confusion. |
| **Bit-vectors everywhere** | One semantics; exact. | Specs become hard to read and reason about; nonlinear BV reasoning is hard. |
| **Explicit choice per expression** (e.g. `int(a) + int(b)` vs. `a +% b`) | Fully explicit, matching the "nothing implicit" principle. | Verbose specs; humans review specs, so verbosity there has a cost. |

Interaction questions: casting between the two, `old()` of bounded values, division/modulo sign conventions.

## 4. Spec language expressiveness

How expressive can specs be while keeping plain-language rendering and weakness checks reliable?

| Option | For | Against |
|---|---|---|
| **Restricted** (quantifier-free + bounded quantifiers, no higher-order) | Renders well; mutation and witness generation decidable/efficient. | Can't specify many real properties (sortedness via permutation, algebraic laws). |
| **Full higher-order** | Expressive. | Rendering degrades to formula pretty-printing; weakness checks become heuristic. |
| **Tiered** (restricted core + opt-in advanced constructs flagged in reports) | Gets most of both; reports show which parts are "hard to review". | More language surface; tier boundaries need design. |

## 5. Source representation

| Option | For | Against |
|---|---|---|
| **Canonical text** | Works with every tool, git, LLM training data. | Agents produce text edits that may break structure; diffs are noisy. |
| **Content-addressed AST store** (Unison-like) with typed edit operations | Stable identities, no merge conflicts on renames, edits always well-formed, perfect incremental checking. | Breaks standard tooling; agents trained on text; needs a full store and UI. |
| **Both** (text is a projection of the store; edits can come either way) | Best of both. | Two sources of truth to keep in sync; complexity. |

## 6. Concurrency model

| Option | For | Against |
|---|---|---|
| **Deterministic by default** (fork-join, no shared mutable state) | Verification stays sequential-like; reproducible. | Excludes some systems patterns (lock-free structures). |
| **Structured tasks** with explicit permissions transfer | Separation logic handles ownership transfer naturally. | Needs concurrent separation logic in the kernel's reach. |
| **Message passing** (channels, actors) | Simple local reasoning. | Protocol properties (deadlock freedom) need session types or similar. |
| **Defer entirely** (single-threaded until phase N) | Keeps scope small. | Retrofits are painful; memory model choice constrains later options. |

## 7. Generics and compile-time evaluation

No textual macros. Options:

| Option | For | Against |
|---|---|---|
| **Monomorphization only, generics checked once** (verify the generic body once against bounded type parameters) | Zero-cost; proofs reused across instantiations. | Requires parametric reasoning in the kernel; trait-like bounds needed. |
| **Monomorphize then verify each instance** | Simple kernel. | Proof blow-up; slow. |
| **Zig-style `comptime`** (compile-time evaluation of ordinary code) | Powerful, one language. | Hard to verify generators; the program that produces code must also be trusted or verified. |
| **Const generics + restricted compile-time functions** (pure, total, terminating) | Covers array sizes and tables. | Less powerful than `comptime`. |

## 8. Data layout control

Questions: SoA/AoS annotations, alignment, packing, explicit padding, SIMD vector types (`f32x8`), bit fields.

| Option | For | Against |
|---|---|---|
| **C layout by default + explicit attributes** | Predictable; FFI-friendly. | Leaves performance on the table unless agents annotate. |
| **Layout as a refinement** (logical type + proven-equivalent physical layout) | Fits the refinement principle; agents may reshape data freely. | Research-level; equivalence of data representations is harder than of functions. |
| **First-class SIMD types with total operations** | Portable vector code. | Target-specific width choices. |

## 9. Backend strategy

| Option | For | Against |
|---|---|---|
| **C emission only** | Portable, simple, debuggable. | C compiler is trusted; UB-free C must be emitted carefully; limited control (e.g. tail calls, SIMD). |
| **LLVM IR** | Performance, wide target support. | Huge trusted base; LLVM has its own UB semantics (`poison`, `undef`). |
| **MLIR** | Good fit for high-level → low-level lowering and SIMD. | Even larger trusted base; moving target. |
| **Direct (own backend, possibly verified à la CompCert)** | Smallest trust for codegen in the long run. | Enormous effort; poor performance initially. |

## 10. Mitigating the lack of training data

| Option | For | Against |
|---|---|---|
| **Grammar-constrained decoding** (grammar given to the sampler) | Guarantees syntactic validity. | Only syntax; requires sampler access. |
| **In-context language reference** (compact reference shipped with every task) | Works with any model. | Consumes context. |
| **Few-shot corpus** of verified examples | Teaches idioms and proof patterns. | Must be maintained as syntax evolves. |
| **Syntax close to C/Rust** | Leverages existing knowledge. | Risk of "false friends": agents write Rust semantics into Quod. |
| **Fine-tuning** on generated corpora | Strongest effect. | Costly, model-specific. |

These are complementary; the question is priority and budget.

## 11. Human-facing report and spec review in practice

- Format: static HTML, Markdown, or interactive (query-able witnesses)?
- Granularity: per function, per module, per change (diff-oriented)?
- What reviewers sign off on: the spec text, the rendered claims, or the examples?
- How to present weakness warnings without alert fatigue?
- How to measure that the report actually helps (user studies, seeded-bug detection rate)?

> **Speculative:** a diff-oriented report ("this change: + effect `alloc`, contract weakened at `push`, 1 new `trust`") may be more useful than full-module reports.

## 12. Role of the SMT solver

> **Partly settled by [ADR-0003](adr/0003-kernel-logic-and-proof-format.md):** solvers are agent-side only for now; every result is elaborated into kernel-checked steps. Still open: whether a certified, partly trusted fragment is ever worth adding.

| Option | For | Against |
|---|---|---|
| **Agent-side helper only**; every result reconstructed into kernel proofs | Minimal trust; deterministic kernel. | Proof reconstruction is hard for some theories (nonlinear, quantifiers); slower loop. |
| **Partly trusted** for specific decidable fragments (e.g. linear arithmetic) via a certified checker | Fast for common cases. | Grows the trusted base; checker must still be deterministic. |
| **Trusted with recorded results** (cache solver verdicts) | Fastest. | Violates the small-kernel principle; must appear in the trust report. |

A possible compromise: start with option 3 restricted and *visible in the trust report*, and migrate obligations to option 1 as reconstruction matures. That would itself need an ADR.
