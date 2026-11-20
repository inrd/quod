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
    // ?h: implementation goes here
    0
}

fn main() {}

} // verus!
