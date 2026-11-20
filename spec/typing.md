# Name resolution and type checking — phase 1

> **Status: draft, provisional, non-normative.** Syntax follows [grammar-draft.md](grammar-draft.md). Arithmetic semantics are from [ADR-0002](../docs/adr/0002-arithmetic-semantics.md) (accepted). The choices in [§14](#14-decided-points-adr-0004) are accepted by [ADR-0004](../docs/adr/0004-phase-1-typing-choices.md); the `usize`/`isize` static-check rule (§4.1, §11) by [ADR-0005](../docs/adr/0005-portable-static-checks-for-pointer-sized-integers.md). Everything else is a proposal.

## 1. Scope

Phase 1 checks a **single file** of `type` and `fn` items. Contracts, `while` loops and spec contexts are specified separately in [contracts.md](contracts.md) (thin slice, [ADR-0006](../docs/adr/0006-thin-end-to-end-slice.md)); this document covers code. The checker has no proofs: each code-side precondition from ADR-0002 that is not statically trivial becomes a **runtime trap** listed in the *unverified checks* report (§11).

Not checked in phase 1. Each of these parses but is rejected with `E0318`, so the [open questions](../docs/open-questions.md) stay open:

| Construct | Why deferred | Arrives |
|---|---|---|
| Generic parameters on `fn`/`type`, generic arguments, `id::<T>` | [open-questions §7](../docs/open-questions.md#7-generics-and-compile-time-evaluation) undecided | by ADR |
| `mod`, `use` | multi-file compilation not designed | by ADR |
| Non-empty `effects { … }` | effect system | phase 4 |
| `int` outside a quantifier binder, `nat` | spec-only types; the slice needs `int` only as a binder ([contracts.md §5.1](contracts.md#51-types-and-literals)) | phase 2 |
| Named array lengths `[T; N]` | needs const generics or const items | by ADR |
| Recursion (direct or mutual) | needs `decreases` | phase 2 |

## 2. Types

| Type | Values | Notes |
|---|---|---|
| `u8 u16 u32 u64 usize` | unsigned integers | `usize` width = target pointer width (ADR-0004, Q1) |
| `i8 i16 i32 i64 isize` | two's-complement signed integers | `isize` width = `usize` width |
| `bool` | `true`, `false` | not an integer; no conversions to or from it |
| `()` | `()` | unit |
| `S` (struct) | declared with `type S { … }` | nominal; all fields stored inline |
| `[T; N]` | `N` elements of `T` | `N`: integer literal, unsuffixed or `usize`, `1 ≤ N ≤ 2^63 - 1` (`E0322`) |
| `&T`, `&mut T` | references | parameter types only (§8) |
| `&[T]`, `&mut [T]` | slices | parameter types only (§8); bare `[T]` is a parse error |

**Type equality** is structural for built-in constructors and nominal for structs. There is **no subtyping and no coercion** of any kind. In particular:

- no integer widening or narrowing (use `as`);
- `&mut T` is not accepted where `&T` is expected (write `&x`);
- `&[T; N]` is not accepted where `&[T]` is expected (§8.3).

All phase-1 types have **value semantics**: assignment, argument passing and `return` copy the value. References are copied as references.

**Struct well-formedness:** field names are unique (`E0302`); a struct must not contain itself by value, directly or through other structs or arrays (`E0324`); fields cannot have reference types (`E0319`).

## 3. Names and scopes

Goal: every name resolves to exactly one thing ([design-principles §12](../docs/design-principles.md#12-agent-friendly-surface)).

- **One item namespace** per file, containing `type` names, `fn` names and the built-in scalar type names. Duplicates are `E0302`; declaring `type u32` or `fn bool` is also `E0302`.
- **Items are order-independent**: a function may call a function declared later.
- **Locals** (parameters, `let`, `for` variables) live in block scopes. **No shadowing**: a local may not reuse the name of an item, a built-in type or any local visible at that point (`E0302`). A name may be reused once the earlier local's scope has ended (two sibling `for i` loops are fine).
- **Field names** are per struct and never clash with locals or items.
- Unknown names are `E0301`.

## 4. Checking mode: bidirectional

Every expression is either **checked** against an expected type or **synthesizes** one. Expected types come from:

| Position | Expected type |
|---|---|
| `let x: T = e` | `T` |
| `place = e`, `place op= e` | type of `place` |
| argument `i` of a call to `f` | type of parameter `i` |
| `return e` | the function's return type |
| field `f: e` in `S.{ … }` | type of field `f` |
| element of `[a, b]` checked against `[T; N]` | `T` |
| `if` condition, operand of `&&` / `\|\|` / `!` on bool | `bool` |
| index `e[i]` | `usize` for `i` |
| `for i: T in a..b` | `T` for `a` and `b` |

When a check-mode expression synthesizes a type different from the expected one, the error is `E0303`.

### 4.1 Integer literals

- A **suffixed** literal synthesizes its suffix type.
- An **unsuffixed** literal takes the expected type, which must be an integer type. With no expected type it is `E0304` ("add a suffix").
- The literal's value must fit its type, checked statically (`E0305`), not trapped. Static checks use the range every target guarantees ([ADR-0005](../docs/adr/0005-portable-static-checks-for-pointer-sized-integers.md)): `usize`/`isize` count as 32-bit. A constant that only fits on 64-bit targets (`4294967296usize`) is `E0305`; use `u64`.
- **Negative literals:** unary `-` applied directly to an integer literal is folded first, and the negated value is range-checked. So `-128i8` is valid; `128i8` alone is `E0305`.

### 4.2 Binary operators without an expected type

Example: `x < 5` or `(a + 1) == b`. Synthesize the operand that is not an unsuffixed literal, then check the other operand against the result. If both are unsuffixed literals, the result is `E0304`.

## 5. Expressions

| Expression | Rule | Result |
|---|---|---|
| `a + b`, `a - b`, `a * b`, `a / b`, `a % b` | `a`, `b`: same integer type `T` | `T`, trap per §11 |
| `a & b`, `a \| b`, `a ^ b` | same integer type `T` (not `bool`: use `&&`, `\|\|`, `!=`) | `T`, total |
| `a << b`, `a >> b` | same integer type `T` for both (ADR-0004, Q4) | `T`, trap per §11 |
| `-a` | `a`: **signed** integer type (unsigned is `E0306`) | trap if `a == MIN` |
| `!a` | `bool` → logical not; integer → bitwise not | same type, total |
| `a == b`, `a != b` | same type, integer or `bool` | `bool` |
| `a < b`, `a <= b`, `a > b`, `a >= b` | same integer type | `bool` |
| `a && b`, `a \|\| b` | `bool`; short-circuit | `bool` |
| `e as U` | `e` and `U` integer types (`bool` or non-integer: `E0307`); `U == type(e)` allowed (redundant check) | `U`; trap if narrowing (§11) |
| `(e)` | — | type of `e` |
| `()` | — | `()` |
| `[e1, …, en]` | needs expected type `[T; N]` with `N == n` (`E0323`), or synthesizes from `e1` if `e1` synthesizes | `[T; n]` |
| `S.{ f1: e1, … }` | every field of `S` exactly once, **in declaration order** (`E0312`) | `S` |
| `e.f` | `e`: struct (or `&`/`&mut` struct, §8.2) with field `f` (`E0311`) | field type |
| `e.len` | `e`: array or slice (through one reference, §8.2) | `usize`; read-only |
| `e[i]` | `e`: array or slice (through one reference); `i: usize` | element type; trap per §11 |
| `e[lo..hi]` | only as the operand of `&`/`&mut` (`E0325`); `e`: array or slice (through one reference); `lo`, `hi: usize` | see `&` below; trap per §11 |
| `&p`, `&mut p` | `p` a place (`&mut`: assignable, `E0308`); only as a call argument (`E0319`) | `&T` / `&mut T`; `&p[lo..hi]` gives `&[T]` / `&mut [T]` |
| `f(a1, …)` | `f` names a `fn` item (`E0310`); exact argument count (`E0309`); each `ai` checked against parameter `i` | return type |
| `?h` | checked mode only; records the expected type, then `E0321` (§10) | expected type |

**Evaluation order** is left to right for operands, arguments, struct fields and array elements. `&&` and `||` short-circuit. When several traps could fire, the first in evaluation order fires. The C emitter must keep this order explicit, because C leaves it unspecified.

## 6. Statements and control flow

| Statement | Rule |
|---|---|
| `let [mut] x: T = e;` | `e` checked against `T`; `T` must not be a reference type (`E0319`) |
| `place = e;` | `place` must be assignable (§7); `e` checked against its type |
| `place op= e;` | same as `place = place op e` with `place` evaluated once; traps as for `op` |
| `if c { … } else { … }` | `c: bool` |
| `for i: T in a..b { … }` | `T` integer type; `a`, `b` checked against `T`; both evaluated once at loop entry, left to right; iterates `a, a+1, …, b-1`; zero iterations if `a >= b` (no trap); `i` is immutable |
| `while c … { … }` | `c: bool`, evaluated before each iteration; clauses per [contracts.md §4](contracts.md#4-while-loops) |
| `break;`, `continue;` | only inside a `for` or `while` body (`E0315`) |
| `return e;` | `e` checked against the return type |
| `e;` | `e` must be a call returning `()` (`E0316`). Discarding a non-unit result is an error ([Power of 10 rule 7](../docs/power-of-10-mapping.md)); there is no discard construct yet |
| `{ … }` | new scope |

**Definite return:** a function whose return type is not `()` must end in a statement that always returns. A statement *always returns* if it is a `return`, a block whose last statement always returns, or an `if` with an `else` whose branches both always return. A `for` or `while` never counts. Otherwise the error is `E0313`. A `()` function may fall off the end.

**No dead code:** a statement that follows a `return`, `break` or `continue`, or an always-returning statement, in the same block is `E0314`.

## 7. Places and mutability

A **place** is a local, `p.f` or `p[i]` where `p` is a place, or a reference parameter (a place for `.f`/`[i]` through it). A place is **assignable** when:

- it is a `let mut` local, or a field/element of an assignable place; or
- it is reached through a `&mut` reference (`r.f`, `r[i]` with `r: &mut …`).

Parameters, `for` variables and non-`mut` locals are not assignable, and neither is `.len`. Violations are `E0308`.

## 8. References and slices (phase-1 subset)

Phase 1 has no memory model (phase 5). To keep emitted C free of dangling pointers without one, references are **second-class**:

### 8.1 Where references may appear

- **Only as parameter types.** Not in `let`, struct fields, return types, array elements or nested references (`E0319`).
- They are created only at call sites, as arguments: `&place` or `&mut place` (grammar change G1, §13), or by passing on a reference parameter unchanged.
- `&mut place` requires an assignable place (`E0308`).

A reference therefore never outlives the caller's frame, and every referent is live for the whole call.

### 8.2 Using references

`.f`, `.len`, `[i]` and `[lo..hi]` see through **exactly one** reference level. This is the only implicit dereference in phase 1 (ADR-0004, Q3). A reference cannot be read or assigned as a whole value except by passing it on as an argument.

### 8.3 Arrays to slices

`&a` with `a: [T; N]` has type `&[T; N]`, never `&[T]` (no coercion). A slice is written explicitly with a range (ADR-0004, Q2):

```
sum(&a[0usize..a.len])        // &[T] over the whole array
fill(&mut buf[lo..hi], 0u8)   // &mut [T] over a sub-range
```

`p[lo..hi]` is valid only directly under `&` or `&mut` (`E0325`). It traps unless `lo <= hi && hi <= p.len`.

### 8.4 Aliasing at calls

Within one call, if an argument is `&mut p` (or a `&mut` parameter passed on), the **root local** of `p` must not appear in any other argument (`E0320`). This is conservative: `f(&mut a[0], &a[1])` is rejected too. It makes phase-1 code compatible with any phase-5 permission model that forbids mutable aliasing.

## 9. Functions

- Parameter names are unique (`E0302`). Parameters are immutable.
- `effects {}` is the only accepted effect set (`E0318`).
- **No recursion:** the call graph must be acyclic (`E0317`). With bounded `for` loops, this makes every phase-1 function terminate.
- Functions are not values: a call target must be a `fn` item path, and a bare `fn` name in any other position is `E0303`.
- `pub` has no effect on checking in single-file phase 1; it is recorded for later module support.

## 10. Holes

`?h` type-checks in check mode only and takes the expected type; with no expected type it is `E0304`. The checker then reports every hole as an `E0321` error: *"unfilled hole `?h`: expected `u32`"*, listing the locals in scope with their types. Holes therefore never reach the emitter. The structured oracle form is phase 3.

## 11. Runtime traps and the unverified-checks report

These are ADR-0002 preconditions. In phase 1, each one that is not *statically trivial* becomes a runtime trap and an entry in the report.

| Operation (type `T`) | Trap condition |
|---|---|
| `a + b`, `a - b`, `a * b`, `op=` | exact result outside `T` |
| `-a` (signed) | `a == MIN_T` |
| `a / b`, `a % b` | `b == 0`; signed: also `a == MIN_T && b == -1` |
| `a << n` | `n < 0` (signed `n`), `n >= bits(T)`, or `a * 2^n` outside `T` |
| `a >> n` | `n < 0` (signed `n`), `n >= bits(T)`; arithmetic shift for signed `T` |
| `e as U` | value of `e` outside `U` (never for widening casts) |
| `e[i]` | `i >= e.len` |
| `&e[lo..hi]` | `lo > hi` or `hi > e.len` |

**Statically trivial** (no trap, no report entry) is defined syntactically, so the result is deterministic and the same for every agent:

- every operand involved *in that check* is an integer literal (after §4.1 negation folding) and the check passes; if it fails, it is a compile error (`E0305`). Each check involves only the operands it reads: `div-zero` reads the divisor only (`x / 2` has no `div-zero` trap), the `<<` amount check reads the amount only, `div-overflow`, `overflow` and lost bits on `<<` read both operands;
- the cast is widening on every target ([ADR-0005](../docs/adr/0005-portable-static-checks-for-pointer-sized-integers.md)): every value the source can hold on any target (`usize`/`isize` as 64-bit) fits the range the target has on every target (`usize`/`isize` as 32-bit). So `u32 as usize` and `usize as u64` are trivial; `u64 as usize` and `usize as u32` are not;
- `a[k]` where `a` is an array (not a slice) and `k` is a literal with `k < N`; with `k >= N` it is `E0305`. Likewise `&a[lo..hi]` with literal `lo <= hi <= N`.

Nothing else is elided in phase 1: no flow facts, no constant propagation. Phase 2 replaces all of this with proof obligations.

**Trap behavior:** the program terminates and does not return to the caller. It is not a `panic` effect ([ADR-0002](../docs/adr/0002-arithmetic-semantics.md#consequences)). The exact mechanism (e.g. `abort()` after a message naming the check) belongs to the C emitter spec.

**Report:** one entry per trap site and kind, in source order (`quodc check`): `file:line:col`, enclosing function, kind (`overflow`, `negate`, `div-zero`, `div-overflow`, `shift`, `cast`, `bounds`, `slice`), operand type(s). Contract clauses add one entry each, with kind `requires`, `ensures`, `invariant` or `decreases` and no operand types ([contracts.md §6](contracts.md#6-phase-1-behavior)). It is plain text in phase 1, with a stable ordering suitable for diffing. Phase 3 adds a JSON form.

## 12. Diagnostics

New `E03xx` codes are listed in [diagnostics.md](diagnostics.md#type-checker). Phase 1 stops at the first error per file.

## 13. Grammar changes this spec needs

In [grammar-draft.md](grammar-draft.md) (provisional) and implemented in the parser.

| # | Change | Why |
|---|---|---|
| G1 | `unary = ( "-" \| "!" ) unary \| "&" [ "mut" ] unary \| cast_expr ;` (`&&` in unary position is a syntax error) | Creating reference arguments (§8.1) |
| G2 | `postfix` index: `"[" expr ( "]" \| ".." expr "]" )` | Array → slice (ADR-0004, Q2) |
| G3 | `requires`/`ensures` on `fn`, `while_stmt`, `==>`, `result`, `old(e)`, quantifiers | Contracts for the slice ([contracts.md §8](contracts.md#8-grammar-and-diagnostics-changes)); not yet in the parser |

## 14. Decided points (ADR-0004)

Accepted by the maintainer on 2026-11-19; rationale and alternatives in [ADR-0004](../docs/adr/0004-phase-1-typing-choices.md).

| # | Question | Decision |
|---|---|---|
| Q1 | Width of `usize`/`isize` | Target pointer width; static checks per [ADR-0005](../docs/adr/0005-portable-static-checks-for-pointer-sized-integers.md) (§4.1, §11) |
| Q2 | Array → slice | Range slicing `&p[lo..hi]` / `&mut p[lo..hi]` (§8.3) |
| Q3 | Implicit dereference | Exactly one level, for `.f`, `.len`, `[i]`, `[lo..hi]` only (§8.2) |
| Q4 | Shift amount type | Same type as the left operand (§5) |
| Q5 | Struct literal field order | Declaration order required (§5) |
| Q6 | Dead code, discarded results | Errors (§6) |
| Q7 | Recursion | Rejected in phase 1 (§9) |
| Q8 | `mod`/`use` | Rejected; single file (§1) |
