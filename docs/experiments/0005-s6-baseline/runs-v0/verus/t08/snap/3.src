// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// SPEC-BEGIN
fn insert_sorted(xs: &mut [u32], n: usize, v: u32) -> (r: usize)
    requires
        n < old(xs).len(),
        forall|i: int, j: int| 0 <= i <= j < n ==> #[trigger] old(xs)[i] <= #[trigger] old(xs)[j],
    ensures
        final(xs).len() == old(xs).len(),
        r <= n,
        final(xs)[r as int] == v,
        forall|k: int| 0 <= k < r ==> final(xs)[k] == old(xs)[k],
        forall|k: int| r + 1 <= k < n + 1 ==> final(xs)[k] == old(xs)[k - 1],
        forall|k: int| n + 1 <= k < final(xs).len() ==> final(xs)[k] == old(xs)[k],
        forall|i: int, j: int| 0 <= i <= j < n + 1 ==> #[trigger] final(xs)[i] <= #[trigger] final(xs)[j],
// SPEC-END
{
    let ghost old = xs@;
    let mut i = n;
    while i > 0 && xs[i - 1] > v
        invariant
            i <= n,
            n < xs.len(),
            xs.len() == old.len(),
            forall|i: int, j: int| 0 <= i <= j < n ==> #[trigger] old[i] <= #[trigger] old[j],
            forall|k: int| 0 <= k < i ==> xs[k] == old[k],
            forall|k: int| i < k <= n ==> xs[k] == old[k - 1],
            forall|k: int| n < k < xs.len() ==> xs[k] == old[k],
            forall|k: int| i <= k < n ==> old[k] > v,
        decreases i,
    {
        xs[i] = xs[i - 1];
        i -= 1;
    }
    xs[i] = v;
    proof {
        assert forall|a: int, b: int| 0 <= a <= b < n + 1 implies #[trigger] xs@[a] <= #[trigger] xs@[b] by {
            if a < i as int && b < i as int {
                assert(xs@[a] == old[a] && xs@[b] == old[b]);
            } else if a < i as int && b == i as int {
                assert(xs@[a] == old[a]);
                assert(old[a] <= old[i as int - 1]);
            } else if a < i as int {
                assert(xs@[a] == old[a]);
                assert(xs@[b] == old[b - 1]);
                assert(old[a] <= old[i as int - 1]);
                assert(old[i as int - 1] <= v);
                assert(old[a] <= old[b - 1]);
            } else if a == i as int {
                if b > a {
                    assert(old[b - 1] > v);
                }
            } else {
                assert(xs@[a] == old[a - 1]);
                assert(xs@[b] == old[b - 1]);
            }
        }
    }
    i
}

fn main() {}

} // verus!
