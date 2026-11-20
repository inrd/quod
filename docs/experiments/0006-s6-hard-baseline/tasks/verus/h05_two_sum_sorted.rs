// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Finds i < j with xs[i] + xs[j] == target in a sorted array (two pointers, linear time).
// SPEC-BEGIN
fn two_sum_sorted(xs: &[u32], target: u64) -> (r: (bool, usize, usize))
    requires
        forall|i: int, j: int| 0 <= i <= j < xs.len() ==> #[trigger] xs[i] <= #[trigger] xs[j],
    ensures
        r.0 ==> r.1 < r.2 < xs.len() && xs[r.1 as int] + xs[r.2 as int] == target,
        !r.0 ==> forall|a: int, b: int| 0 <= a < b < xs.len() ==> #[trigger] xs[a] + #[trigger] xs[b] != target,
// SPEC-END
{
    // ?h: implementation goes here
    (false, 0, 0)
}

fn main() {}

} // verus!
