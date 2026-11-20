// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// SPEC-BEGIN
fn all_below(xs: &[u32], bound: u32) -> (r: bool)
    ensures
        r ==> forall|k: int| 0 <= k < xs.len() ==> xs[k] < bound,
        !r ==> exists|k: int| 0 <= k < xs.len() && xs[k] >= bound,
// SPEC-END
{
    // ?h: implementation goes here
    false
}

fn main() {}

} // verus!
