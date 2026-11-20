// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

class Node {
  var next: Node?
  var val: u32
}

class Pool {
  var free: Node?
}

// Pointer-linked counterpart of t10: pop the head of the free list.
// SPEC-BEGIN
method ptr_alloc(p: Pool) returns (r: Node?)
  modifies p
  ensures old(p.free) == null ==> r == null && p.free == null
  ensures old(p.free) != null ==> r == old(p.free) && p.free == old(p.free.next)
// SPEC-END
{
  r := p.free;
  if r != null {
    p.free := r.next;
  }
}
