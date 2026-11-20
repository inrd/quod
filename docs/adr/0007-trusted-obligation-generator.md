# ADR-0007: A small trusted obligation generator over a specified Core IR

- **Status:** Proposed
- **Date:** 2026-11-19
- **Deciders:** —
- **Author:** Claude (agent), at the maintainer's request
- **Closes:** trust-boundary item 1 in [architecture.md](../architecture.md#trust-boundaries) ("obligation generation is faithful")
- **Depends on:** ADR-0001 (Rust, kernel constraints), ADR-0002 (arithmetic semantics), ADR-0005 (pointer-sized integers); kernel logic and encodings follow ADR-0008 (Proposed; supersedes ADR-0003)
- **Evidence gate:** S4a of [ADR-0006](0006-thin-end-to-end-slice.md): `coreir` and `vcgen` for the slice within budget, every generator soundness case in Validation rejected (pending)
- **Revised:** 2026-11-20 (trusted `coreir` crate for Core IR types and text; evidence gate)

## Context

ADR-0003 makes the kernel build each obligation's hypotheses and negated goal from the obligation itself, so a proof cannot smuggle in assumptions. It does not say where obligations come from. If the obligation generator is wrong (drops a precondition, mis-encodes a loop, forgets an overflow check), the kernel faithfully checks the wrong statements and the soundness claim fails. The generator must be decided before phase 2 fixes the obligation encoding.

A related gap: the front end turns source (including specs) into Core IR. If it mistranslates a spec, what is verified is not what the human reviewed.

## Options considered

### Option A: Untrusted generator, listed in the trust report
- For: no constraints on the generator; fastest to build.
- Against: the largest semantic component becomes a soundness hole; "modulo the VC generator" is close to "modulo the compiler", which is what Quod set out to avoid.

### Option B: Separate trusted crate (`vcgen/`) under kernel-style constraints, over a Core IR with a written semantics
- For: trusted code stays small and auditable; the kernel stays a pure logic checker (reusable, and the planned Lean re-checker can check proofs without re-deriving obligations); Core IR semantics in `spec/` is the reference for audits, tests and a later verified generator.
- Against: two trusted crates instead of one; Core IR must be small and frozen enough to specify.

### Option C: Kernel computes obligations from Core IR
- For: one trusted crate; one boundary.
- Against: puts program semantics inside the logic checker; the kernel's 5,000-line budget must then cover both; every Core IR change touches the kernel, which needs an ADR each time.

### Option D: Untrusted generator plus per-obligation certificates against a formal Core IR semantics (e.g. checked in Lean)
- For: smallest trusted base in the long run.
- Against: needs a mechanized Core IR semantics and a second proof pipeline; out of reach before the semantics is stable.

## Decision

Option B now, with Option D as the long-term path.

- `vcgen/` is a workspace crate with the kernel's constraints (std-only plus the `kernel` crate for term types and the `coreir` crate below, `#![forbid(unsafe_code)]`, no `HashMap`/`HashSet`, floats, clocks, threads, randomness, I/O during generation) and its own budget: **≤ 3,000 lines excluding tests**.
- Its input is **Core IR in a canonical text form**, specified in `spec/core-ir.md` before any `vcgen` code is written. The trusted path is: Core IR text → `coreir` parser → `vcgen` → obligations → kernel.
- **`coreir/`** is a separate trusted crate holding the Core IR types, the parser for the canonical text form and its printer. Every trusted consumer of Core IR text (`vcgen`, and `cgen` if [ADR-0010](0010-trusted-c-emitter-from-core-ir.md) is accepted) reads it through this one parser. It has the same constraints as `vcgen` (std-only, no workspace dependencies) and a budget of **≤ 1,500 lines excluding tests**. The parser rejects any text that is not in canonical form, so one Core IR program has exactly one text. The untrusted front end uses `coreir`'s printer to produce the text.
- The front end (source → Core IR) stays untrusted. To close the spec gap, **human reports are generated from Core IR**, not from source: the human reviews the spec that is actually verified. A round-trip test (source spec → Core IR → rendered spec) catches front-end mistranslation early but is not part of the trusted base.
- Obligation IDs and fingerprints are computed in `vcgen` from Core IR structure, so they are deterministic by construction.

### Encodings

These rules are part of `vcgen`'s trusted behavior and will be restated normatively in `spec/core-ir.md`. They follow [ADR-0008](0008-kernel-rules-from-real-certificates.md) §4–§5 (Proposed) and the experiments behind them.

**Sequences** ([experiment 0003](../experiments/0003-seq-as-arrays.md)): the kernel has `Array(Int, S)` with read-over-write axioms only; `Seq(S)` exists only in this encoding.

| Core IR | Obligation |
|---|---|
| value `a: Seq(S)` (array, slice, `&`/`&mut` view) | an array term `a : Array(Int, S)` and a length term `a_len : Int`, with hypothesis `0 <= a_len` |
| `a.len()` | `a_len` |
| `a[k]` | `select(a, k)`, with the bounds obligation `0 <= k < a_len` |
| `a[i := v]` | `store(a, i, v)`; the length term stays `a_len` |
| view `&a[lo..hi]` | no new array: base `a`, offset `lo`, length `hi - lo`; `s[k]` is `select(a, lo + k)` |
| view of a view | offsets add: base `a`, offset `lo + lo2` |
| write through a `&mut` view | `store` on the base array at `offset + i` |
| `s == t` | `s_len = t_len ∧ ∀k. 0 <= k < s_len → s[k] = t[k]` (indices are relative to each view's offset) |
| quantifier over a view's elements | ranges over base indices (`∀m. lo <= m < hi → … select(a, m) …`), not view indices |

- **Never array `=` for sequence equality.** Array `=` also constrains indices outside `[0, len)`, so as a hypothesis it assumes facts Quod's semantics does not give. That would be unsound.
- Quantifying over base indices is not needed for soundness. It keeps solver triggers free of arithmetic (experiment 0003, `s04` vs `s05`).

**Machine integers** ([experiment 0002](../experiments/0002-bit-vectors.md)): the kernel has no bit-vector sort.

| Core IR | Obligation |
|---|---|
| value `x: uN` | `x : Int` with hypothesis `0 <= x < 2^N` |
| value `x: iN` | `x : Int` with hypothesis `-2^(N-1) <= x < 2^(N-1)` |
| value `x: usize` / `isize` | range hypothesis of `u64` / `i64`; stores and results must fit the 32-bit range ([ADR-0005](0005-portable-static-checks-for-pointer-sized-integers.md)) |
| code `+ - *`, casts, `<<` (ADR-0002 preconditions) | exact `Int` arithmetic plus the range obligation on the result |
| `int(e)` (ADR-0002) | `e` itself |
| `& \| ^`, `>>` on unsigned `uN` | uninterpreted symbols `andN`, `orN`, `xorN`, `lshrN` |
| `add_wrap`, `sub_wrap`, `shl_wrap` on `uN` | uninterpreted symbols `add_wrapN`, `sub_wrapN`, `shl_wrapN` |
| unsigned comparisons | `<`, `<=`, `=` on `Int` |

- Range hypotheses are trusted. A range that is too wide makes valid code unprovable. One that is too narrow is unsound. Soundness tests must cover both.
- The meaning of each symbol comes from the kernel's `qd_bv_int` axiom schemas, never from `vcgen`. A kernel obligation carries no schema as a hypothesis. Proofs cite schemas through `qd_bv_int` steps, which the kernel checks against its own axiom file.
- **Solver export** (untrusted): when `vcgen`'s output is printed as SMT-LIB for solvers or agents, it adds the schemas for the symbols the obligation uses, as quantified axioms with patterns. A wrong export can only make proofs fail to elaborate. It cannot get a false proof past the kernel. So the exporter may live outside `vcgen`.
- **Not yet encoded:**
  - Width-dependent operators on `usize`/`isize`: `add_wrap`, `sub_wrap`, `shl_wrap` and `!` give different results on 32- and 64-bit targets.
  - Bitwise operators on signed types, arithmetic `>>`, and `mul_wrap`.

  `vcgen` rejects these with a diagnostic until an ADR adds them. `&`, `|`, `^` and `>>` on `usize` use the 64-bit symbols; their schemas hold at every width.

## Consequences

- **Trusted base grows:** `vcgen` (≤ 3,000 lines), `coreir` (≤ 1,500 lines) and the Core IR semantics document join the kernel. All three are listed in every trust report. This is not new trust: an obligation generator is trusted under every option except D; this ADR makes it small, constrained and visible.
- The kernel stays a pure first-order proof checker; its rules (ADR-0008) and budget are unchanged by this ADR. The sequence and machine-integer encodings move bookkeeping (lengths, view offsets, range facts) from kernel axioms into `vcgen`.
- Core IR becomes a stable interface. Changes to its semantics need an ADR, like kernel rule changes.
- Compilation from Core IR to C (trust-boundary item 2) is still open and unaffected.
- **Agent loop:** none directly; agents see the same obligations. Reports from Core IR may read slightly less like the source; the report generator must render them carefully.
- Docs to update on acceptance: `architecture.md` (component table, trust boundaries), `CLAUDE.md` (crate list and constraints), `README.md` layout, `xtask` (budgets for `coreir` and `vcgen`), new `spec/core-ir.md` (including the encodings above), new `coreir/README.md` and `vcgen/README.md`.

## Validation

- **Gate (S4a):** `coreir` and `vcgen` for the slice subset are within budget, and every generator soundness case below is rejected: the obligations of a program with a known bug must be unprovable. The cases are written down before `vcgen` code:
  - an out-of-range read through a view;
  - sequence equality that differs only past `len`;
  - overflow that is reachable only on a 32-bit target;
  - a dropped precondition at a call site;
  - a loop invariant that holds on entry but is not preserved;
  - a false `assert` followed by code that is correct given the assertion (the cut rule of [ADR-0013](0013-annotations-and-small-obligations.md)).
- `coreir` round-trips: printing a parsed canonical text gives the same bytes, on the whole slice corpus.
- `coreir` and `vcgen` stay within budget through phase 2.
- The soundness test suite (phase-2 exit criterion) keeps these cases and grows with each new construct.
- Revisit if Core IR cannot be kept small enough to specify in one document, or if a mechanized semantics (Option D) becomes cheap.
