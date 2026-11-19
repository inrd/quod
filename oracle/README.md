# oracle/ — agent query server

**Empty skeleton.** Phase 3. Implements the JSON protocol drafted in [../docs/oracle-protocol.md](../docs/oracle-protocol.md): obligation task lists with stable IDs, goals and context, typed holes, counterexamples, weakest-precondition suggestions, proof submission.

Untrusted: it guides agents; the kernel decides. Rust ([ADR-0001](../docs/adr/0001-implementation-language.md)). May use SMT solvers as search helpers and runs the untrusted proof elaborator ([ADR-0003](../docs/adr/0003-kernel-logic-and-proof-format.md)).
