# Experiments

Spikes that test design assumptions before code depends on them. Each has a report `NNNN-title.md` and, if it has inputs or scripts, a directory of the same name. Reports are evidence for ADRs; they decide nothing on their own.

| # | Question | Result |
|---|---|---|
| [0001](0001-alethe-elaboration.md) | Do cvc5 certificates fit ADR-0003's accepted rules? | No: 0/61 without local rewrite rules; Alethe printer lacks sequences and datatypes; CPC covers all solved obligations |
| [0002](0002-bit-vectors.md) | Bit-vectors: `qd_bv_int` lemma library or bounded bit-blasting? | Lemma library: 14/14 solved with no rejected or bit-vector rule; bit-blasting fails `c04`/`c05` and leaves `hole` steps |
| [0003](0003-seq-as-arrays.md) | Do sequences encoded as SMT arrays plus a length give checkable certificates? | Yes: 17/17 Alethe certificates with no class-D rule (2 need `--enum-inst`); only two read-over-write axioms used |
