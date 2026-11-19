# Diagnostic codes

> **Status: provisional.** The numbering scheme and messages may change until an ADR accepts them. Once accepted, codes are stable across versions and retired codes are never reused ([oracle-protocol.md](../docs/oracle-protocol.md#diagnostics)).

Format: `E` + 4 digits. The first two digits name the stage.

| Range | Stage |
|---|---|
| `E01xx` | lexer |
| `E02xx` | parser |
| `E03xx` | name resolution and type checking (not yet assigned) |
| `E04xx` | C emission (not yet assigned) |

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
| `E0202` | Reserved keyword (phase 2+), e.g. `requires`, `where`, `invariant`. |
| `E0203` | Different arithmetic operators mixed in one chain without parentheses (`a + b * c`, `a + b - c`). |
| `E0204` | Chained comparison (`a < b < c`). |
| `E0205` | `&&` and `\|\|` mixed without parentheses. |
| `E0206` | Keyword valid but construct not available in phase 1 (e.g. `while`). |
