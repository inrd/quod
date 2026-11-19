# compiler/ — `quodc`

Rust ([ADR-0001](../docs/adr/0001-implementation-language.md)), member of the Cargo workspace at the repo root. Untrusted (see [../docs/architecture.md](../docs/architecture.md)). Arithmetic semantics follow [ADR-0002](../docs/adr/0002-arithmetic-semantics.md).

## Status (phase 1)

| Module | Done | Spec |
|---|---|---|
| `lexer.rs` | yes | [grammar-draft §1](../spec/grammar-draft.md) |
| `parser.rs` (recursive descent, one-token decisions, stops at first error) | yes | [grammar-draft §2–6](../spec/grammar-draft.md) |
| `dump.rs` (span- and comment-free S-expression, used for AST equality) | yes | — |
| `fmt.rs` (canonical formatter, keeps every comment) | yes | [format.md](../spec/format.md) |
| `diag.rs` (stable codes) | yes | [diagnostics.md](../spec/diagnostics.md) |
| Name resolution, type checker | no | not yet written down |
| Runtime traps + "unverified checks" report | no | [ADR-0002](../docs/adr/0002-arithmetic-semantics.md#phase-1-transition) |
| C emitter | no | not yet written down |
| Mechanical LL(1) check of the grammar | no | [grammar-draft §8](../spec/grammar-draft.md) |

## CLI

```
quodc fmt [--check | --write] <file>...
quodc parse <file>
```

Exit codes: `0` ok, `1` diagnostics or non-canonical files, `2` usage or I/O error.

## Later scope

- Effect checker, ownership/permission checker (phases 4, 5)
- Core IR and proof obligation generator with stable IDs (phase 2)
- Optimizer using proven facts (phase 2+)
- Human report and trust report generators (phase 8)
