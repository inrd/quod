# Core IR — slice subset

> **Status: draft, provisional, non-normative.** Step S2b of the thin slice ([ADR-0006](../docs/adr/0006-thin-end-to-end-slice.md)). Built to Proposed [ADR-0007](../docs/adr/0007-trusted-obligation-generator.md) (trusted `coreir` and `vcgen`) and [ADR-0010](../docs/adr/0010-trusted-c-emitter-from-core-ir.md) (trusted `cgen`) under [ADR-0012](../docs/adr/0012-evidence-before-acceptance.md)'s rule for gated designs. If those ADRs are accepted, this document becomes part of the **trusted base** (Core IR semantics). §3 is implemented in `coreir/` and §8 in `compiler/src/lower.rs` (`quodc lower`); `vcgen` (§7) and `cgen` are not yet; the C that `cgen` will emit is specified in [c-subset.md](c-subset.md).

## 1. Role

```
source ──front end (untrusted)──► Core IR text ──coreir parser (trusted)──┬─► vcgen ─► obligations ─► kernel
                                   + source map (untrusted)                └─► cgen  ─► C
```

- Core IR is the **one artifact** that both trusted consumers read: the program that is verified and the program that is emitted are the same text.
- It has no sugar. Every operation that can fail is a separate, named step whose precondition is fixed by the operation's name (§5). Evaluation order is the order of statements.
- Spec expressions are explicit terms: lifting to `int` is written (`(int x)`), never implied.
- It covers the slice subset only ([contracts.md §7](contracts.md#7-slice-subset)). Anything else is rejected by lowering (§8).

## 2. Example

Source ([contracts.md §2](contracts.md#2-example)):

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
```

Core IR text:

```
(quod-core 0)

(fn find
  (params (xs (ref (slice u32))) (key u32))
  (ret usize)
  (requires)
  (ensures
    (<= (int result) (int (len xs)))
    (forall k 0 (int result) (!= (int (sel xs k)) (int key)))
    (=> (< (int result) (int (len xs))) (= (int (sel xs (int result))) (int key))))
  (body
    (var i usize 0usize)
    (loop
      (head
        (let $0 usize (len xs))
        (let $1 bool (lt.usize i $0)))
      (cond $1)
      (invariant (<= (int i) (int (len xs))))
      (invariant (forall k 0 (int i) (!= (int (sel xs k)) (int key))))
      (decreases (- (int (len xs)) (int i)))
      (body
        (let $2 u32 (load xs i))
        (let $3 bool (eq.u32 $2 key))
        (if $3
          (then
            (return i))
          (else))
        (let $4 usize (add.usize i 1usize))
        (set i $4)))
    (return i)))
```

## 3. Text form

S-expressions. `coreir` parses a file and prints it back; **the file is accepted only if the printed text is byte-identical to the input**. So every Core IR program has exactly one text, and the parser needs no separate canonicality rules.

### 3.1 Tokens

| Token | Form |
|---|---|
| Parentheses | `(` `)` |
| Name | source identifier (`i`, `find`); a renamed local `name.N` (N ≥ 1); a temporary `$N` (N ≥ 0); decimal, no leading zeros |
| Machine literal | decimal digits plus type suffix: `0usize`, `4294967295u32`; no leading zeros, no `_` |
| `int` literal | decimal, optional leading `-`: `0`, `-1` |
| Keyword / operator | the fixed words in §3.3–§5 |

No comments. ASCII only. One space between tokens on a line; no space after `(` or before `)`.

### 3.2 Layout

- Line 1 is `(quod-core 0)` (format version). Each function follows after one blank line. The file ends with one newline.
- **Block forms** put each child on its own line, indented two spaces more than the form's own line: `fn` (after its name), `requires`, `ensures`, `body`, `then`, `else`, `head`, `loop`, and `if` (after its condition). An empty block form is printed on one line: `(requires)`, `(else)`.
- Every other form is printed on one line.
- Closing parentheses follow the last child on its line, never on a line of their own.

### 3.3 Grammar

```ebnf
file      = "(quod-core 0)" { fn } ;
fn        = "(fn" NAME "(params" { param } ")" "(ret" rtype ")"
            "(requires" { term } ")" "(ensures" { term } ")"
            "(body" { stmt } ")" ")" ;
param     = "(" NAME ptype ")" ;
ptype     = scalar | "(ref (slice" scalar "))" | "(mut (slice" scalar "))" ;
rtype     = scalar | "unit" ;
scalar    = "u32" | "u64" | "usize" | "bool" ;
ltype     = scalar | "(array" scalar NAT ")" ;

stmt      = "(let" NAME vtype rhs ")"            (* immutable *)
          | "(var" NAME ltype rhs ")"            (* mutable local *)
          | "(set" NAME atom ")"
          | "(store" NAME atom atom ")"          (* base, index, value *)
          | "(call" NAME { arg } ")"              (* callee returns unit *)
          | "(if" atom "(then" { stmt } ")" "(else" { stmt } ")" ")"
          | "(loop" "(head" { stmt } ")" "(cond" atom ")"
                    { "(invariant" term ")" } "(decreases" term ")"
                    "(body" { stmt } ")" ")"
          | "(return" [ atom ] ")" ;
vtype     = ltype | "(ref (slice" scalar "))" | "(mut (slice" scalar "))" ;
atom      = NAME | MACHINE_LIT | "true" | "false" ;
arg       = atom ;
rhs       = atom
          | "(" BINOP atom atom ")" | "(" UNOP atom ")"
          | "(len" NAME ")" | "(load" NAME atom ")"
          | "(array" scalar atom { atom } ")"
          | "(view" NAME atom atom ")" | "(mview" NAME atom atom ")"
          | "(call" NAME { arg } ")" ;

term      = NAME | "result" | MACHINE_LIT | INT_LIT | "true" | "false"
          | "(int" term ")" | "(len" term ")" | "(sel" term term ")"
          | "(old" term ")"
          | "(" SPEC_OP term term ")" | "(not" term ")"
          | "(" BITOP term term ")" | "(" BITNOT term ")"
          | "(" ( "forall" | "exists" ) NAME term term term ")" ;   (* binder, lo, hi, body *)
```

`BINOP` and `UNOP` are the `op.T` names of §5; `BITOP` is `band.T bor.T bxor.T shl.T shr.T` and `BITNOT` is `bnot.T` (§6); `SPEC_OP` is one of `+ - * / % = != < <= > >= and or =>`.

### 3.4 Well-formedness (checked by `coreir`)

`coreir` rejects a parsed program that breaks any of these; they are what the trusted consumers may rely on:

- Names are unique per function (parameters, locals, temporaries); a quantifier binder does not shadow any name in scope, but separate quantifiers may reuse a binder name; every name is defined before use; function names are unique and every call names a function in the file.
- Every operand has the type its operation requires (§5), with no conversions. `let`/`var` types match their right-hand side.
- `set` targets a `var` of scalar type; `store` targets a `var` array or a `mut` slice parameter.
- Values of `ref`/`mut` slice type are defined only by `view`/`mview` (or are parameters) and are used only as call arguments, `len`, `load` and `store` bases.
- `head` statements contain no `call`, `store` or `return`, and `set` only `var`s declared in the same `head` (code `&&`/`||` in a loop condition lowers to such a `var`).
- Every path through a function body ends in `return`; `(return)` exactly when the return type is `unit`. No statement follows a `return` in the same list.
- Proposed by [ADR-0014](../docs/adr/0014-alias-free-views-in-core-ir.md), not yet checked: no `mut` slice argument shares its root with another argument of the same call, and each view is used once, as a call argument in its own statement list, with no write to its root in between. `cgen`'s in-place `mut` slices rely on both ([c-subset.md §6.3](c-subset.md#63-mut-slices-in-place-and-the-calling-convention)).
- Terms are well-typed (§6); `result` occurs only in `ensures` of a non-`unit` function; `old` only in `ensures`, `invariant`, `decreases`, never nested, and its argument names only parameters and binders.

## 4. Types and values

| Type | Values | Obligation encoding (ADR-0007) |
|---|---|---|
| `u32`, `u64` | `0 ≤ x < 2^N` | `Int` with range hypothesis |
| `usize` | `0 ≤ x < 2^64` on some target; results must fit `< 2^32` ([ADR-0005](../docs/adr/0005-portable-static-checks-for-pointer-sized-integers.md)) | `Int` with hypothesis `0 ≤ x < 2^64` |
| `bool` | `true`, `false` | `Bool` |
| `(array T N)` | `N` values of `T` (locals only) | array term `a : Array(Int, T)`, length `N` |
| `(ref (slice T))`, `(mut (slice T))` | view of a base sequence: base, offset, length | base array, offset term, length term (§7.3) |
| `int` | all integers (terms only) | `Int` |

Arrays and slices have value semantics in obligations. A `mut` slice parameter behaves as copy-in/copy-out ([ADR-0011](../docs/adr/0011-v1-scope.md)); since `&mut` arguments never alias ([typing.md §8.4](typing.md#84-aliasing-at-calls)), `cgen` may implement it in place ([c-subset.md §6.3](c-subset.md#63-mut-slices-in-place-and-the-calling-convention), which also needs [ADR-0014](../docs/adr/0014-alias-free-views-in-core-ir.md)).

## 5. Code operations

Each operation is one site: one runtime trap during phase 1 (`cgen`), one obligation from phase 2 on (`vcgen`). `T` ranges over `u32`, `u64`, `usize`.

| Operation | Operands → result | Precondition (obligation kind) |
|---|---|---|
| `add.T` `sub.T` `mul.T` | `T T → T` | exact result in the portable range of `T` (`overflow`) |
| `div.T` `rem.T` | `T T → T` | divisor `≠ 0` (`div-zero`); truncates toward zero |
| `shl.T a n` | `T T → T` | `n < bits(T)` and `a·2^n` in the portable range of `T` (`shift`) |
| `shr.T a n` | `T T → T` | `n < bits(T)` (`shift`) |
| `band.T` `bor.T` `bxor.T` | `T T → T` | none |
| `not.u32`, `not.u64` | `T → T` | none (`not.usize` is outside the slice) |
| `not.bool` | `bool → bool` | none |
| `eq.T` `ne.T` `lt.T` `le.T` `gt.T` `ge.T` | `T T → bool` | none |
| `eq.bool` `ne.bool` | `bool bool → bool` | none |
| `(len a)` | array or slice → `usize` | none |
| `(load a i)` | `i: usize` → element | `i < len(a)` (`bounds`) |
| `(store a i v)` | statement | `i < len(a)` (`bounds`) |
| `(view a lo hi)`, `(mview a lo hi)` | `lo hi: usize` → slice | `lo ≤ hi ≤ len(a)` (`slice`) |
| `(array T a…)` | → `(array T n)` | none |
| `(call f a…)` | → return type | each `requires` clause of `f`, instantiated (`call-requires`) |

`bits(usize)` is 32 for this rule and the portable range of `usize` is `[0, 2^32)`, as in [typing.md §11](typing.md#11-runtime-traps-and-the-unverified-checks-report).

**Phase-1 trap elision.** `cgen` omits a trap exactly when typing.md §11 calls the check *statically trivial*, applied to the operation's atoms: every operand the check reads is a literal atom (and the check passes), or `load`/`store`/`view` on an `array` with literal indices in range. Lowering keeps literals as atoms and folds nothing, so the trap sites in the C equal the unverified-checks report. This equality is an S3 test.

## 6. Spec terms

| Term | Type | Meaning |
|---|---|---|
| machine literal, `int` literal, `true`, `false` | its type | |
| `NAME` | type of the parameter, local or binder | its current value; in `ensures`, the value at return |
| `result` | return type | returned value |
| `(int t)` | `int`; `t` a machine integer | the same number (ADR-0007: `t` itself) |
| `(len s)` | `usize`; `s` a sequence term | length |
| `(sel s k)` | element type; `k: int` | element `k` of `s` (indices relative to the view) |
| `(old t)` | type of `t` | `t` in the function's entry state |
| `(+ a b)` `(- a b)` `(* a b)` `(/ a b)` `(% a b)` | `int int → int` | exact; `/` and `%` truncate toward zero |
| `(= a b)` `(!= a b)` | both `int` or both `bool` | |
| `(< a b)` `(<= a b)` `(> a b)` `(>= a b)` | `int int → bool` | |
| `(and a b)` `(or a b)` `(=> a b)` `(not a)` | `bool` | |
| `(band.T a b)` `(bor.T …)` `(bxor.T …)` `(shl.T …)` `(shr.T …)` `(bnot.T a)` | `T T → T` | bit-vector meaning; `shl` drops high bits |
| `(forall k lo hi body)`, `(exists …)` | `bool`; `lo hi: int`; `k: int` in `body` | `k` ranges over `lo ≤ k < hi` |

**Well-formedness conditions** (obligation kind `wf`): `(sel s k)` needs `0 ≤ k < len(s)`; `/` and `%` need a non-zero divisor; `shl.T`/`shr.T` need the amount `< bits(T)`. Each is checked under the guards of [contracts.md §5.2](contracts.md#52-operators) (`and`, `=>`, `or`, quantifier range) and after the earlier clauses listed there.

## 7. Obligations (`vcgen`)

An obligation is a closed kernel formula `H₁ ∧ … ∧ Hₙ → G` ([ADR-0008](../docs/adr/0008-kernel-rules-from-real-certificates.md) term language). `vcgen` computes it by forward symbolic execution of the body over a **state**: a term for each local in scope, the entry-state terms (for `old`), and the **path facts** (hypotheses collected so far).

### 7.1 Function entry

- Each parameter gets a fresh constant; scalars get their range hypothesis (§4). A slice parameter `p` gets a fresh base array `p`, offset `0` and a fresh length `p_len` with `0 ≤ p_len < 2^64`, and the element range hypothesis `∀k. 0 ≤ k < p_len → range(select(p, k))`.
- Entry-state terms are the parameter terms.
- `wf` obligations for each `requires` clause, in order; then each clause is a path fact.
- `wf` obligations for each `ensures` clause, once per function: under the `requires` facts and the earlier `ensures` clauses, with `result` and the final contents of each `mut` parameter as fresh constants (with range hypotheses). Lengths do not change, so `sel` bounds are the same as at entry.

### 7.2 Statements

| Statement | Effect on the state; obligations |
|---|---|
| `let`/`var x T rhs` | obligation for `rhs`'s precondition (§5) under the current path facts; `x` ↦ term of `rhs` |
| `set x a` | `x` ↦ term of `a` |
| `store a i v` | `bounds` obligation; base of `a` ↦ `store(base, offset + i, v)` |
| `if c (then S) (else T)` | run `S` with fact `c`, `T` with fact `¬c`; afterwards each local that differs is `ite(c, s, t)`. A branch that ends in `return` contributes nothing after the `if` |
| `return a` | one `ensures` obligation per clause, with `result` ↦ `a`, under the current path facts |
| `call f args` | §7.3 |
| `loop` | §7.4 |

Path facts are never weakened along a path. Obligations are generated in statement order; within a site, in the order of §5's precondition column.

### 7.3 Calls

For `(call f a₁ … aₙ)` with the caller's state:

1. Bind each parameter of `f` to its argument. A slice argument (a parameter passed on, or a `view`/`mview`) is a view: base, offset, length (`hi - lo` for views; a view of a view adds offsets).
2. One `call-requires` obligation per `requires` clause of `f`, instantiated.
3. For each `mut` view argument: a fresh base array `b'`, with the frame hypothesis that `b'` equals the old base outside the view (`∀m. 0 ≤ m < len(base) ∧ (m < off ∨ m ≥ off + len) → select(b', m) = select(b, m)`) and the element range hypothesis. The caller's base ↦ `b'`.
4. A fresh constant for the result (non-`unit` callee) with its range hypothesis.
5. Each `ensures` clause of `f`, instantiated with `result` ↦ the result constant and `old(·)` ↦ the pre-call argument terms, becomes a path fact.

Sequence equality is never encoded with array `=` (ADR-0007).

### 7.4 Loops

For `(loop (head H) (cond c) (invariant I₁) … (decreases D) (body B))`:

1. **Init:** `invariant-init` obligations `I₁ … Iₖ` in the current state (with their `wf` obligations).
2. **Frame:** the locals `set`, the bases `store`d to, and the bases of `mview` arguments anywhere in `B`. Each gets a fresh constant (with range hypotheses); the rest keep their terms.
3. **Iteration** (from the havocked state with facts `I₁ … Iₖ`): run `H` (its sites give obligations); with fact `c`: evaluate `D` to `d₀` (`wf` obligations), obligation `0 ≤ d₀` (`decreases-bound`); run `B`; at the end of `B`, `invariant-preserve` obligations `I₁ … Iₖ` and `D < d₀` (`decreases-step`). Returns inside `B` produce `ensures` obligations as usual.
4. **Exit:** continue after the loop from the havocked state with facts `I₁ … Iₖ`, the facts from running `H`, and `¬c`.

`old` in `I` and `D` reads the entry-state terms.

### 7.5 Cut rule ([ADR-0013](../docs/adr/0013-annotations-and-small-obligations.md))

Obligations are cut at function boundaries (a call is known only through its contract, §7.3) and at loops (a loop is known only through its invariants, §7.4). Each obligation contains only the path facts of its own site. `assert` (outside the slice) will add one more cut.

### 7.6 Obligation IDs

ID: `<fn>/<kind>/<n>`, where `n` counts the function's obligations of that kind from 0 in generation order. Kinds: `overflow`, `div-zero`, `shift`, `bounds`, `slice`, `call-requires`, `ensures`, `invariant-init`, `invariant-preserve`, `decreases-bound`, `decreases-step`, `wf`.

> **Speculative:** these IDs are stable under edits to other functions but not under edits inside the function that add or remove earlier sites of the same kind. Phase 3 requires stability under a defined set of unrelated edits; this scheme is a starting point to measure in S6. Fingerprints (hashes of the obligation formula) are cache keys for stored certificates. They are not trusted: a certificate is always checked against the obligation `vcgen` generates.

## 8. Lowering (front end, untrusted)

| Source | Core IR |
|---|---|
| local `x` | `x`, or `x.N` if an earlier local of this function had the same name (sibling scopes) |
| any subexpression that is an operation | a fresh `(let $N T …)`, in left-to-right evaluation order ([typing.md §5](typing.md#5-expressions)) |
| integer literal | machine literal with its checked type, in decimal |
| `a && b` (code) | `<a>`, `(var $r bool a)`, `(if a (then <b> (set $r b)) (else))`; for `\|\|` the branches swap |
| `place = e` | index of `place` (if any), then `e`, then `set`/`store` |
| `place op= e` | index of `place`, `load` (or the local), `e`, the operation, `set`/`store` |
| `&p[lo..hi]`, `&mut p[lo..hi]` | `lo`, `hi`, then `(let $N … (view/mview p lo hi))` at that argument's position |
| `while c … { B }` | `(loop (head <c>) (cond <c's atom>) … (body <B>))` |
| `if` without `else` | `(else)` |
| a `()` function that can fall off its end | `(return)` appended |
| spec expression | term, with `(int ·)` around every machine-integer operand of `+ - * / %`, comparisons and quantifier bounds, and around a machine-integer `sel` index; `==>` is `=>`; `xs[k]` is `(sel xs k)`; `xs.len` is `(len xs)` |

Lowering does not fold constants, reorder operations or drop operations whose result is unused.

**Outside the slice.** A construct that type-checks but is outside the slice subset ([contracts.md §7](contracts.md#7-slice-subset)) is rejected by lowering with `E0401` ([diagnostics.md](diagnostics.md#lowering)). In addition to the table in contracts.md §7, the slice requires that a `while` condition contains no calls (§3.4: `head` has no `call`).

**Source map.** Lowering also writes a source map from each Core IR site (function, kind, ordinal) to its source span. It is untrusted and not part of the Core IR text, so source positions never change the text or obligation IDs.

## 9. Trusted-base summary

| Component | Trusted | Budget |
|---|---|---|
| This document (§3–§7) | yes, if ADR-0007/0010 are accepted | — |
| `coreir` (types, parser, printer, §3.4 checks) | yes | ≤ 1,500 lines |
| `vcgen` (§7) | yes | ≤ 3,000 lines |
| `cgen` (§5, [C subset](c-subset.md)) | yes | ≤ 2,000 lines |
| Lowering, source map, fingerprints | no | — |
