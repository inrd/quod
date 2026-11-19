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
| `types.rs`, `check.rs` (name resolution, type checker) | yes | [typing.md](../spec/typing.md), [ADR-0004](../docs/adr/0004-phase-1-typing-choices.md) |
| "Unverified checks" report (trap sites; emitting the traps is the C emitter's job) | yes | [typing.md §11](../spec/typing.md#11-runtime-traps-and-the-unverified-checks-report), [ADR-0002](../docs/adr/0002-arithmetic-semantics.md#phase-1-transition) |
| C emitter | no | not yet written down |
| Mechanical LL(1) check of the grammar | no | [grammar-draft §8](../spec/grammar-draft.md) |

## CLI

```
quodc fmt [--check | --write] <file>...
quodc parse <file>
quodc check <file>     # prints the unverified-checks report
```

Exit codes: `0` ok, `1` diagnostics or non-canonical files, `2` usage or I/O error.

## Later scope

- Effect checker, ownership/permission checker (phases 4, 5)
- Core IR and proof obligation generator with stable IDs (phase 2)
- Optimizer using proven facts (phase 2+)
- Human report and trust report generators (phase 8)
