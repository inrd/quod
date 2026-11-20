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
    if n < 2 {
        return (false, 0, 0);
    }
    let mut a: usize = 0;
    let mut b: usize = 1;
    while b < n
        invariant
            n == xs.len(),
            a < b,
            b <= n,
            forall|i: int, j: int| 0 <= i <= j < xs.len() ==> #[trigger] xs[i] <= #[trigger] xs[j],
            forall|x: int, y: int|
                0 <= x < y < n && (x < a || y < b) ==> #[trigger] xs[y] - #[trigger] xs[x] != target,
        decreases 2 * n - a - b,
    {
        let d = xs[b] as u64 - xs[a] as u64;
        if d == target {
            return (true, a, b);
        } else if d < target {
            assert forall|x: int| a <= x < b implies xs[b as int] - #[trigger] xs[x] != target by {
                assert(xs[a as int] <= xs[x]);
            }
            b += 1;
        } else {
            assert forall|y: int| b <= y < n implies #[trigger] xs[y] - xs[a as int] != target by {
                assert(xs[b as int] <= xs[y]);
            }
            a += 1;
            if a == b {
                b += 1;
            }
        }
    }
    (false, 0, 0)
}

fn main() {}

} // verus!
