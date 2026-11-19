# Power of 10 → Quod mechanisms

Gerard J. Holzmann, *"The Power of 10: Rules for Developing Safety-Critical Code"* (JPL, IEEE Computer 2006). In C these rules are enforced by style guides and static analyzers. In Quod each becomes a language mechanism checked by the compiler or kernel.

Rule texts below are paraphrased. Syntax is **provisional**.

| # | Rule (paraphrased) | Quod mechanism | Enforced by |
|---|---|---|---|
| 1 | Simple control flow: no `goto`, `setjmp`/`longjmp`, or direct/indirect recursion. | No `goto`, no non-local jumps in the grammar. Recursion is **allowed but must carry a `decreases` measure**, and recursion depth must be bounded for `stack` contracts. Structured control flow only. | grammar, kernel |
| 2 | All loops have a fixed upper bound. | Every loop has a `decreases` measure (termination) and a statically known or contracted bound. Unbounded event loops require an explicit `diverges` construct/effect (speculative). | kernel |
| 3 | No dynamic allocation after initialization. | `alloc` **effect** plus `noalloc` contract. A phase can be marked so only init code holds the `alloc` capability; steady-state functions declare `effects {}`/`noalloc`. Regions make allocation patterns explicit. | effect checker |
| 4 | Functions fit on one page (~60 lines). | **Adapted.** Agents don't need short functions for readability; human review targets specs, not bodies. Instead: the *spec* of each function must be short, and the oracle reports obligation count per function as a complexity signal. Optional lint for body length. | lint (advisory) |
| 5 | At least two assertions per function on average. | Assertions become **proven contracts**: `requires`, `ensures`, invariants, `assert` (proven, not run). Redundant facts are encouraged as hallucination checks. A lint can require non-trivial `ensures` on public functions. Spec-weakness tooling flags contracts that say too little. | kernel, spec tooling |
| 6 | Declare data at the smallest possible scope. | Block-scoped `let` only; no global mutable state without an explicit capability; ownership rules prevent escaping references. | type/ownership checker |
| 7 | Check the return value of every non-void function; validate parameters. | Must-use `Result` (discarding is a compile error; explicit `discard` keyword, speculative). Parameter validation becomes `requires`, proven at call sites. | type checker, kernel |
| 8 | Limit the preprocessor to includes and simple macros. | **No textual macros, no preprocessor.** Modules replace includes; compile-time evaluation and generics replace macros ([open question](open-questions.md#7-generics-and-compile-time-evaluation)). | grammar |
| 9 | Restrict pointers: at most one level of dereference, no function pointers. | References with explicit permissions; raw pointers only under `unsafe` with `trust` contracts. Function values must carry their contract and effects in their type (speculative); otherwise disallowed. Multi-level indirection allowed only when permissions are explicit. | type/permission checker |
| 10 | Compile with all warnings on; zero warnings; use static analyzers daily. | No warnings: every diagnostic is either an error or an entry in the trust report / human report. Verification is not a separate analyzer: it *is* compilation. Trust report diffs gate CI. | compiler, CI |

## Beyond Power of 10

Mechanisms Quod adds that the rules don't cover:

- **No undefined behavior** (overflow, shifts, casts, aliasing) by construction.
- **Resource contracts** (`cost`, `stack`, `mem`) proven, not measured.
- **Trust report** making every unverified assumption visible.
- **Refinement proofs** making optimization safe.

## Related standards

MISRA C and the JPL Institutional Coding Standard for C extend these rules. A fuller mapping of MISRA C:2023 rules is deferred; most of its "decidable" rules are subsumed by the grammar and type system, and most "undecidable" rules by proof obligations. See [prior-art.md](prior-art.md).
