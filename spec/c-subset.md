# C subset emitted by `cgen` — slice subset

> **Status: draft, provisional, non-normative.** Step S3a of the thin slice ([ADR-0006](../docs/adr/0006-thin-end-to-end-slice.md)). Built to Proposed [ADR-0010](../docs/adr/0010-trusted-c-emitter-from-core-ir.md) (trusted `cgen`) under [ADR-0012](../docs/adr/0012-evidence-before-acceptance.md)'s rule for gated designs. If ADR-0010 is accepted, this document becomes part of the **trusted base**, next to `cgen`. In-place `mut` slices (§6) also rely on Proposed [ADR-0014](../docs/adr/0014-alias-free-views-in-core-ir.md). No code implements it yet (S3).

## 1. Role and target

`cgen` reads one Core IR file ([core-ir.md](core-ir.md)) through `coreir`'s parser and well-formedness checks, and writes one C file. The translation is syntax-directed: one Core IR statement becomes one C statement, preceded by at most one trap check (§5).

| Item | Rule |
|---|---|
| Language | ISO C99 only. No C11 (`_Static_assert`, `_Noreturn`, `_Generic`, anonymous members), no GNU or other extensions, no compiler builtins, no VLAs, no `restrict`, no `inline`, no floating point, no signed integer types in emitted code |
| Compilers | gcc and clang with `-std=c99 -Wall -Wextra -Werror`; CompCert (`ccomp`). The S3 experiment records exact versions and flags |
| Headers | `<limits.h>`, `<stddef.h>`, `<stdint.h>`, `<stdio.h>`, `<stdlib.h>`; only `fprintf`, `stderr` and `abort` are called |
| Preprocessor | `#include`, one `#define` (`QD_USIZE_MAX`) and one `#if` (§2.2) in the prelude; nothing else |
| Out of scope | `main`, entry points, I/O other than the trap message, `static` data, heap allocation (§10) |

> **Speculative:** CompCert may need `-fstruct-passing` for the slice structs passed by value (§3). S3 confirms the flag set; if a flag changes the meaning of the subset, this document is revised before `cgen` relies on it.

## 2. Implementation-defined assumptions

### 2.1 Checked in every emitted file

Each assumption is a file-scope typedef whose array size is `-1` (a compile error) when the assumption is false. The exact-width macros are undefined when `uint32_t`/`uint64_t` do not exist, so the same lines also check that the types exist.

| Name | Condition | Why |
|---|---|---|
| `char_bit` | `CHAR_BIT == 8` | exact-width types and the trap message assume 8-bit bytes |
| `int_32` | `INT_MAX == 2147483647 && UINT_MAX == 4294967295u` | `uint32_t` never promotes to signed `int`, so every `u32` operation is computed in a 32-bit unsigned type (§4.1) |
| `uint32` | `UINT32_MAX == 4294967295u` | `uint32_t` exists |
| `uint64` | `UINT64_MAX == 18446744073709551615u` | `uint64_t` exists |
| `size` | `(uintmax_t)SIZE_MAX == (uintmax_t)UINT32_MAX \|\| (uintmax_t)SIZE_MAX == (uintmax_t)UINT64_MAX` | `size_t` is 32 or 64 bits ([ADR-0005](../docs/adr/0005-portable-static-checks-for-pointer-sized-integers.md): other widths out of scope); so `size_t` never promotes to `int` |

Emitted text, in this order:

```c
typedef char qd_assume_char_bit[(CHAR_BIT == 8) ? 1 : -1];
typedef char qd_assume_int_32[(INT_MAX == 2147483647 && UINT_MAX == 4294967295u) ? 1 : -1];
typedef char qd_assume_uint32[(UINT32_MAX == 4294967295u) ? 1 : -1];
typedef char qd_assume_uint64[(UINT64_MAX == 18446744073709551615u) ? 1 : -1];
typedef char qd_assume_size[((uintmax_t)SIZE_MAX == (uintmax_t)UINT32_MAX || (uintmax_t)SIZE_MAX == (uintmax_t)UINT64_MAX) ? 1 : -1];
```

Two's-complement and signed-conversion assumptions are not needed: the slice has no signed types. They are added here when signed types enter the subset.

### 2.2 Target-dependent code

Exactly one place in the file depends on the target: whether a `size_t` value can exceed `2^32 - 1`. It is decided by the preprocessor, so no comparison is a tautology on either width (which `-Wtype-limits`/`-Wtautological-*` would reject under `-Werror`):

```c
#define QD_USIZE_MAX ((size_t)4294967295u)

static int qd_usize_wide(size_t a) {
#if SIZE_MAX > UINT32_MAX
    return a > QD_USIZE_MAX;
#else
    (void)a;
    return 0;
#endif
}
```

`QD_USIZE_MAX` is the top of `usize`'s portable range ([core-ir.md §5](core-ir.md#5-code-operations)). `qd_usize_wide` is emitted only if a trap check uses it.

### 2.3 Assumed, not checkable in C

Listed in the trust report under the C toolchain:

- the C compiler and C library conform to C99 for the constructs used here;
- `abort()` does not return; `fprintf` to `stderr` does not invoke undefined behavior for the arguments `qd_trap` passes;
- the implementation distinguishes identifiers of any length (C99 guarantees only 31 significant characters for external names; gcc, clang and CompCert have no limit);
- the caller of an emitted function follows the calling convention of §6.3.

## 3. Types

| Core IR type | C type | Notes |
|---|---|---|
| `u32` | `uint32_t` | |
| `u64` | `uint64_t` | |
| `usize` | `size_t` | pointer-sized, as [ADR-0004](../docs/adr/0004-phase-1-typing-choices.md) Q1 requires; the C type of lengths and indices, so slices map to C objects without conversions. Values may exceed `2^32 - 1` on 64-bit targets (e.g. a length from the caller); every *result* of `add`/`sub`/`mul`/`shl` is checked to be `≤ QD_USIZE_MAX` (§5), which enforces ADR-0005's portable range. Literals are `≤ 2^32 - 1` (checked by `coreir`) |
| `bool` | `_Bool` | values `0`/`1` only; every value is produced by an explicit `(_Bool)` cast of a comparison, of `!`, or of a literal (§4). `<stdbool.h>` is not used |
| `(array T N)` | `qd_arr_<T>_<N>`: `typedef struct { <C T> e[N]; } …` | a struct, so `let`/`var` copies and initialization have value semantics. `N ≥ 1` (an array literal has at least one element) |
| `(ref (slice T))` | `qd_ref_<T>`: `typedef struct { const <C T> *p; size_t n; } …` | `p` points to `n` readable elements; never null (§6.3) |
| `(mut (slice T))` | `qd_mut_<T>`: `typedef struct { <C T> *p; size_t n; } …` | `p` points to `n` writable elements; never null |
| `unit` (return type) | `void` | |

`<T>` in type names is the Core IR scalar name (`u32`, `u64`, `usize`, `bool`). Structs are passed and returned by value; only slices are ever passed (arrays are locals only).

## 4. Values and operations

### 4.1 No meaning-changing promotions

- Under `int_32`, `uint32_t` (rank possibly below `int`) promotes, if at all, to the 32-bit `unsigned int`; `uint64_t` and `size_t` do not promote. So every machine-integer operation is computed in an unsigned type of exactly the operand's width.
- Both operands of a binary operation always have the same C type (Core IR has no mixed operations), so the usual arithmetic conversions change nothing.
- Every arithmetic result is cast back to the operand type: `(uint32_t)(a + b)`. The cast never changes a value, because the trap check (§5) guarantees the exact result is representable. **Unsigned wraparound is never relied on**: no emitted operation, in code or in a trap check, ever wraps.
- `_Bool` operands promote to `int` with value `0`/`1`; this happens only in `!`, in `==`/`!=` on bools (written with explicit `(int)` casts), and in controlling expressions of `if`. All are value-preserving.

### 4.2 Atoms

| Atom | C |
|---|---|
| name | mangled name (§7) |
| `Nu32`, `Nu64`, `Nusize` | `(uint32_t)Nu`, `(uint64_t)Nu`, `(size_t)Nu` (decimal; the `u` suffix gives an unsigned type that holds `N`) |
| `true`, `false` | `(_Bool)1`, `(_Bool)0` |

Every atom is a primary expression or a cast, so it binds tighter than any operator it is an operand of; no parentheses are added around atoms.

### 4.3 Right-hand sides

`a`, `b`, `n`, `i`, `lo`, `hi` are atoms; `x` is a sequence name; `T` is the C type of the operation's `T`.

| Core IR | C expression |
|---|---|
| atom `a` | `a` |
| `add.T a b` / `sub.T` / `mul.T` | `(T)(a + b)` / `(T)(a - b)` / `(T)(a * b)` |
| `div.T a b` / `rem.T` | `(T)(a / b)` / `(T)(a % b)` (C99 truncates toward zero; operands are non-negative) |
| `shl.T a n` / `shr.T a n` | `(T)(a << n)` / `(T)(a >> n)` |
| `band.T` / `bor.T` / `bxor.T` | `(T)(a & b)` / `(T)(a \| b)` / `(T)(a ^ b)` |
| `not.u32 a` / `not.u64 a` | `(uint32_t)~a` / `(uint64_t)~a` |
| `not.bool a` | `(_Bool)!a` |
| `eq.T` `ne.T` `lt.T` `le.T` `gt.T` `ge.T` | `(_Bool)(a == b)`, `!=`, `<`, `<=`, `>`, `>=` |
| `eq.bool a b` / `ne.bool a b` | `(_Bool)((int)a == (int)b)` / `(_Bool)((int)a != (int)b)` |
| `(len x)`, `x` an array of length `N` | `(size_t)Nu` |
| `(len x)`, `x` a slice | `x.n` |
| `(load x i)` | `x.e[i]` (array) / `x.p[i]` (slice) |
| `(array T a₁ … aₙ)` | initializer `{ { a₁, …, aₙ } }` (only as the right-hand side of a declaration) |
| `(view x lo hi)` | initializer `{ (const T *)(B + lo), (size_t)(hi - lo) }` |
| `(mview x lo hi)` | initializer `{ B + lo, (size_t)(hi - lo) }` |
| `(call f a₁ … aₙ)` | `f_f(a₁, …, aₙ)` |

In `view`/`mview`, `B` is `x.e` for an array and `x.p` for a slice parameter. `B + lo` with `lo ≤ len` is a pointer into or one past the end of the same object, which C99 defines; the trap check (§5) runs first. Function arguments are atoms, so C's unspecified argument evaluation order cannot matter.

### 4.4 Statements

| Core IR | C |
|---|---|
| `(let x τ rhs)` | `const C(τ) x = <rhs>;` |
| `(var x τ rhs)` | `C(τ) x = <rhs>;` |
| `(set x a)` | `x = a;` |
| `(store x i v)` | `x.e[i] = v;` (array) / `x.p[i] = v;` (slice) |
| `(call f a…)` | `f_f(a…);` |
| `(if c (then S) (else E))` | `if (c) {` S `} else {` E `}`; `else` omitted when E is empty; an empty S is `if (c) {` + newline + `}` |
| `(loop (head H) (cond c) … (body B))` | `for (;;) {` H `if (!c) {` `break;` `}` B `}` |
| `(return a)` / `(return)` | `return a;` / `return;` |

- **Order:** C statements appear in Core IR statement order. `cgen` never reorders, merges, splits (beyond the trap check), folds or drops statements. Core IR is in A-normal form, so each C statement performs at most one operation.
- **Scopes:** each Core IR block (`then`, `else`, loop) is one C block. Head and body statements share the `for` block, as head and body share the loop's scope in `coreir`. Names are unique per function, so C declarations never shadow.
- **Contracts and terms are not emitted:** `requires`, `ensures`, `invariant` and `decreases` produce no C, as [contracts.md §6](contracts.md#6-phase-1-behavior) requires.
- **Unused names:** a parameter or local that no statement reads gets `(void)x;` right after its declaration (parameters: at the top of the body, in parameter order), so `-Wunused-*` cannot fail a build. This is the only statement `cgen` adds.

### 4.5 Functions

- Return type per §3; parameters `const C(τ) name`, separated by `, `; `void` for no parameters. Prototype and definition have identical text.
- All functions have external linkage (Core IR has no visibility), so a harness can call them (§10).

## 5. Phase-1 traps

### 5.1 `qd_trap`

Emitted once, only if the file has at least one trap check:

```c
static void qd_trap(const char *kind, const char *fn, unsigned long n) {
    (void)fprintf(stderr, "quod trap: %s/%s/%lu\n", fn, kind, n);
    abort();
}
```

A trap prints one line and terminates; it does not return to the caller and is not a `panic` effect ([typing.md §11](typing.md#11-runtime-traps-and-the-unverified-checks-report)). A trap check is

```c
if (<condition>) {
    qd_trap("<kind>", "<fn>", <n>ul);
}
```

placed immediately before the C statement for the Core IR statement holding the site. The operation itself is written only after its check, so it never runs with a failing precondition.

**Site identity.** `<fn>` is the Core IR function name (a source identifier, safe in a string literal). `<kind>` is the obligation kind of [core-ir.md §5](core-ir.md#5-code-operations). `<n>` counts the function's sites of that kind from 0 in Core IR text order, **including elided sites**. So `<fn>/<kind>/<n>` is the site's obligation ID ([core-ir.md §7.6](core-ir.md#76-obligation-ids)) and its source-map key ([core-ir.md §8](core-ir.md#8-lowering-front-end-untrusted)). Kinds that can trap: `overflow`, `div-zero`, `shift`, `bounds`, `slice`. `call-requires` and contract clauses never trap (contracts are erased).

### 5.2 Pre-checks

`M` is `UINT32_MAX`, `UINT64_MAX`, `QD_USIZE_MAX` and `W` (bits) is `32`, `64`, `32` for `u32`, `u64`, `usize` ([core-ir.md §5](core-ir.md#5-code-operations): `bits(usize)` is 32). Each condition reads only the operands, evaluates without wraparound or undefined behavior (sub-expressions that could wrap are guarded by `||`/`&&` short-circuit), and is true exactly when the precondition fails.

| Operation | Part (operands it reads) | Trap condition |
|---|---|---|
| `add.u32`, `add.u64` | overflow (`a`, `b`) | `b > M - a` |
| `add.usize` | overflow (`a`, `b`) | `qd_usize_wide(a) \|\| b > QD_USIZE_MAX - a` |
| `sub.u32`, `sub.u64` | overflow (`a`, `b`) | `b > a` |
| `sub.usize` | overflow (`a`, `b`) | `b > a \|\| qd_usize_wide(a - b)` |
| `mul.T` | overflow (`a`, `b`) | `a != 0 && b > M / a`; if `a` is a literal: `b != 0 && a > M / b` (never divides by a constant zero) |
| `div.T`, `rem.T` | div-zero (`b`) | `b == 0` |
| `shl.T a n` | amount (`n`); lost bits (`a`, `n`) | `n >= W \|\| a > (M >> n)` |
| `shr.T a n` | amount (`n`) | `n >= W` |
| `load`/`store` on array of length `N` | bounds (`i`) | `i >= (size_t)Nu` |
| `load`/`store` on slice `x` | bounds (`i`, `x.n`) | `i >= x.n` |
| `view`/`mview` on array of length `N` | slice (`lo`, `hi`) | `lo > hi \|\| hi > (size_t)Nu` |
| `view`/`mview` on slice `x` | slice (`lo`, `hi`, `x.n`) | `lo > hi \|\| hi > x.n` |

Literals in conditions are written as atoms (§4.2); `0` in `a != 0`/`b == 0` is written `(T)0u`. Comparisons are between operands of the same C type.

### 5.3 Elision

Exactly [core-ir.md §5](core-ir.md#5-code-operations) "Phase-1 trap elision", which is typing.md §11's *statically trivial*:

- A **part** is dropped when every operand it reads is a literal atom (array lengths count as literals; `x.n` does not) and the part passes. The site's check is omitted when all its parts are dropped; otherwise the remaining parts are joined with `||` in the table's order.
- If a part's operands are all literals and the part **fails**, `cgen` refuses the file (no C is written). The front end never produces this (`E0305`); refusing keeps `cgen` sound on hand-written Core IR.
- Nothing else is elided: no flow facts, no constant propagation, no use of contracts.

Since lowering keeps literals as atoms, the sites with a trap check in the C are exactly the entries of the unverified-checks report (an S3 test).

From phase 2 on, a program that passes `check` has every precondition proven and `cgen` emits no checks ([ADR-0010](../docs/adr/0010-trusted-c-emitter-from-core-ir.md) decision 3); there is no mode in which `cgen` chooses which checks to keep.

## 6. Slices, views and calls

### 6.1 Building views

`(let t (ref (slice T)) (view x lo hi))` becomes the slice check followed by `const qd_ref_<T> t = { (const C(T) *)(B + lo), (size_t)(hi - lo) };`; `mview` likewise with `qd_mut_<T>` and no pointer cast. `hi - lo` is computed after the check, so `lo ≤ hi`.

### 6.2 Arguments and results

- Scalars and slice structs are passed by value. A slice struct is two words; the elements are not copied.
- A non-`unit` result is returned by value and bound by `let`; a `unit` call is a statement.

### 6.3 `mut` slices in place, and the calling convention

[ADR-0011](../docs/adr/0011-v1-scope.md) gives `mut` slice parameters copy-in/copy-out meaning, and `vcgen` models them that way ([core-ir.md §7.3](core-ir.md#73-calls)). `cgen` passes a pointer and writes in place. The two agree when:

1. **No aliasing at a call:** a `mut` slice argument's root (the array or slice parameter it views) is the root of no other argument of that call.
2. **Views are fresh at the call:** between a view's definition and its use, no statement writes its root. `vcgen` reads a view's contents as of its definition; C reads them at the call.
3. **Partial writes are unobservable:** a callee that traps never returns, so no caller sees a half-finished copy-out.

`coreir` does not check (1) or (2) today; [ADR-0014](../docs/adr/0014-alias-free-views-in-core-ir.md) proposes adding both to the well-formedness rules ([core-ir.md §3.4](core-ir.md#34-well-formedness-checked-by-coreir)). Until it is accepted, in-place `mut` slices are part of that ADR's experiment, not a settled design.

For external callers (the S3 harness, later C interop), every slice argument has a non-null `p` pointing to `n` valid elements, and a `mut` slice argument overlaps no other slice argument. This is (1) at the boundary; it is assumed (§2.3), not checked.

## 7. Names

Core IR names are injected into disjoint C identifier families by their first character:

| Core IR | C | Example |
|---|---|---|
| function `f` | `f_f` | `find` → `f_find` |
| parameter or local `x` | `v_x` | `xs` → `v_xs` |
| renamed local `x.N` | `vN_x` | `x.1` → `v1_x` |
| temporary `$N` | `t_N` | `$3` → `t_3` |
| `cgen` prelude | `qd_…`, `QD_…` | `qd_trap`, `qd_ref_u32`, `QD_USIZE_MAX` |

- **Injective:** families differ in their first character (`f`, `v`, `t`, `q`/`Q`); within `v`, the character after `v` is `_` (source name) or a digit (renamed, and the digits end at the first `_`).
- **No keywords:** every emitted name has `_` as its second character or a digit after `v`; no C99 keyword does (`_Bool`, `_Complex`, `_Imaginary` start with `_`).
- **No reserved identifiers:** no name starts with `_`. No name matches a reserved pattern of the included headers (`int…_t`, `uint…_t`, `E[0-9A-Z]…`, `is[a-z]…`, `to[a-z]…`, `str[a-z]…`, `mem[a-z]…`, `SIG…`, `LC_…`), since the second character is `_` or a digit. No library macro (`stdin`, `stderr`, `EOF`, …) has this shape.
- A Quod function named `main` becomes `f_main`, so it never collides with the harness's `main`.

## 8. File layout

The file is a pure function of the Core IR text: byte-identical across runs, hosts and targets. No timestamps, paths, versions or host data.

1. `/* Generated by quodc cgen from Core IR (quod-core 0). Do not edit. */`
2. The five `#include`s, in the order of §1.
3. The assumption typedefs (§2.1).
4. `#define QD_USIZE_MAX …` (always).
5. `qd_usize_wide`, if used; `qd_trap`, if used.
6. Slice typedefs used anywhere in the file, in the order `ref` before `mut`, then `u32`, `u64`, `usize`, `bool`; then array typedefs used, ordered by scalar (same order), then `N` ascending. Each typedef is written over three lines (`typedef struct {`, the member(s) one per line, `} name;`).
7. One prototype per function, in Core IR file order.
8. One definition per function, in Core IR file order.

Formatting: ASCII, LF line endings, 4-space indentation, `{` at the end of the opening line, `}` on its own line, one statement per line, one space around binary operators and after commas, none inside parentheses. Items 1–8 are separated by one blank line; the prototypes form one block; definitions are separated by one blank line. The file ends with one newline.

## 9. Worked example: `find`

Core IR from [core-ir.md §2](core-ir.md#2-example). Sites: `(load xs i)` is `find/bounds/0`; `(add.usize i 1usize)` is `find/overflow/0` (its right operand is a literal, its left is not, so the check stays). The comparisons have no precondition.

```c
/* Generated by quodc cgen from Core IR (quod-core 0). Do not edit. */

#include <limits.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

typedef char qd_assume_char_bit[(CHAR_BIT == 8) ? 1 : -1];
typedef char qd_assume_int_32[(INT_MAX == 2147483647 && UINT_MAX == 4294967295u) ? 1 : -1];
typedef char qd_assume_uint32[(UINT32_MAX == 4294967295u) ? 1 : -1];
typedef char qd_assume_uint64[(UINT64_MAX == 18446744073709551615u) ? 1 : -1];
typedef char qd_assume_size[((uintmax_t)SIZE_MAX == (uintmax_t)UINT32_MAX || (uintmax_t)SIZE_MAX == (uintmax_t)UINT64_MAX) ? 1 : -1];

#define QD_USIZE_MAX ((size_t)4294967295u)

static int qd_usize_wide(size_t a) {
#if SIZE_MAX > UINT32_MAX
    return a > QD_USIZE_MAX;
#else
    (void)a;
    return 0;
#endif
}

static void qd_trap(const char *kind, const char *fn, unsigned long n) {
    (void)fprintf(stderr, "quod trap: %s/%s/%lu\n", fn, kind, n);
    abort();
}

typedef struct {
    const uint32_t *p;
    size_t n;
} qd_ref_u32;

size_t f_find(const qd_ref_u32 v_xs, const uint32_t v_key);

size_t f_find(const qd_ref_u32 v_xs, const uint32_t v_key) {
    size_t v_i = (size_t)0u;
    for (;;) {
        const size_t t_0 = v_xs.n;
        const _Bool t_1 = (_Bool)(v_i < t_0);
        if (!t_1) {
            break;
        }
        if (v_i >= v_xs.n) {
            qd_trap("bounds", "find", 0ul);
        }
        const uint32_t t_2 = v_xs.p[v_i];
        const _Bool t_3 = (_Bool)(t_2 == v_key);
        if (t_3) {
            return v_i;
        }
        if (qd_usize_wide(v_i) || (size_t)1u > QD_USIZE_MAX - v_i) {
            qd_trap("overflow", "find", 0ul);
        }
        const size_t t_4 = (size_t)(v_i + (size_t)1u);
        v_i = t_4;
    }
    return v_i;
}
```

The contract and the loop's `invariant`/`decreases` clauses produce no C.

## 10. Out of scope; how S3 drives the code

The slice has no entry point and no I/O, so emitted files contain neither. S3 tests drive them with a **harness**, outside the trusted path:

| Item | Rule |
|---|---|
| Harness | a hand-written C file per test that `#include`s the emitted file (one translation unit), defines `main`, builds arrays and slice structs that follow §6.3, calls `f_…` functions and prints results to `stdout` in a fixed format |
| Names | harness identifiers avoid the families of §7 (e.g. prefix `h_`) |
| Builds | each test is compiled with gcc, clang (`-std=c99 -Wall -Wextra -Werror`) and CompCert; with UBSan (`-fsanitize=undefined -fno-sanitize-recover`) where available, on a 64-bit and, if available, a 32-bit target |
| Pass | the three binaries print identical output; UBSan reports nothing; each trap test exits by `SIGABRT` with the expected `quod trap: <fn>/<kind>/<n>` line on `stderr` |
| Reports | trap sites in the C equal the unverified-checks report (§5.3) |

The test runner (Rust, in `compiler/tests/` or a new `cgen/tests/`) is untrusted test code.

## 11. Trusted-base summary

| Component | Trusted | Notes |
|---|---|---|
| This document | yes, if ADR-0010 is accepted | the subset and its assumptions |
| `cgen` | yes | ≤ 2,000 lines ([ADR-0010](../docs/adr/0010-trusted-c-emitter-from-core-ir.md)) |
| C compiler and C library | named in the trust report | §2.3 |
| Harness, test runner | no | |
