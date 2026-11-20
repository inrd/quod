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
    let ghost orig = dst@;
    let mut i: usize = 0;
    while i < src.len()
        invariant
            0 <= i <= src.len(),
            off + src.len() <= orig.len(),
            dst@.len() == orig.len(),
            forall|k: int| 0 <= k < i ==> dst@[off + k] == src[k],
            forall|k: int| 0 <= k < off ==> dst@[k] == orig[k],
            forall|k: int| off + src.len() <= k < dst@.len() ==> dst@[k] == orig[k],
        decreases src.len() - i,
    {
        assert(off + i < dst.len());
        let j: usize = off + i;
        dst[j] = src[i];
        i += 1;
    }
}

fn main() {}

} // verus!
