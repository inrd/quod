// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

spec fn has(s: Seq<u32>, v: u32, n: int) -> bool {
    exists|m: int| 0 <= m < n && s[m] == v
}

// Removes duplicates from a sorted array in place; returns the new length.
// SPEC-BEGIN
fn dedup_sorted(xs: &mut [u32]) -> (k: usize)
    requires
        forall|i: int, j: int| 0 <= i <= j < old(xs).len() ==> #[trigger] old(xs)[i] <= #[trigger] old(xs)[j],
    ensures
        final(xs).len() == old(xs).len(),
        k <= final(xs).len(),
        forall|a: int, b: int| 0 <= a < b < k ==> #[trigger] final(xs)[a] < #[trigger] final(xs)[b],
        forall|v: u32| old(xs)@.contains(v) <==> final(xs)@.subrange(0, k as int).contains(v),
// SPEC-END
{
    let n = xs.len();
    if n == 0 {
        return 0;
    }
    let ghost old = xs@;
    let mut k: usize = 1;
    let mut i: usize = 1;
    proof {
        assert(has(xs@, old[0], 1)) by {
            assert(xs@[0] == old[0]);
        }
    }
    while i < n
        invariant
            xs.len() == n,
            old.len() == n,
            forall|a: int, b: int| 0 <= a <= b < n ==> old[a] <= old[b],
            1 <= k <= i <= n,
            forall|j: int| i <= j < n ==> xs@[j] == old[j],
            xs@[k as int - 1] == old[i as int - 1],
            forall|a: int, b: int| 0 <= a < b < k ==> xs@[a] < xs@[b],
            forall|m: int| 0 <= m < k ==> #[trigger] has(old, xs@[m], i as int),
            forall|j: int| 0 <= j < i ==> #[trigger] has(xs@, old[j], k as int),
        decreases n - i,
    {
        let ghost ps = xs@;
        let ghost pk = k;
        let ghost pi = i;
        assert(old[pi as int - 1] <= old[pi as int]);
        if xs[i] != xs[k - 1] {
            let x = xs[i];
            xs[k] = x;
            k += 1;
            i += 1;
            proof {
                assert forall|a: int, b: int| 0 <= a < b < k implies xs@[a] < xs@[b] by {
                    if b < pk {
                    } else {
                        if a < pk - 1 {
                            assert(ps[a] < ps[pk as int - 1]);
                        }
                    }
                }
                assert forall|j: int| 0 <= j < i implies #[trigger] has(xs@, old[j], k as int) by {
                    if j < pi {
                        assert(has(ps, old[j], pk as int));
                        let m = choose|m: int| 0 <= m < pk && ps[m] == old[j];
                        assert(xs@[m] == old[j]);
                    } else {
                        assert(xs@[pk as int] == old[j]);
                    }
                }
                assert forall|m: int| 0 <= m < k implies #[trigger] has(old, xs@[m], i as int) by {
                    if m < pk {
                        assert(has(old, ps[m], pi as int));
                        let j = choose|j: int| 0 <= j < pi && old[j] == ps[m];
                        assert(old[j] == xs@[m]);
                    } else {
                        assert(old[pi as int] == xs@[m]);
                    }
                }
            }
        } else {
            i += 1;
            proof {
                assert forall|j: int| 0 <= j < i implies #[trigger] has(xs@, old[j], k as int) by {
                    if j < pi {
                    } else {
                        assert(xs@[pk as int - 1] == old[j]);
                    }
                }
                assert forall|m: int| 0 <= m < k implies #[trigger] has(old, xs@[m], i as int) by {
                    assert(has(old, xs@[m], pi as int));
                    let j = choose|j: int| 0 <= j < pi && old[j] == xs@[m];
                    assert(old[j] == xs@[m]);
                }
            }
        }
    }
    proof {
        assert forall|v: u32| old.contains(v) <==> xs@.subrange(0, k as int).contains(v) by {
            if old.contains(v) {
                let j = choose|j: int| 0 <= j < old.len() && old[j] == v;
                assert(has(xs@, old[j], k as int));
                let m = choose|m: int| 0 <= m < k && xs@[m] == old[j];
                assert(xs@.subrange(0, k as int)[m] == v);
            }
            if xs@.subrange(0, k as int).contains(v) {
                let m = choose|m: int| 0 <= m < k && xs@.subrange(0, k as int)[m] == v;
                assert(has(old, xs@[m], n as int));
                let j = choose|j: int| 0 <= j < n && old[j] == xs@[m];
                assert(old[j] == v);
            }
        }
    }
    k
}

fn main() {}

} // verus!
