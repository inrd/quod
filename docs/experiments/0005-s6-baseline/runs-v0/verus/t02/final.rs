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
    let mut i: usize = 0;
    while i < xs.len()
        invariant
            0 <= i <= xs.len(),
            forall|k: int| 0 <= k < i ==> xs[k] < bound,
        decreases xs.len() - i,
    {
        if xs[i] >= bound {
            return false;
        }
        i += 1;
    }
    true
}

fn main() {}

} // verus!
