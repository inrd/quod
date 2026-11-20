// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Reverses an owned pointer-linked list; returns the new head.
// SPEC-BEGIN
pub struct Node {
    pub val: u32,
    pub next: Option<Box<Node>>,
}

pub open spec fn view(l: Option<Box<Node>>) -> Seq<u32>
    decreases l,
{
    match l {
        None => Seq::empty(),
        Some(n) => seq![n.val] + view(n.next),
    }
}

fn ptr_list_reverse(l: Option<Box<Node>>) -> (r: Option<Box<Node>>)
    ensures
        view(r) == view(l).reverse(),
// SPEC-END
{
    // ?h: implementation goes here
    None
}

fn main() {}

} // verus!
