// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Sums the elements into a 64-bit result.
// SPEC-BEGIN
spec fn sum(s: Seq<u32>) -> int
    decreases s.len(),
{
    if s.len() == 0 {
        0
    } else {
        s[0] + sum(s.subrange(1, s.len() as int))
    }
}

fn sum_u64(xs: &[u32]) -> (r: u64)
    requires
        xs.len() < 0x1_0000_0000,
    ensures
        r == sum(xs@),
// SPEC-END
{
    let n = xs.len();
    let mut i = n;
    let mut acc: u64 = 0;
    while i > 0
        invariant
            i <= n,
            n == xs.len(),
            n < 0x1_0000_0000,
            acc == sum(xs@.subrange(i as int, n as int)),
            acc <= (n - i) * 0xffff_ffffu64,
        decreases i,
    {
        i -= 1;
        proof {
            let a = xs@.subrange(i as int, n as int);
            let b = xs@.subrange(i as int + 1, n as int);
            assert(a.subrange(1, a.len() as int) =~= b);
            assert(a[0] == xs@[i as int]);
            assert((n - i) * 0xffff_ffffu64 <= 0xffff_ffff_ffff_ffffu64) by (nonlinear_arith)
                requires n < 0x1_0000_0000, i < n;
            assert((n - i - 1) * 0xffff_ffffu64 + 0xffff_ffffu64 == (n - i) * 0xffff_ffffu64) by (nonlinear_arith);
        }
        acc = acc + xs[i] as u64;
    }
    proof {
        assert(xs@.subrange(0, n as int) =~= xs@);
    }
    acc
}

fn main() {}

} // verus!
