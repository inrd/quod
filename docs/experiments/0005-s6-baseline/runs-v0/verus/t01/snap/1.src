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
    let mut r: usize = 0;
    let mut i: usize = 1;
    while i < xs.len()
        invariant
            1 <= i <= xs.len(),
            r < i,
            forall|k: int| 0 <= k < i ==> xs[k] <= xs[r as int],
        decreases xs.len() - i,
    {
        if xs[i] > xs[r] {
            r = i;
        }
        i += 1;
    }
    r
}

fn main() {}

} // verus!
