# Experiments

Spikes that test design assumptions before code depends on them. Each has a report `NNNN-title.md` and, if it has inputs or scripts, a directory of the same name. Reports are evidence for ADRs; they decide nothing on their own.

| # | Question | Result |
|---|---|---|
| [0001](0001-alethe-elaboration.md) | Do cvc5 certificates fit ADR-0003's accepted rules? | No: 0/61 without local rewrite rules; Alethe printer lacks sequences and datatypes; CPC covers all solved obligations |
