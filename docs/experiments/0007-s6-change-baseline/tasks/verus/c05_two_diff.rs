// Experiment 0007 change task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// CHANGE: Finds i < j with xs[j] - xs[i] == target in a sorted array (two pointers, linear time)
//   (was: xs[i] + xs[j] == target).
// SPEC-BEGIN
fn two_diff_sorted(xs: &[u32], target: u64) -> (r: (bool, usize, usize))
    requires
        forall|i: int, j: int| 0 <= i <= j < xs.len() ==> #[trigger] xs[i] <= #[trigger] xs[j],
    ensures
        r.0 ==> r.1 < r.2 < xs.len() && xs[r.2 as int] - xs[r.1 as int] == target,
        !r.0 ==> forall|a: int, b: int| 0 <= a < b < xs.len() ==> #[trigger] xs[b] - #[trigger] xs[a] != target,
// SPEC-END
{
    let n = xs.len();
    let mut lo: usize = 0;
    let mut hi: usize = n;
    while lo + 1 < hi
        invariant
            hi <= n,
            n == xs.len(),
            lo <= hi,
            lo < n || n == 0,
            forall|i: int, j: int| 0 <= i <= j < xs.len() ==> #[trigger] xs[i] <= #[trigger] xs[j],
            forall|a: int, b: int|
                0 <= a < b < n && (a < lo || b >= hi) ==> #[trigger] xs[a] + #[trigger] xs[b] != target,
        decreases hi - lo,
    {
        let s = xs[lo] as u64 + xs[hi - 1] as u64;
        if s == target {
            return (true, lo, hi - 1);
        } else if s < target {
            assert forall|b: int| lo < b < hi implies xs[lo as int] + xs[b] != target by {
                assert(xs[b] <= xs[hi - 1 as int]);
            }
            lo += 1;
        } else {
            assert forall|a: int| lo <= a < hi - 1 implies #[trigger] xs[a] + xs[hi - 1 as int] != target by {
                assert(xs[lo as int] <= xs[a]);
            }
            hi -= 1;
        }
    }
    (false, 0, 0)
}

fn main() {}

} // verus!
