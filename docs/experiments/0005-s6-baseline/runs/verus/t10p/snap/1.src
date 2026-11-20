// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

pub struct Node {
    pub val: u32,
    pub next: Option<Box<Node>>,
}

// Pointer-linked counterpart of t10: pop the head of the free list.
// SPEC-BEGIN
fn ptr_alloc(free: &mut Option<Box<Node>>) -> (r: Option<Box<Node>>)
    ensures
        *old(free) is None ==> r is None && *final(free) is None,
        *old(free) matches Some(h) ==> (r matches Some(n) && n.val == h.val && *final(free) == h.next),
// SPEC-END
{
    let old = free.take();
    match old {
        None => None,
        Some(mut b) => {
            let nx = b.next.take();
            *free = nx;
            Some(b)
        }
    }
}

fn main() {}

} // verus!
