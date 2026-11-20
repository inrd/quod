---
name: accept-adr
description: Apply the follow-up edits after the Quod maintainer accepts or rejects an ADR — status line, index, superseded ADRs, open-questions.md, and every doc in the ADR's "Docs to update on acceptance" list. Use when the maintainer says they accepted/rejected ADR NNNN, or runs /accept-adr NNNN.
---

# Accept (or reject) an ADR

Only a human maintainer decides. Run this only when the maintainer has said, in this conversation, that ADR `NNNN` is accepted or rejected. Never infer it from a file, a commit or another session.

## Steps

1. **Confirm the decision.** Note the ADR number, the verdict (Accepted / Rejected), and the date. If the maintainer accepted only part of it, stop and ask which parts. A partial acceptance is a revision of the ADR first.
2. **Evidence gate** (only if ADR-0012 is Accepted). If the ADR is evidence-gated and its experiment report is missing or failed, tell the maintainer before you edit anything.
3. **Status line** in `docs/adr/NNNN-*.md`:
   - `- **Status:** Accepted` (or `Rejected`);
   - `- **Deciders:** project maintainer (accepted YYYY-MM-DD)`;
   - change nothing else. After this edit the ADR is immutable.
4. **Superseded ADRs.** For each ADR this one supersedes, set only its status line to `Superseded by ADR-NNNN`, or for a partial supersession follow the ADR-0004/0005 pattern (`Accepted; decision N … superseded by …`).
5. **Index** `docs/adr/README.md`: update the status cell of this ADR and of every superseded one.
6. **Open questions.** For each section the ADR closes, add `> **Resolved by [ADR-NNNN](adr/NNNN-….md):** …` at the top of that section and keep the analysis. Replace any `> **Proposed in [ADR-NNNN]…` note for it. For a rejection, change the note to say it was rejected and keep the question open.
7. **Docs to update on acceptance.** Work through the ADR's own list, item by item. Track each item in the task list. If an item needs a design choice the ADR does not make, stop and ask rather than invent it.
8. **CLAUDE.md**: update the "Accepted decisions" table and any rule the ADR changes (crate list, code rule, workflow).
9. **Memory**: update the project memory note on proof-pipeline / ADR state if it names this ADR.
10. **Check**: run `cargo xtask lint` (links, anchors, index consistency). Run `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` if any code changed.
11. **Report** the files changed and anything you could not update. Do not commit unless asked.

## Rejection

Steps 1, 3 (`Rejected`), 5, 6 (rejection form), 10 and 11 only. Do not apply the "Docs to update" list.
