// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

class Node {
  var next: Node?
  var val: u32
  constructor (v: u32, n: Node?) ensures val == v && next == n { val := v; next := n; }
}

// Pointer-linked counterpart of t09: insert a new node holding v after node.
// SPEC-BEGIN
method ptr_insert_after(node: Node, v: u32) returns (n: Node)
  modifies node
  ensures fresh(n)
  ensures node.next == n
  ensures n.next == old(node.next)
  ensures n.val == v
  ensures node.val == old(node.val)
// SPEC-END
{
  n := new Node(v, node.next);
  node.next := n;
}
