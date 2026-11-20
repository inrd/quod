# Quod

**Quod** is an experimental systems programming language designed for code written mostly by AI agents, with every code path checked against a contract by a small proof kernel.

It aims to be a C alternative: a low-level primitive with no garbage collector, no runtime and no undefined behavior, with higher-level abstractions built on top as verified libraries. These are design goals; see the status below for what exists today.

| | |
|---|---|
| Compiler | `quodc` |
| Source extension | `.qd` |
| Status | **Phase 1 in progress.** `quodc` parses, formats and type-checks the phase-1 core and parses contracts. Nothing is verified yet: no C emitter, `vcgen`, prover or kernel. Accepted decisions are in the [ADR index](docs/adr/README.md). All syntax is provisional. |

## The idea in four lines

1. Agents make writing code cheap and checking it expensive, so the compiler is a **verification oracle** built for the agent ↔ compiler loop.
2. Humans review **specs** (spec ↔ intent); a small trusted kernel checks **proofs** (code ↔ spec). Humans should rarely read implementation code.
3. Verbosity is free for agents, so **nothing is implicit**. Redundant statements are consistency checks.
4. Proven facts can replace runtime checks, so proofs should also buy performance.

## Sketch (provisional, non-normative)

```c
fn push(rb: &mut Ring<f32, N>, x: f32) -> Result<(), Full>
    effects {}
    ensures ok(result) => rb.len == old(rb.len) + 1
{
    if rb.len == N { return Err(Full); }
    let i = (rb.head + rb.len) % N;   // intended: proven i < N, so no bounds check
    rb.data[i] = x;
    rb.len += 1;
    return Ok(());
}
```

## Repository layout

```
docs/          design documents (start with vision.md)
docs/adr/      architecture decision records
spec/          language specification drafts (grammar, format, diagnostics)
examples/      provisional syntax sketches (.qd)
compiler/      quodc: parser, formatter, type checker (phase 1, in progress)
kernel/        trusted proof checker                  (empty: phase 2)
oracle/        JSON query server for agents           (empty: phase 3)
tests/         test corpus (parse/, typecheck/ populated)
tools/         pinned external tools for experiments (cvc5, Ethos, Carcara)
xtask/         repository checks (cargo xtask lint)
```

## Reading order

1. [docs/vision.md](docs/vision.md)
2. [docs/design-principles.md](docs/design-principles.md)
3. [docs/architecture.md](docs/architecture.md)
4. [docs/roadmap.md](docs/roadmap.md)
5. [docs/open-questions.md](docs/open-questions.md)

Also: [Power of 10 mapping](docs/power-of-10-mapping.md), [oracle protocol](docs/oracle-protocol.md), [prior art](docs/prior-art.md), [grammar draft](spec/grammar-draft.md), [typing draft](spec/typing.md), [contracts draft](spec/contracts.md), [Core IR draft](spec/core-ir.md), [canonical format](spec/format.md), [diagnostic codes](spec/diagnostics.md).

## Building

```
cargo build
cargo test
./target/debug/quodc check examples/01_core_max.qd   # type-check only for now
```

## License

[MIT](LICENSE)
