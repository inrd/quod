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
    // ?h: implementation goes here
    0
}

fn main() {}

} // verus!
