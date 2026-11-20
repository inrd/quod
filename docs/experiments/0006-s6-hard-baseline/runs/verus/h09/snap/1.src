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
    let ghost s = view(l);
    let mut cur = l;
    let mut acc: Option<Box<Node>> = None;
    while cur.is_some()
        invariant
            view(cur).reverse() + view(acc) == s.reverse(),
        decreases view(cur).len(),
    {
        let mut n = cur.unwrap();
        cur = n.next.take();
        proof {
            let v = n.val;
            assert(view(cur).reverse() + seq![v] + view(acc) =~= s.reverse());
        }
        n.next = acc;
        acc = Some(n);
    }
    acc
}

fn main() {}

} // verus!
