# External tools

Pinned versions of the solvers and checkers used by [experiments](../docs/experiments/README.md) and the [ADR-0006](../docs/adr/0006-thin-end-to-end-slice.md) slice. None is a build dependency of `quodc`, and none is part of the trusted base.

```sh
tools/fetch.sh            # cvc5, Ethos, CPC signature  (~25 MB download)
tools/fetch.sh carcara    # also build Carcara from source (needs cargo, ~2–3 min)
tools/fetch.sh baseline   # Dafny and Verus for experiment 0005 (~85 MB; Verus also needs its Rust toolchain via rustup)
```

Everything lands in `tools/bin/` (git-ignored). Each download is checked against the SHA-256 in [pins.sh](pins.sh) before it is unpacked; a mismatch stops the script. Re-running skips tools already at the pinned version.

| Tool | Version | Source | Used for |
|---|---|---|---|
| `cvc5` | 1.4.2 | official static release | Alethe and CPC certificates |
| `ethos` | 0.2.5 | official release; the exact commit cvc5 1.4.2 pins | checking CPC proofs |
| `cpc/` | from cvc5 1.4.2 source | `proofs/eo/cpc` | the CPC signature Ethos checks against |
| `carcara` | commit `819f309` | built with `cargo install --locked` | checking and elaborating Alethe proofs |
| `dafny` | 4.11.0 (bundled Z3) | official release | S6 baseline ([experiment 0005](../docs/experiments/0005-s6-baseline.md)) |
| `verus` | 0.2026.11.02.8bf68ee | official release; needs Rust 1.99.1 | S6 baseline |

Platforms: macOS arm64, Linux x86_64, Linux arm64.

## Helpers

| Script | Does |
|---|---|
| [check-cpc.sh](check-cpc.sh) `<file.smt2> [cvc5 flags]` | proves the obligation with cvc5 and checks the CPC proof with Ethos (`--require-proof-of-false`), as cvc5's own regression tests do; prints `checked` |

Experiment scripts take the solver path as their first argument: pass `tools/bin/cvc5`.

## Known limits

- **Carcara and RARE rewrites.** cvc5's Alethe proofs use `rare_rewrite` steps (experiment 0001). Carcara checks them only when given the rule definitions (`--rare-file`) in its own `declare-rare-rule` format; cvc5 ships them in its `define-rule` format (`src/theory/*/rewrites`), and Carcara has no converter. Until one exists, Carcara rejects those steps. This is part of ADR-0006 S4a, not of tool pinning.
- Bumping a version changes experiment results. Update `pins.sh` (version and every checksum) and say which version each experiment report used.
