// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// SPEC-BEGIN
fn reverse(xs: &mut [u32])
    ensures
        final(xs).len() == old(xs).len(),
        forall|k: int| 0 <= k < final(xs).len() ==> final(xs)[k] == old(xs)[final(xs).len() - 1 - k],
// SPEC-END
{
    // ?h: implementation goes here
}

fn main() {}

} // verus!
