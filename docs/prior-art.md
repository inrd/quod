# Prior art

What Quod takes from, and avoids in, each system. Brief by design; see the systems' own papers for detail.

| System | Take | Avoid |
|---|---|---|
| **SPARK/Ada** | Contracts as part of the language; absence-of-runtime-errors proofs; flow analysis; track record in certified industrial code; `Global`/`Depends` as effect-like annotations. | Ada's large surface and verbosity *in specs*; reliance on SMT with timeouts (GNATprove) for soundness-relevant verdicts. |
| **Dafny** | Clean contract syntax (`requires`, `ensures`, `old`, `decreases`); ghost code; good error locality; lemma style. | Unstable proofs ("brittleness") from SMT heuristics and timeouts; GC-based runtime; large trusted base (Boogie + Z3). |
| **F\*** | Effect system with user-defined effects; refinement types; Low\* subset that extracts to C (HACL\*). Proof that verified C crypto is practical. | Complex type system with steep learning curve; SMT in the trusted base; effect inference that hides information. |
| **Verus** | Verified Rust with linear ghost permissions; `spec`/`proof`/`exec` modes (≈ Quod's strata); fast SMT encoding. | Inheriting Rust's complexity; SMT trusted; proofs not independently checkable. |
| **Lean 4** | **Kernel design**: small trusted kernel, independent re-checkers, proof terms as artifacts; metaprogramming via tactics instead of macros; typed holes / goal view UX. | Dependent type theory may be heavier than Quod's VCs need; toolchain churn. |
| **ATS** | Linear types + theorem proving combined with C-level performance; proofs erased at compile time. | Notoriously hard syntax; poor ergonomics; low adoption. Lesson: surface syntax matters. |
| **Austral** | Small, deliberately simple linear-type language; spec fits in one document; capability-based security; no implicit anything. | Lack of verification beyond linearity. Take its "small enough to fit in one head" discipline. |
| **Zig** | No hidden control flow or allocation; explicit allocators passed as parameters; C interop; `comptime`. | `comptime` as unverified code generation; illegal behavior still exists in release-fast builds. |
| **Koka** | Row-polymorphic algebraic effects; effect types inferred and displayed; Perceus reference counting. | Effect handlers' runtime cost and complexity for a no-runtime language; Quod needs effects as *permissions*, not control operators (at least initially). |
| **Viper / Prusti** | Permission-based separation logic (`acc(x.f)`), fractional permissions, frame reasoning; intermediate verification language design. | SMT/symbolic-execution backends in the trusted base; Prusti's dependence on Rust compiler internals. |
| **Fiat Cryptography** | **Proven-equivalent optimization**: generate fast code from a reference spec with proofs; deployed in BoringSSL/Chrome. Direct model for `refines`. | Coq-heavy pipeline that only experts can extend. |
| **CompCert** | Verified compilation; precise C semantics; clear statement of what is trusted (parser, assembler). Benchmark for "trusted base honesty". | Its scale of effort as a phase-1 target. |
| **RAML** | Automatic amortized resource analysis; polynomial cost bounds inferred and checked. Model for `cost` contracts. | Inference-heavy; Quod prefers declared bounds checked by proof. |
| **Agda / Idris** | **Typed holes**: write `?h`, ask for goal and context, refine interactively. Idris's elaborator reflection. Direct model for the oracle's hole protocol. | Total dependent types as the main programming model; performance. |
| **Unison** | **Content-addressed code**: definitions identified by hash, renames free, no dependency conflicts, perfect incremental compilation. | Departure from text breaks ordinary tooling; see [open-questions.md](open-questions.md#5-source-representation). |
| **MISRA C / JPL C standard** | Accumulated knowledge of what goes wrong in safety-critical C; Power of 10. See [power-of-10-mapping.md](power-of-10-mapping.md). | Enforcement by style guide and partial static analysis; Quod enforces by construction and proof. |

## Cross-cutting lessons

1. **Proof stability is a first-class requirement.** SMT-dependent verifiers suffer from proofs breaking after unrelated changes. Explicit proof artifacts + deterministic kernel address this at the cost of larger proofs (cheap for agents).
2. **Modes/strata work.** Verus (`spec`/`proof`/`exec`), Dafny (ghost), SPARK (ghost code) all converge on separating specification from execution.
3. **C extraction is viable** for verified low-level code (Low\*/HACL\*, Fiat, CompCert).
4. **Syntax and ergonomics decide adoption** (ATS vs. Dafny). For Quod, the "users" of implementation syntax are agents, but humans read specs, so spec syntax must stay clean.
5. **Trusted-base honesty** (CompCert's explicit list) is a model for the trust report.
