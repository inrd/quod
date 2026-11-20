// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Reverses an index-linked list in a pool in place; returns the new head.
// next[n] is the link of node n; next.len() is the null index.
// order is the ghost list of nodes from head.
// SPEC-BEGIN
fn pool_list_reverse(next: &mut [usize], head: usize, Ghost(order): Ghost<Seq<usize>>) -> (r: usize)
    requires
        forall|i: int| 0 <= i < order.len() ==> order[i] < old(next).len(),
        forall|i: int, j: int| 0 <= i < j < order.len() ==> order[i] != order[j],
        head == (if order.len() == 0 { old(next).len() } else { order[0] }),
        forall|i: int| 0 <= i < order.len() - 1 ==> old(next)[order[i] as int] == order[i + 1],
        order.len() > 0 ==> old(next)[order[order.len() - 1] as int] == old(next).len(),
    ensures
        final(next).len() == old(next).len(),
        r == (if order.len() == 0 { final(next).len() } else { order[order.len() - 1] }),
        forall|i: int| 1 <= i < order.len() ==> final(next)[order[i] as int] == order[i - 1],
        order.len() > 0 ==> final(next)[order[0] as int] == final(next).len(),
        forall|n: int| 0 <= n < final(next).len() && !order.contains(n as usize) ==> final(next)[n] == old(next)[n],
// SPEC-END
{
    let n = next.len();
    let mut prev: usize = n;
    let mut cur: usize = head;
    let ghost mut k: int = 0;
    while cur != n
        invariant
            next.len() == n,
            n == old(next).len(),
            0 <= k <= order.len(),
            cur == (if k < order.len() { order[k] } else { n }),
            prev == (if k == 0 { n } else { order[k - 1] }),
            forall|i: int| 0 <= i < order.len() ==> order[i] < old(next).len(),
            forall|i: int, j: int| 0 <= i < j < order.len() ==> order[i] != order[j],
            forall|i: int| 0 <= i < order.len() - 1 ==> old(next)[order[i] as int] == order[i + 1],
            order.len() > 0 ==> old(next)[order[order.len() - 1] as int] == old(next).len(),
            forall|i: int| 1 <= i < k ==> next[order[i] as int] == order[i - 1],
            k > 0 ==> next[order[0] as int] == n,
            forall|i: int| k <= i < order.len() ==> next[order[i] as int] == old(next)[order[i] as int],
            forall|m: int| 0 <= m < n && !order.contains(m as usize) ==> next[m] == old(next)[m],
        decreases order.len() - k,
    {
        assert(k < order.len());
        let nx = next[cur];
        next[cur] = prev;
        prev = cur;
        cur = nx;
        proof {
            k = k + 1;
            if k < order.len() {
                assert(nx == order[k]);
            } else {
                assert(nx == n);
            }
        }
    }
    proof {
        if k < order.len() {
            assert(false);
        }
    }
    prev
}

fn main() {}

} // verus!
