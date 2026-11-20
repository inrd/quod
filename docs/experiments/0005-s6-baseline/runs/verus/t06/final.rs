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
    let mut i: usize = 0;
    while i < n
        invariant
            n == xs.len(),
            xs.len() == old(xs).len(),
            0 <= i <= n,
            forall|k: int| 0 <= k < old(xs).len() ==> old(xs)[k] * f <= 4294967295,
            forall|k: int| 0 <= k < i ==> xs[k] == old(xs)[k] * f,
            forall|k: int| i <= k < n ==> xs[k] == old(xs)[k],
        decreases n - i,
    {
        let v = xs[i];
        xs[i] = v * f;
        i += 1;
    }
}

fn main() {}

} // verus!
