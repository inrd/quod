# Oracle protocol (draft)

> **Status:** draft for phase 3. Message shapes are illustrative; field names will change. Transport, versioning and encoding details are open.

The oracle is the compiler running as a server. Agents send JSON requests and receive structured, actionable answers. It is untrusted: it guides agents, the kernel decides.

## Design goals

- **Structured, never prose-only.** Every diagnostic has a stable code, a location, machine-readable data, and optional human text.
- **Stable IDs.** Obligations and holes keep their IDs across edits that don't affect them, so an agent's task list survives refactors elsewhere.
- **Minimal and concrete.** Counterexamples are concrete values with the shortest trace reaching the failure.
- **Deterministic.** Same source + same request → same response.
- **Context-bounded.** Responses include only what's needed (the compositional context of a goal), not whole files.

## Transport (proposal)

JSON-RPC 2.0 over stdio, one session per workspace. Alternative: line-delimited JSON. Open.

## Common types

```jsonc
// Location
{ "file": "src/ring.qd", "line": 12, "col": 9, "end_line": 12, "end_col": 27 }

// Stable obligation ID: module path + item + kind + structural path (not line numbers)
"ob:ring::push/ensures#0"
"ob:ring::push/body/stmt3/index-bound"

// Stable hole ID: as written in source
"?h1"
```

### Obligation kinds (initial list)

`precondition`, `postcondition`, `type-invariant`, `loop-invariant-init`, `loop-invariant-preserve`, `termination`, `index-bound`, `overflow`, `div-zero`, `shift-range`, `cast-range`, `permission`, `refinement`, `resource-cost`, `resource-stack`, `effect`.

## Requests

### `obligations/list` — the task list

```json
{ "jsonrpc": "2.0", "id": 1, "method": "obligations/list",
  "params": { "module": "ring", "status": ["open", "failed"] } }
```

```json
{ "jsonrpc": "2.0", "id": 1, "result": {
  "obligations": [
    { "id": "ob:ring::push/ensures#0",
      "kind": "postcondition",
      "status": "open",
      "loc": { "file": "src/ring.qd", "line": 9, "col": 13 },
      "statement": "ok(result) => rb.len == old(rb.len) + 1",
      "depends_on": [],
      "fingerprint": "sha256:4f1c…" },
    { "id": "ob:ring::push/body/stmt3/index-bound",
      "kind": "index-bound",
      "status": "failed",
      "loc": { "file": "src/ring.qd", "line": 13, "col": 5 },
      "statement": "i < N",
      "fingerprint": "sha256:9a02…" }
  ],
  "summary": { "total": 7, "proved": 5, "open": 1, "failed": 1, "trusted": 0 }
} }
```

`fingerprint` changes when the obligation's statement or context changes; agents can cache proofs by it.

### `goal/at` — goal and context at a program point or obligation

```json
{ "jsonrpc": "2.0", "id": 2, "method": "goal/at",
  "params": { "obligation": "ob:ring::push/body/stmt3/index-bound" } }
```

```json
{ "jsonrpc": "2.0", "id": 2, "result": {
  "goal": "i < N",
  "context": {
    "bindings": [
      { "name": "rb", "type": "&mut Ring<f32, N>" },
      { "name": "x",  "type": "f32" },
      { "name": "i",  "type": "u32", "def": "(rb.head + rb.len) % N" }
    ],
    "facts": [
      { "fact": "rb.head < N",   "source": "type-invariant Ring.head" },
      { "fact": "rb.len <= N",   "source": "type-invariant Ring.len" },
      { "fact": "rb.len != N",   "source": "branch at line 11" },
      { "fact": "N > 0",         "source": "MISSING?", "note": "required for % N; not derivable" }
    ],
    "callee_contracts": []
  }
} }
```

### `hole/info` — typed holes

```json
{ "jsonrpc": "2.0", "id": 3, "method": "hole/info", "params": { "hole": "?h1" } }
```

```json
{ "jsonrpc": "2.0", "id": 3, "result": {
  "hole": "?h1",
  "kind": "fn-body",
  "expected_type": "i128",
  "effects_allowed": [],
  "must_satisfy": [
    { "id": "ob:sum::sum_fast/refines", "statement": "result == sum(xs)" },
    { "id": "ob:sum::sum_fast/cost",    "statement": "cost <= xs.len / 4 + 16" }
  ],
  "in_scope": [ { "name": "xs", "type": "&[i64]" } ],
  "callable": [ { "name": "simd::load_i64x4", "signature": "…", "effects": [] } ]
} }
```

Hole kinds: `expr`, `fn-body`, `loop-invariant`, `measure`, `proof`.

### `check/counterexample` — concrete failure

```json
{ "jsonrpc": "2.0", "id": 4, "method": "check/counterexample",
  "params": { "obligation": "ob:ring::push/body/stmt3/index-bound" } }
```

```json
{ "jsonrpc": "2.0", "id": 4, "result": {
  "found": true,
  "model": { "N": 0, "rb.head": "<vacuous: N=0 makes Ring uninhabited>" },
  "trace": [
    { "loc": { "file": "src/ring.qd", "line": 13 }, "step": "eval (rb.head + rb.len) % N", "value": "div-by-zero" }
  ],
  "minimal": true,
  "searcher": "smt:z3-4.13 (untrusted)",
  "hint": { "kind": "missing-precondition", "suggest": "add `where N > 0` to Ring" }
} }
```

Counterexamples are untrusted; the agent should re-validate them by concrete execution where possible (`check/run-witness`, speculative).

### `wp/suggest` — weakest-precondition suggestion

```json
{ "jsonrpc": "2.0", "id": 5, "method": "wp/suggest",
  "params": { "obligation": "ob:app::main/body/stmt7/precondition" } }
```

```json
{ "jsonrpc": "2.0", "id": 5, "result": {
  "callee": "ring::push",
  "required": "rb.len < N",
  "wp_at_caller_entry": "q.len + k < N",
  "options": [
    { "kind": "add-requires",  "text": "requires q.len + k < N" },
    { "kind": "add-check",     "text": "if q.len + k >= N { return Err(Full); }" },
    { "kind": "strengthen-loop-invariant", "text": "invariant q.len + (k - j) < N" }
  ]
} }
```

### `proof/submit` — kernel check

```json
{ "jsonrpc": "2.0", "id": 6, "method": "proof/submit",
  "params": { "obligation": "ob:ring::push/ensures#0",
              "format": "alethe-qd/0",
              "proof": "(step t1 (cl ...) :rule eq_congruent) ... (step t14 (cl) :rule resolution :premises (t9 t13))" } }
```

```json
{ "jsonrpc": "2.0", "id": 6, "result": {
  "status": "accepted", "kernel": "quod-kernel 0.0.0", "fingerprint": "sha256:4f1c…" } }
```

On rejection: `{ "status": "rejected", "at_step": "t14", "rule": "la_generic", "expected": "…", "got": "…" }`.

The proof is text in the kernel's Alethe subset ([ADR-0003](adr/0003-kernel-logic-and-proof-format.md)): a flat list of steps refuting the negated goal, ending in the empty clause `(cl)`. The kernel builds the hypotheses and negated goal from the obligation; `assume` lines in the submitted proof must match them exactly. Proofs containing rejected rules (`hole`, `lia_generic`, …) are refused before checking; run them through the elaborator (`proof/elaborate`, untrusted) first.

## Diagnostics

Every compiler error is also a protocol message:

```json
{ "code": "E0412", "severity": "error",
  "loc": { "file": "src/a.qd", "line": 4, "col": 2 },
  "data": { "kind": "effect-violation", "callee": "log::write", "missing_effects": ["io"] },
  "fixes": [ { "kind": "add-effect", "text": "effects { io }" } ],
  "message": "call to log::write requires effect `io`" }
```

Diagnostic codes are stable across versions; retired codes are never reused.

## Notifications (server → agent)

- `obligations/changed` — IDs whose status changed after an edit.
- `trust/changed` — the trust report changed (new or removed `trust`/`assume`).
- `effects/changed` — a function's declared or inferred effects changed.

## Open issues

- Stable ID scheme robustness under refactors (structural paths vs. content hashes vs. explicit labels).
- Whether edits go through the protocol (typed edit operations) or through files ([open-questions.md](open-questions.md#5-source-representation)).
- Batching and streaming for large task lists.
- How much SMT-derived information to expose without implying trust.
