// Experiment 0007 change task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;
use vstd::set_lib::*;

verus! {

// CHANGE: Reverses an index-linked list in a pool in place; returns the new head and
//   the list length (was: new head only). next[n] is the link of node n;
//   next.Length is the null index. order is the ghost list of nodes from head.
// SPEC-BEGIN
fn pool_list_reverse(next: &mut [usize], head: usize, Ghost(order): Ghost<Seq<usize>>) -> (r: (usize, usize))
    requires
        forall|i: int| 0 <= i < order.len() ==> order[i] < old(next).len(),
        forall|i: int, j: int| 0 <= i < j < order.len() ==> order[i] != order[j],
        head == (if order.len() == 0 { old(next).len() } else { order[0] }),
        forall|i: int| 0 <= i < order.len() - 1 ==> old(next)[order[i] as int] == order[i + 1],
        order.len() > 0 ==> old(next)[order[order.len() - 1] as int] == old(next).len(),
    ensures
        final(next).len() == old(next).len(),
        r.1 == order.len(),
        r.0 == (if order.len() == 0 { final(next).len() } else { order[order.len() - 1] }),
        forall|i: int| 1 <= i < order.len() ==> final(next)[order[i] as int] == order[i - 1],
        order.len() > 0 ==> final(next)[order[0] as int] == final(next).len(),
        forall|n: int| 0 <= n < final(next).len() && !order.contains(n as usize) ==> final(next)[n] == old(next)[n],
// SPEC-END
{
    let n = next.len();
    proof {
        let m = order.map_values(|x: usize| x as int);
        assert(m.no_duplicates()) by {
            assert forall|i: int, j: int| 0 <= i < j < m.len() implies m[i] != m[j] by {
                assert(order[i] != order[j]);
            }
        }
        m.unique_seq_to_set();
        let r = set_int_range(0, n as int);
        assert(m.to_set().subset_of(r)) by {
            assert forall|x: int| m.to_set().contains(x) implies r.contains(x) by {
                let i = choose|i: int| 0 <= i < m.len() && m[i] == x;
                assert(order[i] < n);
            }
        }
        lemma_int_range(0, n as int);
        vstd::set_lib::lemma_len_subset(m.to_set(), r);
        assert(order.len() <= n);
    }
    let mut prev: usize = n;
    let mut cur: usize = head;
    let ghost mut k: int = 0;
    let mut len: usize = 0;
    while cur != n
        invariant
            next.len() == n,
            n == old(next).len(),
            0 <= k <= order.len(),
            len == k,
            order.len() <= n,
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
        len = len + 1;
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
    (prev, len)
}

fn main() {}

} // verus!
