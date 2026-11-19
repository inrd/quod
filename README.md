# Quod

**Quod** is an experimental systems programming language in which code is written almost entirely by AI agents and every code path is formally verified.

It is meant as a C alternative: a low-level, high-performance primitive with **no garbage collector, no runtime and no undefined behavior**. Higher-level abstractions are built on top of it as verified libraries.

| | |
|---|---|
| Compiler | `quodc` |
| Source extension | `.qd` |
| Status | **Phase 1 in progress.** Core decisions accepted ([ADR-0001–0003](docs/adr/README.md)). `quodc` has a lexer, parser and canonical formatter; no type checker or C emitter yet. All syntax is provisional. |

## The idea in four lines

1. Agents make writing code cheap and checking it expensive, so the compiler is a **verification oracle** built for the agent ↔ compiler loop.
2. Humans review **specs** (spec ↔ intent); a small trusted kernel checks **proofs** (code ↔ spec). Humans should rarely read implementation code.
3. Verbosity is free for agents, so **nothing is implicit**. Redundant statements are consistency checks.
4. Proven facts remove runtime checks: **correctness pays for performance**.

## Sketch (provisional, non-normative)

```c
fn push(rb: &mut Ring<f32, N>, x: f32) -> Result<(), Full>
    effects {}
    ensures ok(result) => rb.len == old(rb.len) + 1
{
    if rb.len == N { return Err(Full); }
    let i = (rb.head + rb.len) % N;   // proven i < N: no bounds check emitted
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
compiler/      quodc: lexer, parser, formatter        (phase 1, in progress)
kernel/        trusted proof checker                  (empty: phase 2)
oracle/        JSON query server for agents           (empty: phase 3)
tests/         test corpus (parse/ populated)
```

## Reading order

1. [docs/vision.md](docs/vision.md)
2. [docs/design-principles.md](docs/design-principles.md)
3. [docs/architecture.md](docs/architecture.md)
4. [docs/roadmap.md](docs/roadmap.md)
5. [docs/open-questions.md](docs/open-questions.md)

Also: [Power of 10 mapping](docs/power-of-10-mapping.md), [oracle protocol](docs/oracle-protocol.md), [prior art](docs/prior-art.md), [grammar draft](spec/grammar-draft.md), [typing draft](spec/typing.md), [canonical format](spec/format.md), [diagnostic codes](spec/diagnostics.md).

## Building

```
cargo build
cargo test
./target/debug/quodc fmt examples/01_core_max.qd
```

## License

[MIT](LICENSE)
