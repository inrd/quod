# Contracts, loops and spec contexts — slice subset

> **Status: draft, provisional, non-normative.** Step S2 of the thin slice ([ADR-0006](../docs/adr/0006-thin-end-to-end-slice.md)). Syntax follows [grammar-draft.md](grammar-draft.md); typing extends [typing.md](typing.md). Arithmetic in spec contexts is from [ADR-0002](../docs/adr/0002-arithmetic-semantics.md) (accepted). Everything else is a proposal. Implemented in `quodc` (parser, formatter, type checker); lowering and verification follow in later slice steps.

## 1. Scope

This document adds the constructs the slice needs to state and check functional correctness:

| Construct | Where | Section |
|---|---|---|
| `requires e`, `ensures e` | after `effects { … }` on a `fn` | §3 |
| `while c invariant e … decreases e { … }` | statement | §4 |
| `result`, `old(e)` | spec contexts | §5.3 |
| `forall k: int in lo..hi { e }`, `exists …` | spec contexts | §5.4 |
| `a ==> b` | spec contexts | §5.2 |
| `int` | quantifier binders only | §5.1 |

It also fixes the **slice subset** (§7): the part of the language that S2b–S6 lower to Core IR, emit as C and verify.

Not in this document: `assert`, `assume`, `spec fn`, `lemma`, ghost code, `where`, `nat`, `x as int`, `int(e)`, `decreases` on functions (no recursion, [typing.md §9](typing.md#9-functions)), sequence equality. They stay reserved.

This draft does not settle [open-questions §4](../docs/open-questions.md#4-spec-language-expressiveness): the slice has only range-bounded quantifiers because it needs nothing more, not because §4 is decided.

## 2. Example

```
pub fn find(xs: &[u32], key: u32) -> usize
    effects {}
    ensures result <= xs.len
    ensures forall k: int in 0..result { xs[k] != key }
    ensures result < xs.len ==> xs[result] == key
{
    let mut i: usize = 0;
    while i < xs.len
        invariant i <= xs.len
        invariant forall k: int in 0..i { xs[k] != key }
        decreases xs.len - i
    {
        if xs[i] == key {
            return i;
        }
        i = i + 1;
    }
    return i;
}

pub fn fill(dst: &mut [u32], v: u32) -> ()
    effects {}
    ensures forall k: int in 0..dst.len { dst[k] == v }
{
    let mut i: usize = 0;
    while i < dst.len
        invariant i <= dst.len
        invariant forall k: int in 0..i { dst[k] == v }
        decreases dst.len - i
    {
        dst[i] = v;
        i = i + 1;
    }
}
```

`find` returns `xs.len` when `key` is absent. The slice has no `Option` (no structs, no generics).

## 3. Function contracts

```ebnf
fn_item = "fn" IDENT "(" [ param { "," param } ] ")" "->" type
          effects_clause
          { "requires" expr }
          { "ensures" expr }
          block ;
```

- All `requires` come before all `ensures` (the grammar enforces it). Zero or more of each. Several clauses mean their conjunction; each clause is its own obligation and report entry.
- Each clause is a **spec context** (§5) of type `bool` (`E0303` otherwise).
- **Names in scope:** the parameters; in `ensures`, also `result` (§5.3). Locals of the body are not in scope.
- **Meaning:** the caller must establish every `requires` at the call; the body may assume them. The body must establish every `ensures` at every `return` and, for a `()` function, at the end of the body. A caller learns about the call only through `ensures` (compositional verification).
- `requires` is evaluated in the state at entry. `ensures` is evaluated in the state at return: a `&mut` parameter's elements are their final values; `old(e)` gives values at entry.

## 4. `while` loops

```ebnf
while_stmt = "while" expr { "invariant" expr } "decreases" expr block ;
```

| Part | Context | Type | Meaning |
|---|---|---|---|
| condition `c` | code | `bool` | evaluated before each iteration; may trap ([typing.md §11](typing.md#11-runtime-traps-and-the-unverified-checks-report)) |
| `invariant e` | spec | `bool` | holds before the first evaluation of `c` and after each iteration that ends normally (end of body or `continue`) |
| `decreases d` | spec | any integer type, lifted to `int` | exactly one; `0 <= d` when the body starts, and `d` at the end of the iteration is less than `d` at its start |

- **Names in scope:** locals visible at the loop, and the parameters. Locals declared in the body are not in scope in the clauses.
- `old(e)` in an `invariant` or `decreases` refers to the **function's** entry state, not the loop's.
- After the loop, code may rely on `invariant ∧ ¬c` when the loop exits through `c`. A `break` exit carries no invariant. Exactly which locals a loop may change (its frame) is `vcgen` behavior, defined in [core-ir.md §7.4](core-ir.md#74-loops).
- `decreases` is mandatory ([design-principles §4](../docs/design-principles.md), [Power of 10 rule 2](../docs/power-of-10-mapping.md)): every loop terminates by proof, not by the bounded `for` construct.
- `break` and `continue` are allowed in a `while` body ([typing.md §6](typing.md#6-statements-and-control-flow)); they are outside the slice subset (§7).
- A `while` never *always returns* for the definite-return rule, and a statement after it is reachable.

## 5. Spec contexts

A **spec context** is the expression of a `requires`, `ensures`, `invariant` or `decreases` clause, including everything nested in it (ADR-0002: the enclosing keyword decides, never inference). Everything else is code.

### 5.1 Types and literals

| Rule | Detail |
|---|---|
| `int` | unbounded integers. In the slice it is written only as a quantifier binder type (§5.4); in code it is `E0326` |
| Arithmetic lifts | `+ - * / %` on any integer types (machine or `int`, mixed widths allowed) lift both operands to `int` and give `int`. No overflow. `/` and `%` truncate toward zero |
| Comparisons lift | `== != < <= > >=` on integer operands compare mathematical values, for any mix of integer types (`i + 1 < xs.len` with `i: usize` compares `int` with `usize`) |
| Unsuffixed literals | `int`, except as an operand of a bitwise operator (next row) |
| Bitwise keeps machine types | `& \| ^ << >>` and integer `!` need operands of one machine type `T`, give `T` with its bit-vector meaning (`<<` drops high bits). An unsuffixed literal operand is checked against the other operand's type ([typing.md §4.2](typing.md#42-binary-operators-without-an-expected-type)). `int` operands are `E0306` |
| `bool` | `&& \|\| !` and `==`/`!=` as in code |
| No conversions | no `as` in the slice; `x as int` and `int(e)` are not available yet |

Comparison across machine widths is a deliberate extension of ADR-0002's rule (`int` vs machine type): since arithmetic already lifts, `i + 1 < n` and `i < n` must mean the same kind of comparison.

### 5.2 Operators

| Expression | Rule | Result |
|---|---|---|
| `a ==> b` | `a`, `b`: `bool`; lowest precedence; not associative (`a ==> b ==> c` is a syntax error) | `bool` |
| `a && b`, `a \|\| b` | as in code | `bool` |
| `e[i]` | `e`: array or slice (one reference level, [typing.md §8.2](typing.md#82-using-references)); `i`: any integer type | element type |
| `e.len` | as in code | `usize` |
| `?h` | as in code: records the expected type, then `E0321` | — |

Not available in spec contexts: calls (`E0329`; there is no `spec fn` yet), `&`/`&mut` (`E0319`, since they only appear as call arguments), `e[lo..hi]` (`E0325`), casts, array and struct literals (`E0318`). A literal index that is negative, or not below a fixed array's length, is `E0305`, as in code.

**Well-formedness.** A spec expression must be defined where it is evaluated: every index is in bounds, every divisor is non-zero, and `<<`/`>>` amounts are below `bits(T)`. These are spec well-formedness obligations (ADR-0002), not traps. They are checked under the guards of the expression: in `a && b` and `a ==> b`, `b` may assume `a`; in `a || b`, `b` may assume `!a`; a quantifier body may assume its range. So `i < xs.len && xs[i] == 0` is well formed.

Clauses are checked for well-formedness in order, each under the clauses before it:

| Clause | May assume |
|---|---|
| `requires` | earlier `requires` |
| `ensures` | all `requires`, earlier `ensures` |
| `invariant` | earlier `invariant`s of the same loop |
| `decreases` | all `invariant`s of the loop and its condition |

This is why `find` and `fill` (§2) state `i <= xs.len` before the quantified invariant that indexes up to `i`.

### 5.3 `result` and `old`

| Expression | Allowed in | Rule |
|---|---|---|
| `result` | `ensures` of a function whose return type is not `()` (`E0328`) | the returned value; type is the return type |
| `old(e)` | `ensures`, `invariant`, `decreases`; not in `requires`, not nested; `e` reads only parameters and quantifier binders (`E0327`) | `e` evaluated in the function's entry state; type of `e` |

Parameters are immutable and arrays are values, so `old(e)` differs from `e` only when `e` reads through a `&mut` parameter (`old(dst[k])`). `old(dst.len)` always equals `dst.len`.

### 5.4 Quantifiers

```ebnf
quant = ( "forall" | "exists" ) IDENT ":" type "in" expr ".." expr "{" expr "}" ;
```

- `forall k: int in lo..hi { e }` means: for every integer `k` with `lo <= k < hi`, `e`. `exists` is the dual. The range is empty when `lo >= hi`.
- The binder type must be `int` (`E0318` for any other type). `lo` and `hi` are spec expressions of any integer type, lifted; the body has type `bool`.
- The binder follows the local naming rules ([typing.md §3](typing.md#3-names-and-scopes)): no shadowing (`E0302`); its scope is the body only.
- Only range-bounded quantifiers exist: every quantifier states its range in one place, which keeps solver triggers simple (ADR-0007 quantifies over base indices) and renders directly in the human report ("for every `k` from `lo` to `hi - 1`").

## 6. Phase-1 behavior

Until `vcgen` exists (S4a), contracts are type-checked but not proven:

- The C emitter **erases** every clause. Clauses are never executed: they use `int` and quantifiers, which have no runtime meaning, and a runtime check would differ from the proven meaning.
- The loop condition and body run with phase-1 traps as usual.
- Every clause is an entry in the **unverified checks** report ([typing.md §11](typing.md#11-runtime-traps-and-the-unverified-checks-report)), with kind `requires`, `ensures`, `invariant` or `decreases`. A `while` loop is not guaranteed to terminate until its `decreases` entry is proven.

## 7. Slice subset

The constructs that slice steps S2b–S6 lower, emit and verify. Every other construct still parses and type-checks as in [typing.md](typing.md); lowering to Core IR rejects it with `E0401` ([core-ir.md §8](core-ir.md#8-lowering-front-end-untrusted)).

| Area | In the slice | Not in the slice |
|---|---|---|
| Scalar types | `u32`, `u64`, `usize`, `bool`, `()` (return type only) | `u8`, `u16`, signed types, `isize` |
| Compound types | `[T; N]` locals with `T` a slice scalar; `&[T]`, `&mut [T]` parameters | structs, `&T`, `&mut T`, `&[T; N]`, arrays as parameters or return values |
| Items | `fn` with `requires`/`ensures` | `type` |
| Statements | `let`, `=`, `+=`/`-=`/`*=`, `if`/`else`, `while` (condition without calls), `return`, call statement, block | `for`, `break`, `continue` |
| Code expressions | literals, locals, `+ - * / %`, `& \| ^ << >>`, `!`, comparisons, `&& \|\|`, `e[i]`, `e.len`, `&mut p[lo..hi]`, `&p[lo..hi]`, passing a slice parameter on, calls, array literals | casts, unary `-`, struct literals, field access |
| Spec expressions | §5, with the code expressions above except calls | — |

Width-dependent operations that [ADR-0007](../docs/adr/0007-trusted-obligation-generator.md#encodings) does not encode yet are also outside the slice: `!` on `usize`, and `<<` on `usize` in spec contexts (its bit-vector meaning depends on the target width).

## 8. Grammar and diagnostics changes

Grammar changes are in [grammar-draft.md](grammar-draft.md) (§2, §4, §5). New diagnostic codes ([diagnostics.md](diagnostics.md#type-checker)):

| Code | Meaning |
|---|---|
| `E0326` | Spec-only construct in code (`old`, `result`, `forall`, `exists`, `==>`, type `int`). |
| `E0327` | `old` in `requires`, nested `old`, or `old` reading a local that does not exist at function entry. |
| `E0328` | `result` outside `ensures`, inside `old`, or in a function returning `()`. |
| `E0329` | Call in a spec context. |

`E0318` also covers a quantifier binder of a type other than `int`. `while` is no longer `E0206` (that code is retired), and `requires`, `ensures`, `invariant`, `decreases`, `old`, `result`, `forall`, `exists` are no longer `E0202`.
