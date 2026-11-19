# ADR-0005: Portable static checks for pointer-sized integers

- **Status:** Accepted
- **Date:** 2026-11-19
- **Deciders:** project maintainer (accepted 2026-11-19)
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** none
- **Supersedes:** decision 1 of [ADR-0004](0004-phase-1-typing-choices.md), second sentence only. The rest of ADR-0004 stands, including Q1(a) (`usize`/`isize` have the target's pointer width).

## Context

ADR-0004 decision 1 says: *"For deciding whether a cast is statically trivial, they [`usize`/`isize`] count as 64-bit, so trivial-check results don't depend on the target."*

Read literally, that rule is unsound. If `usize` counts as 64-bit, then `u64 as usize` counts as widening and gets no runtime check. On a 32-bit target, the cast then silently truncates, which is exactly what ADR-0002 forbids. `spec/typing.md` §11 had the same sentence but then said `u64 as usize` "is not trivial", which contradicts it. The intent was clear (no trap may depend on a guess about the target), but the stated rule does not achieve it.

The same question applies beyond casts. Every static check (literal ranges, constant folding, literal shift amounts, literal indices) needs a width for `usize`/`isize`, and ADR-0004 gives none.

Constraints:
- **Zero UB** ([design-principles §3](../design-principles.md#3-zero-undefined-behavior)): a check may be skipped only if it passes on every target.
- **Determinism:** `quodc check` must give the same diagnostics and the same unverified-checks report on every host and for every target.
- ADR-0004 Q1(a) stays: `usize` is pointer-sized in emitted C.

## Options considered

### Option A: Conservative per-direction widths (implemented)
Static reasoning uses the width that makes the conclusion true on every target:
- a value *produced* by a `usize`/`isize` expression may need up to 64 bits;
- a value *stored into* `usize`/`isize` must fit in 32 bits.

For casts: skip the check only if every value the source can hold on any target fits the range the destination has on every target. For constants: they must fit the 32-bit range.
- For: sound on every target with pointer width 32 or 64. Results don't depend on the target. One rule covers casts, literals, folding and shift amounts.
- Against: rejects some valid 64-bit-only constants (`4294967296usize` is `E0305`; write `u64` and cast). `usize as u32` always keeps a runtime check, even on 32-bit targets.

### Option B: Fix `usize` at 64 bits everywhere
- For: one width, no special cases.
- Against: reverses ADR-0004 Q1. It misdescribes 32-bit targets, and C interop through `size_t` breaks there.

### Option C: Width from the selected target
- For: no rejected constants; skips every check that is truly trivial on that target.
- Against: diagnostics and the unverified-checks report depend on the build target. The same source can be accepted for one target and rejected for another, and agents see different obligations per target. This breaks the determinism goal.

### Option D: Keep ADR-0004 wording, read as "conservative"
- For: no new ADR.
- Against: the accepted text says the opposite of the implemented rule. An accepted ADR that misstates the rule will mislead later readers and agents.

## Decision

Option A. Replace the second sentence of ADR-0004 decision 1 with:

> Static checks treat `usize`/`isize` conservatively and independently of the target. A value of type `usize`/`isize` is assumed to need up to 64 bits, and a value stored into `usize`/`isize` must fit in 32 bits. So a cast skips its runtime check only when it widens on every target (`u32 as usize`, `usize as u64`). Constants of type `usize`/`isize` must fit the 32-bit range (`E0305` otherwise).

Targets with a pointer width other than 32 or 64 bits are out of scope until an ADR adds them.

## Consequences

- **Already implemented:** `compiler/src/types.rs` (`IntTy::portable_range`, `widest_range`, `widens_to`) and `compiler/src/check.rs`. Covered by `tests/typecheck/` (`ok_usize_casts.qd`, `ok_literals.qd`, `err_E0305_usize.qd`) and unit tests in `types.rs`. No code change is needed on acceptance.
- **Spec:** `spec/typing.md` §4.1 and §11 already state this rule; on acceptance, link them to this ADR.
- **C emitter (future):** must not assume `usize` is 64-bit. Constants of type `usize` fit 32 bits by construction, so they can be emitted as plain `size_t` literals.
- **Trusted base:** none. The rule lives in the untrusted front end. Phase 2 inherits it: obligations involving `usize` must hold for both widths, or be stated per target by a later ADR.
- **Agent loop:** one deterministic set of diagnostics and runtime checks per source file. Large constants need `u64`, which `E0305`'s message says.
- On acceptance: set ADR-0004's status line to note that decision 1 is partly superseded. This is the only edit the ADR process allows to an accepted ADR. Update the ADR index and `CLAUDE.md`.

## Validation

- `tests/typecheck/ok_usize_casts.qd` covers casts in both directions between `usize`/`isize` and fixed-width types. Expected: runtime checks exactly where some 32- or 64-bit target could fail. To add when the C emitter lands: run the emitted C with UBSan on a 32-bit and a 64-bit target.
- Revisit if 64-bit-only constants in `usize` turn out common in the corpus, or if a 16-bit target becomes a goal.
