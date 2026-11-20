# ADR-0014: Alias-free, fresh slice views as Core IR well-formedness rules

- **Status:** Proposed
- **Date:** 2026-11-20
- **Deciders:** —
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** none
- **Depends on:** [ADR-0007](0007-trusted-obligation-generator.md) (Core IR, trusted `coreir`), [ADR-0010](0010-trusted-c-emitter-from-core-ir.md) (trusted `cgen`), [ADR-0011](0011-v1-scope.md) (`mut` slices are copy-in/copy-out)
- **Evidence gate:** S3 of [ADR-0006](0006-thin-end-to-end-slice.md): `coreir` rejects every program in a negative corpus that breaks rule 1 or 2 and accepts every lowered program of the slice corpus; on the slice corpus, the compiled C agrees with an untrusted copy-in/copy-out Core IR interpreter (pending)

## Context

`vcgen` and `cgen` read the same Core IR text but give slices different execution models:

- `vcgen` ([core-ir.md §7.3](../../spec/core-ir.md#73-calls)) treats a `mut` slice argument as copy-in/copy-out (ADR-0011), and a view as a value (base, offset, length) computed when its `let` runs.
- `cgen` ([c-subset.md §6](../../spec/c-subset.md#6-slices-views-and-calls)) passes a pointer: the callee writes in place, and a view reads memory when it is used.

The models agree only if no other argument aliases a `mut` argument, and nothing writes a view's root between the view's definition and its use. Neither is a Core IR well-formedness rule ([core-ir.md §3.4](../../spec/core-ir.md#34-well-formedness-checked-by-coreir)), so `coreir` accepts programs where the verified and the emitted behavior differ. The front end does not prevent it either. Today `quodc lower` accepts

```
return f(&a[0..2], g(&mut a[0..1]));
```

and emits `(let $1 … (view a …))`, `(let $3 u32 (call g $2))` with `$2` an `mview` of `a`, then `(call f $1 $3)`. `vcgen` would give `f` the contents of `a` before `g` ran; the C gives `f` the contents after. E0320 ([typing.md §8.4](../../spec/typing.md#84-aliasing-at-calls)) checks only arguments that are themselves `&mut`, not calls nested inside other arguments. Even a complete front-end rule would not be enough, because the front end is untrusted.

## Options considered

### Option A: Well-formedness rules in `coreir` (trusted)
Add to §3.4:
1. **No aliasing at a call.** The *root* of a slice value is the array local or slice parameter that a `view`/`mview` names, or the slice parameter itself. In each `call`, if an argument is a `mut` slice, its root is the root of no other argument.
2. **Views are fresh.** A value defined by `view`/`mview` is used exactly once, as a `call` argument in the same statement list as its definition. No statement between the definition and that call (including statements nested in them) `store`s to its root or passes a `mut` slice with the same root to a call.
- For: a syntactic check of a few dozen lines in `coreir`; both trusted consumers may then rely on it; `cgen` stays in place and copy-free. Lowering already places views at their argument position, so only programs like the example above change.
- Against: grows `coreir` (budget ≤ 1,500 lines) and the Core IR semantics in the trusted base. Rule 2 rejects some harmless programs (a view reused across two calls); lowering never produces them.

### Option B: `cgen` copies `mut` slices in and out
- For: matches ADR-0011's meaning literally.
- Against: slice lengths are dynamic, so the copy needs a VLA or heap memory; the C subset forbids both. It does not fix rule 2 (stale `ref` views).

### Option C: Change `vcgen` to read views at the call
- For: no new check.
- Against: fixes rule 2 only; aliasing `mut` arguments still break copy-in/copy-out, and `vcgen` gets more complex.

### Option D: Rely on the front end (E0320 and lowering)
- For: no trusted change.
- Against: puts the untrusted front end in the soundness argument, which ADR-0007 and ADR-0010 exist to avoid; and the current front end already misses the example above.

## Decision

Option A. Rules 1 and 2 become Core IR well-formedness rules, checked by `coreir`. The front end is also fixed, so that it never emits Core IR that `coreir` rejects: E0320 extends to `&mut` borrows nested anywhere in another argument, or lowering rejects such calls with E0401. That is an untrusted implementation detail, decided when it is built.

## Consequences

- **Trusted base:** grows. Core IR semantics gain rules 1 and 2 ([core-ir.md §3.4](../../spec/core-ir.md#34-well-formedness-checked-by-coreir)), and `coreir` gains their check (estimated < 100 lines, within budget). In exchange, `cgen` can implement `mut` slices in place without copies, and `vcgen`'s snapshot reading of views is exact.
- **Front end:** the example above becomes a compile error. Programs that break the rules are rare in the slice corpus: none of the current `tests/lower/` programs do.
- **Agent loop:** none. The rule matches the phase-5 direction of forbidding mutable aliasing (typing.md §8.4).
- Docs to update on acceptance: `spec/core-ir.md` §3.4 (move rules 1–2 from the proposal note into the list), `spec/c-subset.md` §6.3 (drop the "until accepted" note), `spec/typing.md` §8.4 (E0320 covers nested borrows, if that is how the front end is fixed), `spec/diagnostics.md` if a code changes.

## Validation

- **Gate (S3):** a negative corpus in `tests/coreir/` (aliasing `mut` arguments; a view used after a `store` to its root; a view used after a call that writes its root; a view used twice; a view used in a nested block) is rejected by `coreir`; every lowered program of the slice corpus is accepted; on the slice corpus, the C compiled from `cgen` and an untrusted copy-in/copy-out Core IR interpreter give the same outputs.
- Revisit if rule 2 rejects programs agents write naturally (for example, one view passed to several calls), or when phase 5 brings a general permission model.
