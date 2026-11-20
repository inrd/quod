// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Merges two sorted arrays into a new sorted vector.
// SPEC-BEGIN
fn merge(a: &[u32], b: &[u32]) -> (out: Vec<u32>)
    requires
        a.len() + b.len() <= usize::MAX,
        forall|i: int, j: int| 0 <= i <= j < a.len() ==> #[trigger] a[i] <= #[trigger] a[j],
        forall|i: int, j: int| 0 <= i <= j < b.len() ==> #[trigger] b[i] <= #[trigger] b[j],
    ensures
        out.len() == a.len() + b.len(),
        forall|i: int, j: int| 0 <= i <= j < out.len() ==> #[trigger] out[i] <= #[trigger] out[j],
        out@.to_multiset() == a@.to_multiset().add(b@.to_multiset()),
// SPEC-END
{
    // ?h: implementation goes here
    Vec::new()
}

fn main() {}

} // verus!
