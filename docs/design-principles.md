# Design principles

Each principle states the rule, why it exists, and its consequences. Syntax in this document is **provisional and non-normative**.

---

## 1. Two strata: spec and impl + proofs

**Rule.** Every program has two layers:

| Layer | Contents | Author | Reader | Size |
|---|---|---|---|---|
| **Spec** | types, type invariants, contracts, `spec fn` reference definitions, effects, resource bounds | human + agent | human | small |
| **Impl + proofs** | `fn`/`impl fn` bodies, loop invariants, ghost code, proof terms | agent | kernel | large, verbose |

**Why.** Human attention is the scarcest resource. It should be spent where intent lives (the spec), not on implementation details a machine can check.

**Consequences.**
- The spec layer must stay small and readable; it is the input to the human report.
- A change that touches only the impl layer and still checks needs no human review. A change to the spec layer always does.
- Tooling can diff the spec layer separately (`quodc spec-diff`, speculative).

---

## 2. Small trusted kernel (de Bruijn criterion)

**Rule.** The only trusted component is a minimal, deterministic **proof checker** (the kernel). The parser front end, type checker, verification-condition generator, optimizer, C emitter, SMT solvers and agents are all untrusted.

**Why.** If a large, heuristic component is trusted, its bugs become soundness bugs. A small kernel can be audited, and in the long run verified itself.

**Consequences.**
- Proofs are **explicit artifacts** stored alongside code, written by agents, checked by the kernel.
- SMT solvers are *search helpers* for agents. Their certificates are elaborated, outside the kernel, into flat proofs using the kernel's accepted rules ([ADR-0003](adr/0003-kernel-logic-and-proof-format.md)).
- **No timeouts.** Checking is deterministic and reproducible; a proof either checks or it doesn't, on every machine.
- The path from checked obligations to emitted code must itself be trustworthy: either the obligation generator is in the trusted base (and kept small) or its output is validated. See [architecture.md](architecture.md#trust-boundaries).

---

## 3. Zero undefined behavior

**Rule.** Every operation is either **total** (defined for all inputs) or carries a **precondition** that must be proven at every use.

| Operation | Treatment |
|---|---|
| `a + b` on `u32` | precondition: no overflow. Wrapping/saturating/checked variants are distinct, explicitly named operations (`add_wrap`, `add_sat`, `add_checked`). |
| `a / b` | precondition: `b != 0` (and for signed: not `MIN / -1`). |
| `xs[i]` | precondition: `i < xs.len`. |
| `*p` | precondition: permission to read `p` (memory model). |
| shift `a << n` | precondition: `n < bit_width`. |
| cast `u64 as u32` | precondition: value fits; truncation is a separate operation. |

In spec contexts (`requires`, `ensures`, `invariant`, `where`, `spec fn`, …), arithmetic on machine integers means exact mathematical arithmetic on `int`, with no overflow; bitwise operators keep their bit-vector meaning ([ADR-0002](adr/0002-arithmetic-semantics.md)).

**Consequences.**
- Proven preconditions compile to nothing. Unproven ones are compile errors, not runtime checks (from phase 2; during phase 1 they compile as runtime traps listed in a build report).
- An explicit runtime check is always available (e.g. `if i < xs.len { … }`), and the branch condition becomes a proven fact.

---

## 4. Contracts and invariants

**Rule.** Behavior is specified with:

- `requires P` — precondition, proven by every caller.
- `ensures Q` — postcondition, proven by the body; may use `result` and `old(e)`.
- **Type invariants** — `where` clauses on fields or types, established on construction and preserved by every mutation.
- **Loop invariants** — `invariant I` on every loop.
- **Termination measures** — `decreases e` on every loop and every recursive function.

**Why.** Contracts are the unit of compositional verification (principle 10) and the raw material of the human report.

**Consequences.** Every loop and every recursion has a termination argument. Non-terminating code (event loops, servers) must be expressed through an explicit construct with its own effect (speculative: `diverges`).

---

## 5. Effects and capabilities in signatures

**Rule.** Every function declares its effects. A function may only call functions whose effects are a subset of its own.

| Effect | Meaning |
|---|---|
| `io` | performs input/output |
| `alloc` | may allocate memory |
| `panic` | may abort (only in explicitly allowed contexts) |
| `blocking` | may block the thread |
| `unsafe` | uses `trust` axioms (FFI, hardware) |
| `time` | reads clocks; output may depend on time |

`effects {}` means pure and total (modulo explicit memory permissions).

**Why.** Effects are the most important information for reviewing a change at a glance: "this function now allocates" or "this now does I/O" is a first-class review signal.

**Consequences.**
- Effect changes appear in the human report and in spec diffs.
- Capabilities are values (speculative): a function doing `io` takes a capability parameter, so authority is explicit and can be attenuated.
- Effect polymorphism (for higher-order functions) is needed but its design is open.

---

## 6. Refinement workflow

**Rule.** A behavior can be given as a simple reference implementation (`spec fn`) plus one or more optimized implementations (`impl fn f refines g`) that are **proven equivalent** to it.

```c
spec fn sum(xs: &[i64]) -> i128 = fold(xs, 0, add)

impl fn sum_fast(xs: &[i64]) -> i128
    refines sum
    effects {}
{ ... SIMD, unrolled ... }
```

**Why.** Separates *what* from *how fast*. The human reviews `sum`; agents may rewrite `sum_fast` aggressively without any correctness risk.

**Consequences.** `spec fn` are pure, may be non-executable (e.g. use mathematical integers or quantifiers) and generate no code unless called from executable code.

---

## 7. Resource contracts

**Rule.** Functions may declare and must then prove resource bounds:

| Contract | Meaning |
|---|---|
| `cost <= e` | abstract step count bounded by `e` (unit model: open) |
| `stack <= n` | maximum stack bytes, including callees |
| `noalloc` | no heap allocation (equivalent to the absence of `alloc` in effects, kept as explicit redundancy) |
| `mem <= e` | bound on allocated bytes |

**Why.** Real-time and embedded code needs these guarantees; today they come from external tools and testing.

**Consequences.** Bounded loops and well-founded recursion with explicit measures make stack depth and cost computable. Recursion depth must be bounded by a measure for `stack` to be provable.

> **Speculative:** the `cost` model (abstract steps vs. target-specific cycles) is undecided. Abstract steps are portable but weak; cycles need a trusted target model.

---

## 8. Explicit memory model

**Rule.** Memory safety comes from **ownership plus explicit permissions** in the style of separation logic (Viper, Prusti):

- Each memory location has an owner; references carry read or write permission.
- Permissions are explicit in contracts (`requires acc(p.field)`, speculative notation) when not derivable from types.
- **Regions/arenas** are the primary allocation model: allocate into a region, free the region as a whole. Individual free is a library concern.
- **FFI and hardware access** go through capability effects (`unsafe`, `io`) whose contracts are explicit `trust` axioms.

**Why.** Separation-logic permissions give frame reasoning (what a call doesn't touch stays unchanged) for free, which compositional verification needs.

---

## 9. Graded trust and the trust report

**Rule.** Every unproven fact is introduced by `trust` (an axiom about external code) or `assume` (a local unproven fact), and each carries **provenance**: who introduced it, why, and a reference.

```c
trust fn memcpy(dst: *mut u8, src: *const u8, n: usize)
    effects { unsafe }
    requires acc_w(dst, n) && acc_r(src, n) && disjoint(dst, src, n)
    ensures  bytes(dst, n) == old(bytes(src, n))
    provenance "libc memcpy, C11 §7.24.2.1"
```

**Consequences.** Every build produces a **trust report** listing every unproven assumption, transitively. A release build may be configured to reject `assume` entirely, or to require approval for each `trust` change.

---

## 10. Compositional verification

**Rule.** A function is checked using only its own body, its own contract and its callees' **contracts** (never callee bodies). Modules are checked against their dependencies' interfaces.

**Why.**
- Bounds the context an agent needs: to work on `f`, it reads `f` and the signatures of what `f` calls.
- Enables parallelism: **the proof decomposition is the work breakdown**. Each obligation is an independent task for an agent.
- Makes checking incremental: changing a body re-checks only that body; changing a contract re-checks callers.

**Consequences.** Contracts must be strong enough for callers. Too-weak contracts surface as unprovable caller obligations, which the oracle reports.

---

## 11. Oracle server

**Rule.** The compiler exposes a structured, machine-readable query protocol (JSON) designed for repair loops. See [oracle-protocol.md](oracle-protocol.md).

Capabilities:
- **Typed holes** (`?h1`): the agent leaves a hole; the oracle reports its expected type, context and goals.
- **Goals and context** at any program point.
- **Counterexamples** with concrete values and minimal traces.
- **Weakest-precondition suggestions** for failing calls.
- **Proof obligations as a task list** with stable IDs that survive unrelated edits.

**Why.** Agents perform best with precise, structured, actionable feedback. Free-text error messages waste context and invite misreading.

---

## 12. Agent-friendly surface

**Rule.**
- Minimal **C-like syntax**, close to LLM training data.
- A small, **LL(1)-style grammar** suited to grammar-constrained decoding (the sampler can be restricted to syntactically valid tokens).
- **One canonical format**; the formatter is part of the compiler and non-canonical input is rejected or normalized.
- **No overloading, no implicit conversions, no textual macros.** Every name resolves to exactly one thing; every conversion is written out.

**Why.** Ambiguity is where agents hallucinate. Each removed implicit rule is one fewer thing an agent can get subtly wrong.

---

## 13. Spec-quality tooling

**Rule.** The toolchain attacks the prompt → spec gap directly:

- **Vacuity and weakness detection:** generate implementation mutants; if a mutant still satisfies the spec, the spec is too weak and the mutant is shown as evidence.
- **Surprising behaviors:** generate concrete inputs/outputs the spec allows that a human may not expect (e.g. "`sort` may return an empty list" if permutation is missing).
- **Plain-language rendering:** render contracts as claims, tables and examples.

The per-module **human report** — contracts, examples, effects, resource bounds, trust assumptions, spec warnings — is the language's real human UI.

> **Speculative:** how far plain-language rendering can go without becoming misleading is open. See [open-questions.md](open-questions.md#4-spec-language-expressiveness).

---

## 14. Safety-critical heritage

**Rule.** Each of Holzmann's "Power of 10" rules (JPL) maps to a language-level mechanism enforced by the checker, not by a style guide. See [power-of-10-mapping.md](power-of-10-mapping.md).
