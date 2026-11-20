// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Reverses an index-linked list in a pool in place; returns the new head.
// next[n] is the link of node n; next.len() is the null index.
// order is the ghost list of nodes from head.
// SPEC-BEGIN
fn pool_list_reverse(next: &mut [usize], head: usize, Ghost(order): Ghost<Seq<usize>>) -> (r: usize)
    requires
        forall|i: int| 0 <= i < order.len() ==> order[i] < old(next).len(),
        forall|i: int, j: int| 0 <= i < j < order.len() ==> order[i] != order[j],
        head == (if order.len() == 0 { old(next).len() } else { order[0] }),
        forall|i: int| 0 <= i < order.len() - 1 ==> old(next)[order[i] as int] == order[i + 1],
        order.len() > 0 ==> old(next)[order[order.len() - 1] as int] == old(next).len(),
    ensures
        final(next).len() == old(next).len(),
        r == (if order.len() == 0 { final(next).len() } else { order[order.len() - 1] }),
        forall|i: int| 1 <= i < order.len() ==> final(next)[order[i] as int] == order[i - 1],
        order.len() > 0 ==> final(next)[order[0] as int] == final(next).len(),
        forall|n: int| 0 <= n < final(next).len() && !order.contains(n as usize) ==> final(next)[n] == old(next)[n],
// SPEC-END
{
    // ?h: implementation goes here
    0
}

fn main() {}

} // verus!
