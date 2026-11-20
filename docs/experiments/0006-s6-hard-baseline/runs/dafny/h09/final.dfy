// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// Reverses a pointer-linked list in place; returns the new head.
// nodes is the ghost list of nodes from head.
// SPEC-BEGIN
class Node {
  var val: u32
  var next: Node?
}

method ptr_list_reverse(head: Node?, ghost nodes: seq<Node>) returns (r: Node?)
  requires forall i, j :: 0 <= i < j < |nodes| ==> nodes[i] != nodes[j]
  requires head == (if |nodes| == 0 then null else nodes[0])
  requires forall i :: 0 <= i < |nodes| - 1 ==> nodes[i].next == nodes[i + 1]
  requires |nodes| > 0 ==> nodes[|nodes| - 1].next == null
  modifies set i | 0 <= i < |nodes| :: nodes[i]
  ensures r == (if |nodes| == 0 then null else nodes[|nodes| - 1])
  ensures forall i :: 1 <= i < |nodes| ==> nodes[i].next == nodes[i - 1]
  ensures |nodes| > 0 ==> nodes[0].next == null
  ensures forall i :: 0 <= i < |nodes| ==> nodes[i].val == old(nodes[i].val)
// SPEC-END
{
  var prev: Node? := null;
  var cur := head;
  ghost var k := 0;
  ghost var n := |nodes|;
  while cur != null
    invariant 0 <= k <= n
    invariant prev == (if k == 0 then null else nodes[k-1])
    invariant cur == (if k == n then null else nodes[k])
    invariant forall i :: 1 <= i < k ==> nodes[i].next == nodes[i-1]
    invariant k > 0 ==> nodes[0].next == null
    invariant forall i :: k <= i < n - 1 ==> nodes[i].next == nodes[i+1]
    invariant k < n ==> nodes[n-1].next == null
    invariant forall i :: 0 <= i < n ==> nodes[i].val == old(nodes[i].val)
    decreases n - k
  {
    var nx := cur.next;
    cur.next := prev;
    prev := cur;
    cur := nx;
    k := k + 1;
  }
  r := prev;
}
