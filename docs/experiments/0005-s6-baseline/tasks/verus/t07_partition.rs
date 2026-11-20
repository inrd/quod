// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Rearranges xs in place (a permutation; not stated in the spec, which
// matches the Quod slice subset, but required of the implementation).
// SPEC-BEGIN
fn partition(xs: &mut [u32], pivot: u32) -> (r: usize)
    ensures
        final(xs).len() == old(xs).len(),
        r <= final(xs).len(),
        forall|k: int| 0 <= k < r ==> final(xs)[k] < pivot,
        forall|k: int| r <= k < final(xs).len() ==> final(xs)[k] >= pivot,
// SPEC-END
{
    // ?h: implementation goes here
    0
}

fn main() {}

} // verus!
