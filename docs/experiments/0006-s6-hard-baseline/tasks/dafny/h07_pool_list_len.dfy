// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// Length of an index-linked list in a pool. next[n] is the link of node n;
// next.Length is the null index. order is the ghost list of nodes from head.
// SPEC-BEGIN
method pool_list_len(next: array<usize>, head: usize, ghost order: seq<usize>) returns (r: usize)
  requires next.Length < 0x1_0000_0000_0000_0000
  requires forall i :: 0 <= i < |order| ==> order[i] < next.Length
  requires forall i, j :: 0 <= i < j < |order| ==> order[i] != order[j]
  requires head == (if |order| == 0 then next.Length else order[0])
  requires forall i :: 0 <= i < |order| - 1 ==> next[order[i]] == order[i + 1]
  requires |order| > 0 ==> next[order[|order| - 1]] == next.Length
  ensures r == |order|
// SPEC-END
{
  // ?h: implementation goes here
}
