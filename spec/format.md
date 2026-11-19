# Canonical format

> **Status: provisional, non-normative.** Describes what `quodc fmt` emits in phase 1. Changes with the grammar.

The formatter defines layout; whitespace is not significant to the AST. Required properties (phase-1 exit criteria): `format(format(x)) == format(x)` and `parse(format(x)) == parse(x)` (AST equality ignoring spans and comments).

## Layout rules

| Construct | Canonical form |
|---|---|
| Indentation | 4 spaces; no tabs; no trailing whitespace; file ends with one newline |
| Items | separated by exactly one blank line |
| `fn` | signature on one line; `effects { … }` on the next line, indented once; body `{` on its own line |
| `type` | one field per line, each followed by `,` |
| Blocks | one statement per line; empty block is `{}` |
| `if` / `else` | `} else {` and `} else if c {` on the closing-brace line |
| Expressions | always on one line (no width-based wrapping in phase 1); binary operators surrounded by one space; no space inside `()`/`[]`; range `a..b` without spaces |
| Struct literal | `Name.{ f: e, g: e }`; `Name.{}` is not valid (at least one field) |
| Array literal | `[a, b, c]` |
| References, slices | `&x`, `&mut x`, `& &x` (space keeps `&&` from lexing as one token), `a[lo..hi]` without spaces |
| Effects | `effects {}` or `effects { a, b }` |
| Parentheses | kept as written; the formatter never adds or removes them |
| Integer literals | kept as written (base, underscores, suffix) |
| Blank lines inside blocks | removed, except after a comment that was followed by a blank line |

## Comments

Comments are not part of the AST, but the formatter never drops one.

- A comment on its own line is attached as a *leading* comment to the next item, field or statement and printed above it at its indentation.
- A comment after code on the last line of an item, field or statement is a *trailing* comment, printed after it with one space: `x = 1; // note`.
- A comment after an opening `{` on the same line stays there: `for i: u32 in 0..n { // note`.
- Any other comment inside a multi-line statement is moved above that statement.
- Comments after the last item are separated from it by one blank line.
- Comments before a closing `}` with no following statement stay at the end of the block.
- A comment followed by a blank line keeps one blank line after it (e.g. a file header).
- Comment text after `//` is kept verbatim except trailing whitespace.
