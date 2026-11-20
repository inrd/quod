// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Three-way partition of values 0, 1, 2 in one pass, by swapping.
// SPEC-BEGIN
fn dutch_flag(xs: &mut [u32]) -> (r: (usize, usize))
    requires
        forall|k: int| 0 <= k < old(xs).len() ==> old(xs)[k] <= 2,
    ensures
        final(xs).len() == old(xs).len(),
        r.0 <= r.1 <= final(xs).len(),
        forall|k: int| 0 <= k < r.0 ==> final(xs)[k] == 0,
        forall|k: int| r.0 <= k < r.1 ==> final(xs)[k] == 1,
        forall|k: int| r.1 <= k < final(xs).len() ==> final(xs)[k] == 2,
        final(xs)@.to_multiset() == old(xs)@.to_multiset(),
// SPEC-END
{
    // ?h: implementation goes here
    (0, 0)
}

fn main() {}

} // verus!
