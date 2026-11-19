# compiler/ — `quodc`

**Empty skeleton.** Phase 1 starts here, in Rust ([ADR-0001](../docs/adr/0001-implementation-language.md)), as part of a Cargo workspace at the repo root. Arithmetic semantics follow [ADR-0002](../docs/adr/0002-arithmetic-semantics.md).

Future scope (untrusted components, see [../docs/architecture.md](../docs/architecture.md)):

- Lexer, LL(1) parser, canonical formatter (phase 1)
- Name resolution, type checker, effect checker, ownership/permission checker (phases 1, 4, 5)
- Core IR and proof obligation generator with stable IDs (phase 2)
- Optimizer using proven facts; C emitter producing UB-free C (phase 1+)
- Human report and trust report generators (phase 8)
