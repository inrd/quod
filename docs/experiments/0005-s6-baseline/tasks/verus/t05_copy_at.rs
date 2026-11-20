// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// SPEC-BEGIN
fn copy_at(dst: &mut [u32], src: &[u32], off: usize)
    requires
        off + src.len() <= old(dst).len(),
    ensures
        final(dst).len() == old(dst).len(),
        forall|k: int| 0 <= k < src.len() ==> final(dst)[off + k] == src[k],
        forall|k: int| 0 <= k < off ==> final(dst)[k] == old(dst)[k],
        forall|k: int| off + src.len() <= k < final(dst).len() ==> final(dst)[k] == old(dst)[k],
// SPEC-END
{
    // ?h: implementation goes here
}

fn main() {}

} // verus!
