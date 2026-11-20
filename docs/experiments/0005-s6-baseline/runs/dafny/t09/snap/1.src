// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// Pool of nodes as parallel arrays; links are indices.
// SPEC-BEGIN
method pool_insert_after(next: array<usize>, val: array<u32>, node: usize, slot: usize, v: u32)
  requires next.Length < 0x1_0000_0000_0000_0000
  requires next.Length == val.Length
  requires node < next.Length
  requires slot < next.Length
  requires node != slot
  modifies next, val
  ensures next[node] == slot
  ensures next[slot] == old(next[node])
  ensures val[slot] == v
  ensures forall k :: 0 <= k < next.Length ==> (k != node && k != slot) ==> next[k] == old(next[k])
  ensures forall k :: 0 <= k < val.Length ==> k != slot ==> val[k] == old(val[k])
// SPEC-END
{
  val[slot] := v;
  next[slot] := next[node];
  next[node] := slot;
}
