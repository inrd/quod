// Experiment 0007 change task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

// CHANGE: Position of the first node holding key in an index-linked list in a pool,
//   or the list length if none (was: list length). next[n] is the link of node n,
//   val[n] its value; next.Length is the null index. order is the ghost list of nodes.
// SPEC-BEGIN
fn pool_list_find(next: &[usize], val: &[u32], head: usize, key: u32, Ghost(order): Ghost<Seq<usize>>) -> (r: usize)
    requires
        forall|i: int| 0 <= i < order.len() ==> order[i] < next.len(),
        forall|i: int, j: int| 0 <= i < j < order.len() ==> order[i] != order[j],
        head == (if order.len() == 0 { next.len() } else { order[0] }),
        forall|i: int| 0 <= i < order.len() - 1 ==> next[order[i] as int] == order[i + 1],
        order.len() > 0 ==> next[order[order.len() - 1] as int] == next.len(),
        val.len() == next.len(),
    ensures
        r <= order.len(),
        forall|p: int| 0 <= p < r ==> val[order[p] as int] != key,
        r < order.len() ==> val[order[r as int] as int] == key,
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
            val.len() == next.len(),
            forall|p: int| 0 <= p < count ==> val[order[p] as int] != key,
        decreases order.len() - count,
    {
        assert(count < order.len());
        if val[cur] == key {
            return count;
        }
        cur = next[cur];
        count += 1;
    }
    count
}

fn main() {}

} // verus!
