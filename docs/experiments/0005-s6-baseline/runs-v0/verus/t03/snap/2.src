// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// SPEC-BEGIN
fn lower_bound(xs: &[u32], key: u32) -> (r: usize)
    requires
        forall|i: int, j: int| 0 <= i <= j < xs.len() ==> #[trigger] xs[i] <= #[trigger] xs[j],
    ensures
        r <= xs.len(),
        forall|k: int| 0 <= k < r ==> xs[k] < key,
        forall|k: int| r <= k < xs.len() ==> xs[k] >= key,
// SPEC-END
{
    let mut lo: usize = 0;
    let mut hi: usize = xs.len();
    while lo < hi
        invariant
            lo <= hi <= xs.len(),
            forall|i: int, j: int| 0 <= i <= j < xs.len() ==> #[trigger] xs[i] <= #[trigger] xs[j],
            forall|k: int| 0 <= k < lo ==> xs[k] < key,
            forall|k: int| hi <= k < xs.len() ==> xs[k] >= key,
        decreases hi - lo
    {
        let mid = lo + (hi - lo) / 2;
        if xs[mid] < key {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

fn main() {}

} // verus!
