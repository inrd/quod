// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Length of an index-linked list in a pool. next[n] is the link of node n;
// next.len() is the null index. order is the ghost list of nodes from head.
// SPEC-BEGIN
fn pool_list_len(next: &[usize], head: usize, Ghost(order): Ghost<Seq<usize>>) -> (r: usize)
    requires
        forall|i: int| 0 <= i < order.len() ==> order[i] < next.len(),
        forall|i: int, j: int| 0 <= i < j < order.len() ==> order[i] != order[j],
        head == (if order.len() == 0 { next.len() } else { order[0] }),
        forall|i: int| 0 <= i < order.len() - 1 ==> next[order[i] as int] == order[i + 1],
        order.len() > 0 ==> next[order[order.len() - 1] as int] == next.len(),
    ensures
        r == order.len(),
// SPEC-END
{
    // ?h: implementation goes here
    0
}

fn main() {}

} // verus!
