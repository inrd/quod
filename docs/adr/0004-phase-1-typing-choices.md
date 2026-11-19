# ADR-0004: Phase-1 typing choices

- **Status:** Accepted
- **Date:** 2026-11-19
- **Deciders:** project maintainer (accepted 2026-11-19)
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** none. [open-questions.md §7](../open-questions.md#7-generics-and-compile-time-evaluation) (generics) stays open; phase 1 rejects generics.
- **Depends on:** ADR-0002 (arithmetic semantics)

## Context

The phase-1 type checker is specified in [spec/typing.md](../../spec/typing.md) (draft). Eight points in it needed a maintainer decision (typing.md §14, Q1–Q8, now recorded there as decided). They govern what phase-1 code may say, and therefore what the C emitter and the phase-2 obligation generator must handle. Constraints: nothing implicit ([design-principles §12](../design-principles.md#12-agent-friendly-surface)), zero UB without a memory model (phase 5), and keeping undecided questions open.

## Options considered

| # | Question | Options | Chosen | Why |
|---|---|---|---|---|
| Q1 | Width of `usize`/`isize` | (a) target pointer width; (b) fixed 64-bit | **(a)** | Honest C interop (`size_t`); (b) would misdescribe 32-bit targets. Cost: traps can differ per target |
| Q2 | Array → slice | (a) slicing syntax `a[lo..hi]`; (b) built-in `slice(&a)`; (c) coercion | **(a)** | Also covers sub-slices, which are needed anyway; (c) is an implicit conversion |
| Q3 | Dereference for `.f`, `.len`, `[i]` through a reference | (a) implicit, exactly one level; (b) explicit deref syntax | **(a)** | Affects places only and never changes a value's type; (b) adds noise to every slice access |
| Q4 | Shift amount type | (a) same as the left operand; (b) any unsigned; (c) always `u32` | **(a)** | One rule for every binary operator |
| Q5 | Struct literal field order | (a) declaration order required; (b) any order | **(a)** | One canonical form; reordering is a likely agent slip |
| Q6 | Dead code and discarded results | (a) errors; (b) warnings | **(a)** | No warnings ([Power of 10 rule 10](../power-of-10-mapping.md)) |
| Q7 | Recursion in phase 1 | (a) rejected; (b) allowed, termination unchecked until phase 2 | **(a)** | With bounded loops, every phase-1 function terminates |
| Q8 | `mod`/`use` in phase 1 | (a) rejected, single file; (b) design multi-file now | **(a)** | Modules deserve their own design doc |

## Decision

1. `usize`/`isize` have the target's pointer width. For deciding whether a cast is statically trivial, they count as 64-bit, so trivial-check results don't depend on the target.
2. Arrays and slices are sliced with `p[lo..hi]` (`lo`, `hi: usize`). This is valid only directly under `&` or `&mut`, has type `&[T]` / `&mut [T]`, and traps unless `lo <= hi && hi <= p.len`. There is no coercion from `&[T; N]` to `&[T]`.
3. `.f`, `.len` and `[i]` (including `[lo..hi]`) see through exactly one reference level. No other implicit dereference exists.
4. Both operands of `<<` and `>>` have the same integer type.
5. Struct literals list every field exactly once, in declaration order.
6. Unreachable statements and discarded non-`()` results are compile errors.
7. Phase 1 rejects recursion (`E0317`). Recursion returns in phase 2 with `decreases`.
8. Phase 1 compiles a single file. `mod`/`use` parse but are rejected (`E0318`) until a module design is accepted.

## Consequences

- **Grammar:** Q2 adds range indexing to `postfix`. Creating reference arguments also needs a unary `&`/`&mut` (typing.md §13, G1). Both go into the provisional grammar draft.
- **Emitter:** Q1 means trap sites involving `usize` behave per target. The unverified-checks report names the type, so the dependence is visible.
- **Phase 2:** Q3's one-level deref and Q2's slice bounds become ordinary obligations (`i < r.len`, `lo <= hi <= p.len`). Q7 and Q8 are lifted by later ADRs or phase-2 work.
- **Trusted base:** none. All of this lives in the untrusted front end.
- **Agent loop:** fewer ways to write the same thing (Q4, Q5) and no warnings to ignore (Q6), so agents get one clear error instead of a choice.
- Docs updated: `spec/typing.md` (Q1–Q8 marked resolved), `spec/grammar-draft.md` (G1, G2), `spec/diagnostics.md`.

## Validation

- Phase-1 corpus: every example and test that type-checks also compiles to warning-free, UBSan-clean C (phase-1 exit criterion).
- Revisit Q3 if agents misread values reached through references as copies. Revisit Q5 if field-order errors dominate agent diagnostics in the phase-3 loop. Revisit Q1 if per-target trap differences cause corpus results to diverge between targets.
