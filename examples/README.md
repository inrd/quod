# Examples

> **All syntax is provisional and non-normative.** These files are design sketches. `01_core_max.qd` parses with the phase-1 `quodc` and is kept in canonical format (checked by `cargo test`); the others use phase-2+ constructs and are rejected.

| File | Shows | Phase |
|---|---|---|
| `01_core_max.qd` | explicit widths, mandatory `effects`, bounded `for`, explicit runtime guards | 1 |
| `02_ring_buffer.qd` | type invariants (`where`), `ensures`, `old()`, proven index bounds; an open overflow obligation | 2 |
| `03_refinement_sum.qd` | `spec fn` + `impl fn … refines`, typed hole, `cost` contract, loop invariants | 6–7 |
| `04_effects_trust.qd` | effects, capabilities, regions, `trust fn` with provenance, `noalloc`/`stack` | 4–5, 7 |
| `05_holes_and_proofs.qd` | typed holes in invariants/measures, oracle workflow, explicit proof placeholder | 2–3 |

Spec-only operators used in the sketches (`=>`, `forall`, `ok()`, `some()`, `unwrap()`) are not in the phase-1 grammar.
