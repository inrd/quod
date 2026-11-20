// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// SPEC-BEGIN
fn scale(xs: &mut [u32], f: u32)
    requires
        forall|k: int| 0 <= k < old(xs).len() ==> old(xs)[k] * f <= 4294967295,
    ensures
        final(xs).len() == old(xs).len(),
        forall|k: int| 0 <= k < final(xs).len() ==> final(xs)[k] == old(xs)[k] * f,
// SPEC-END
{
    // ?h: implementation goes here
}

fn main() {}

} // verus!
