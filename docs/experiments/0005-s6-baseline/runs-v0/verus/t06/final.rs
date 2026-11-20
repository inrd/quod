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
    let n = xs.len();
    let ghost orig = xs@;
    let mut i: usize = 0;
    while i < n
        invariant
            n == xs.len(),
            orig.len() == n,
            0 <= i <= n,
            forall|k: int| 0 <= k < n ==> orig[k] * f <= 4294967295,
            forall|k: int| 0 <= k < i ==> xs[k] == orig[k] * f,
            forall|k: int| i <= k < n ==> xs[k] == orig[k],
        decreases n - i,
    {
        assert(xs[i as int] == orig[i as int]);
        assert(orig[i as int] * f <= 4294967295);
        xs[i] = xs[i] * f;
        i += 1;
    }
}

fn main() {}

} // verus!
