// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// Length of an index-linked list in a pool. next[n] is the link of node n;
// next.len() is the null index. order is the ghost list of nodes from head.
// SPEC-BEGIN
fn pool_list_len(next: &[usize], head: usize, Ghost(order): Ghost<Seq<usize>>) -> (r: usize)
    requires
        forall|i: int| 0 <= i < order.len() ==> order[i] < next.len(),
        forall|i: int, j: int| 0 <= i < j < order.len() ==> order[i] != order[j],
        head == (if order.len() == 0 { next.len() } else { order[0] }),
        forall|i: int| 0 <= i < order.len() - 1 ==> next[order[i] as int] == order[i + 1],
        order.len() > 0 ==> next[order[order.len() - 1] as int] == next.len(),
    ensures
        r == order.len(),
// SPEC-END
{
    proof {
        let m = order.map_values(|x: usize| x as int);
        assert(m.no_duplicates()) by {
            assert forall|i: int, j: int| 0 <= i < j < m.len() implies m[i] != m[j] by {
                assert(m[i] == order[i] as int);
                assert(m[j] == order[j] as int);
            }
        }
        m.unique_seq_to_set();
        let r = vstd::set_lib::set_int_range(0, next.len() as int);
        assert(m.to_set().subset_of(r)) by {
            assert forall|x: int| m.to_set().contains(x) implies r.contains(x) by {
                let i = choose|i: int| 0 <= i < m.len() && m[i] == x;
                assert(order[i] < next.len());
            }
        }
        vstd::set_lib::lemma_int_range(0, next.len() as int);
        vstd::set_lib::lemma_len_subset(m.to_set(), r);
        assert(order.len() <= next.len());
    }
    let mut cur = head;
    let mut count: usize = 0;
    while cur != next.len()
        invariant
            count <= order.len(),
            order.len() <= next.len(),
            forall|i: int| 0 <= i < order.len() ==> order[i] < next.len(),
            forall|i: int| 0 <= i < order.len() - 1 ==> next[order[i] as int] == order[i + 1],
            order.len() > 0 ==> next[order[order.len() - 1] as int] == next.len(),
            cur == (if count == order.len() { next.len() } else { order[count as int] }),
        decreases order.len() - count,
    {
        assert(count < order.len());
        cur = next[cur];
        count += 1;
    }
    count
}

fn main() {}

} // verus!
