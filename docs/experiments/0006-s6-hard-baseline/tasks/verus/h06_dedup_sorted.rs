// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Removes duplicates from a sorted array in place; returns the new length.
// SPEC-BEGIN
fn dedup_sorted(xs: &mut [u32]) -> (k: usize)
    requires
        forall|i: int, j: int| 0 <= i <= j < old(xs).len() ==> #[trigger] old(xs)[i] <= #[trigger] old(xs)[j],
    ensures
        final(xs).len() == old(xs).len(),
        k <= final(xs).len(),
        forall|a: int, b: int| 0 <= a < b < k ==> #[trigger] final(xs)[a] < #[trigger] final(xs)[b],
        forall|v: u32| old(xs)@.contains(v) <==> final(xs)@.subrange(0, k as int).contains(v),
// SPEC-END
{
    // ?h: implementation goes here
    0
}

fn main() {}

} // verus!
