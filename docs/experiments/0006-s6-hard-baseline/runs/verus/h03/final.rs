// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Counts the elements equal to key.
// SPEC-BEGIN
spec fn count(s: Seq<u32>, key: u32) -> nat
    decreases s.len(),
{
    if s.len() == 0 {
        0
    } else {
        (if s[0] == key { 1nat } else { 0nat }) + count(s.subrange(1, s.len() as int), key)
    }
}

fn count_eq(xs: &[u32], key: u32) -> (r: usize)
    ensures
        r == count(xs@, key),
// SPEC-END
{
    let n = xs.len();
    let mut i = n;
    let mut r: usize = 0;
    proof {
        assert(xs@.subrange(n as int, n as int) =~= Seq::<u32>::empty());
    }
    while i > 0
        invariant
            i <= n,
            n == xs@.len(),
            r <= n - i,
            r == count(xs@.subrange(i as int, n as int), key),
        decreases i,
    {
        i = i - 1;
        proof {
            let a = xs@.subrange(i as int, n as int);
            let b = xs@.subrange(i as int + 1, n as int);
            assert(a.subrange(1, a.len() as int) =~= b);
            assert(a[0] == xs@[i as int]);
        }
        if xs[i] == key {
            r = r + 1;
        }
    }
    proof {
        assert(xs@.subrange(0, n as int) =~= xs@);
    }
    r
}

fn main() {}

} // verus!
