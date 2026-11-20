// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Pool of nodes as parallel arrays; links are indices.
// SPEC-BEGIN
fn pool_insert_after(next: &mut [usize], val: &mut [u32], node: usize, slot: usize, v: u32)
    requires
        old(next).len() == old(val).len(),
        node < old(next).len(),
        slot < old(next).len(),
        node != slot,
    ensures
        final(next).len() == old(next).len(),
        final(val).len() == old(val).len(),
        final(next)[node as int] == slot,
        final(next)[slot as int] == old(next)[node as int],
        final(val)[slot as int] == v,
        forall|k: int| 0 <= k < final(next).len() ==> (k != node && k != slot) ==> final(next)[k] == old(next)[k],
        forall|k: int| 0 <= k < final(val).len() ==> k != slot ==> final(val)[k] == old(val)[k],
// SPEC-END
{
    // ?h: implementation goes here
}

fn main() {}

} // verus!
