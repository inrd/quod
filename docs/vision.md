# Vision

## Problem

Coding agents now produce code faster than humans can read it. The bottleneck has moved from *writing* to *checking*. Mainstream systems languages make checking worse:

- **C/C++** permit undefined behavior; a plausible-looking agent edit can be silently wrong.
- **Rust** removes memory-safety UB but not logic errors, overflow-as-panic, or unbounded resource use, and its correctness still rests on human review of the implementation.
- **Verification languages** (Dafny, F\*, Verus, SPARK) prove correctness but were designed for expert humans: proof effort is the scarce resource, so they lean on implicit automation (SMT with timeouts, heuristics) that is non-deterministic and opaque to an agent in a repair loop.

## Thesis

1. **Writing is cheap; checking is expensive.** The language is designed around the agent ↔ compiler feedback loop. The compiler is a *verification oracle* that answers structured questions, not just a translator that emits errors.
2. **Humans review specs, machines check proofs.** Trust comes from two separate checks:
   - **spec ↔ intent** — a human reviews a small spec, helped by generated reports, examples and witnesses;
   - **code ↔ spec** — a small, deterministic proof kernel checks explicit proofs.
   Humans should rarely need to read implementation code.
3. **Verbosity is free, so nothing is implicit.** Agents don't tire of typing. Every conversion, effect, bound and permission is written out. Redundant statements (e.g. restating a fact already implied) act as consistency checks that catch hallucinations early.
4. **Correctness and performance go together.** Every proven fact is a runtime check that need not exist: bounds, overflow, null, alignment. Refinement proofs let agents optimize aggressively (SIMD, unrolling, layout changes) with no correctness risk.

## What Quod is

- A **C alternative**: no GC, no runtime, no undefined behavior, predictable layout, C-level performance.
- A **primitive**: higher-level abstractions (collections, async, allocators, protocols) are built as verified libraries, not baked into the language.
- A **two-stratum language**: a small human-auditable *spec* layer above a large, verbose, agent-written *impl + proofs* layer.

## What Quod is not

- Not a language optimized for human ergonomics when writing implementations. Human ergonomics matter for *specs and reports*.
- Not a general-purpose theorem prover. The logic exists to verify programs.
- Not a dynamic or garbage-collected language, and not a scripting language.

## Success criteria

The project's key metric (see [roadmap.md](roadmap.md), phase 9): **how fast an agent drives functions from typed holes to fully verified code** on a fixed benchmark corpus, while the trusted base stays small and every assumption is visible in the trust report.

Secondary criteria:

- A human can decide whether a module does what they intended by reading its generated human report, without reading implementation code.
- Generated C performs on par with idiomatic hand-written C on the benchmark corpus, with runtime checks eliminated where proven.
- Proof checking is reproducible bit-for-bit across machines and runs.

> **Speculative:** whether the spec ↔ intent gap can really be closed by tooling (weakness detection, witnesses, plain-language rendering) is the project's largest research risk. See [open-questions.md](open-questions.md).
