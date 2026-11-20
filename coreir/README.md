# coreir/ — Core IR text and well-formedness

Implements §3 of [spec/core-ir.md](../spec/core-ir.md) for the thin slice ([ADR-0006](../docs/adr/0006-thin-end-to-end-slice.md) S2b):

- `ast.rs`: Core IR syntax.
- `text.rs`: parser and printer for the canonical text. A text is accepted only if printing the parsed program gives back the same bytes.
- `check.rs`: the well-formedness rules of §3.4, which `vcgen` and `cgen` rely on.

**Trust status:** built to Proposed [ADR-0007](../docs/adr/0007-trusted-obligation-generator.md). If it is accepted, this crate is part of the trusted base. It already follows the kernel's constraints (no dependencies, `#![forbid(unsafe_code)]`, no hash collections, floats, threads, clocks, randomness or I/O) and its budget of ≤ 1,500 lines excluding tests; `cargo xtask lint` checks both.

Tests: `tests/corpus.rs` parses every `tests/coreir/*.core` file and the example in the spec, and checks a list of texts that must be rejected.
