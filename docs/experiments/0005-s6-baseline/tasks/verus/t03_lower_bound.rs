// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// SPEC-BEGIN
fn lower_bound(xs: &[u32], key: u32) -> (r: usize)
    requires
        forall|i: int, j: int| 0 <= i <= j < xs.len() ==> #[trigger] xs[i] <= #[trigger] xs[j],
    ensures
        r <= xs.len(),
        forall|k: int| 0 <= k < r ==> xs[k] < key,
        forall|k: int| r <= k < xs.len() ==> xs[k] >= key,
// SPEC-END
{
    // ?h: implementation goes here
    0
}

fn main() {}

} // verus!
