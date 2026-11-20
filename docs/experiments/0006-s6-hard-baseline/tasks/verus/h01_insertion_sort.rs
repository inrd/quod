// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Sorts xs in place.
// SPEC-BEGIN
fn insertion_sort(xs: &mut [u32])
    ensures
        final(xs).len() == old(xs).len(),
        forall|i: int, j: int| 0 <= i <= j < final(xs).len() ==> #[trigger] final(xs)[i] <= #[trigger] final(xs)[j],
        final(xs)@.to_multiset() == old(xs)@.to_multiset(),
// SPEC-END
{
    // ?h: implementation goes here
}

fn main() {}

} // verus!
