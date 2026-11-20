// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// SPEC-BEGIN
fn max_index(xs: &[u32]) -> (r: usize)
    requires
        xs.len() > 0,
    ensures
        r < xs.len(),
        forall|k: int| 0 <= k < xs.len() ==> xs[k] <= xs[r as int],
// SPEC-END
{
    // ?h: implementation goes here
    0
}

fn main() {}

} // verus!
