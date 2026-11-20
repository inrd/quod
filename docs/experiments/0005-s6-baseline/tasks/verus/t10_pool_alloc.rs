// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Pool free list: next[k] is the link of node k, next.len() is the null index;
// free[0] is the free-list head.
// SPEC-BEGIN
fn pool_alloc(next: &[usize], free: &mut [usize]) -> (r: usize)
    requires
        old(free).len() == 1,
        old(free)[0] <= next.len(),
        forall|k: int| 0 <= k < next.len() ==> next[k] <= next.len(),
    ensures
        final(free).len() == 1,
        old(free)[0] == next.len() ==> r == next.len() && final(free)[0] == next.len(),
        old(free)[0] < next.len() ==> r == old(free)[0] && final(free)[0] == next[r as int],
        final(free)[0] <= next.len(),
// SPEC-END
{
    // ?h: implementation goes here
    0
}

fn main() {}

} // verus!
