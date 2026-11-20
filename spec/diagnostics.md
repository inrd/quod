# Diagnostic codes

> **Status: provisional.** The numbering scheme and messages may change until an ADR accepts them. Once accepted, codes are stable across versions and retired codes are never reused ([oracle-protocol.md](../docs/oracle-protocol.md#diagnostics)).

Format: `E` + 4 digits. The first two digits name the stage.

| Range | Stage |
|---|---|
| `E01xx` | lexer |
| `E02xx` | parser |
| `E03xx` | name resolution and type checking ([typing.md](typing.md)) |
| `E04xx` | lowering to Core IR ([core-ir.md](core-ir.md); not yet assigned) |
| `E05xx` | C emission (not yet assigned) |

Phase 1 stops at the first error in each file (no recovery).

## Lexer

| Code | Meaning |
|---|---|
| `E0101` | Unexpected character (anything outside the lexical grammar, including non-ASCII outside comments). |
| `E0102` | Malformed integer literal: bad digit for its base, empty digits after `0x`/`0b`, unknown suffix, or value above `2^64 - 1`. |
| `E0103` | `?` not followed by an identifier (malformed hole). |

## Parser

| Code | Meaning |
|---|---|
| `E0201` | Unexpected token: the parser expected something else. |
| `E0202` | Reserved keyword (phase 2+), e.g. `assert`, `where`, `lemma`. |
| `E0203` | Different arithmetic operators mixed in one chain without parentheses (`a + b * c`, `a + b - c`). |
| `E0204` | Chained comparison (`a < b < c`). |
| `E0205` | `&&` and `\|\|` mixed without parentheses. |
| `E0206` | Retired (was: `while` not available in phase 1). Never emitted; the number is not reused. |

## Type checker

Specified in [typing.md](typing.md); implemented in `compiler/src/check.rs`.

| Code | Meaning |
|---|---|
| `E0301` | Unknown name (type, function or local). |
| `E0302` | Duplicate definition or shadowing (item, built-in type name, field, parameter, local). |
| `E0303` | Type mismatch. |
| `E0304` | Unsuffixed integer literal (or hole) with no expected type. |
| `E0305` | Constant out of range: literal does not fit its type, or a statically trivial check fails (literal index ≥ `N`, literal overflow). |
| `E0306` | Operator not defined for the operand type (e.g. `-` on unsigned, `<` on `bool`, `&` on `bool`, indexing a non-array). |
| `E0307` | Invalid cast (`as` with a non-integer type). |
| `E0308` | Not assignable (immutable local, parameter, `for` variable, `.len`, through `&`). |
| `E0309` | Wrong number of arguments. |
| `E0310` | Call target is not a function. |
| `E0311` | Unknown field. |
| `E0312` | Struct literal fields missing, duplicated or out of declaration order. |
| `E0313` | Missing return on some path. |
| `E0314` | Unreachable statement. |
| `E0315` | `break`/`continue` outside a loop. |
| `E0316` | Expression statement is not a call returning `()`. |
| `E0317` | Recursion (not available in phase 1). |
| `E0318` | Construct not available in phase 1 (generics, `mod`/`use`, non-empty effects, `nat`, named array length; in spec contexts: casts, array and struct literals, quantifier binders not of type `int`). |
| `E0319` | Reference type outside a parameter type. |
| `E0320` | `&mut` argument aliases another argument of the same call. |
| `E0321` | Unfilled hole. |
| `E0322` | Invalid array length (zero, too large, or non-`usize` suffix). |
| `E0323` | Array literal length does not match the expected type. |
| `E0324` | Struct contains itself by value. |
| `E0325` | Range index `p[lo..hi]` not directly under `&`/`&mut`. |
| `E0326` | Spec-only construct in code: `old`, `result`, `forall`, `exists`, `==>`, type `int` ([contracts.md](contracts.md)). |
| `E0327` | `old` in `requires`, nested `old`, or `old` reading a local that does not exist at function entry. |
| `E0328` | `result` outside `ensures`, inside `old`, or in a function returning `()`. |
| `E0329` | Call in a spec context (no `spec fn` yet). |

