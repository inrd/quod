// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

pub struct Node {
    pub val: u32,
    pub next: Option<Box<Node>>,
}

// Pointer-linked counterpart of t09: insert a new node holding v after node.
// SPEC-BEGIN
fn ptr_insert_after(node: &mut Node, v: u32)
    ensures
        final(node).val == old(node).val,
        final(node).next matches Some(n) && n.val == v && n.next == old(node).next,
// SPEC-END
{
    let old_next = node.next.take();
    node.next = Some(Box::new(Node { val: v, next: old_next }));
}

fn main() {}

} // verus!
