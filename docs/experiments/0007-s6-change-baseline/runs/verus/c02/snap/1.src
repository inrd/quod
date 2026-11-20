// Experiment 0007 change task. Do not edit between SPEC-BEGIN and SPEC-END.
use vstd::prelude::*;

verus! {

broadcast use vstd::seq_lib::group_to_multiset_ensures;

// CHANGE: Four-way partition of values 0, 1, 2, 3 in one pass, by swapping
//   (was: three values, two bounds).
// SPEC-BEGIN
fn dutch_flag(xs: &mut [u32]) -> (r: (usize, usize, usize))
    requires
        forall|k: int| 0 <= k < old(xs).len() ==> old(xs)[k] <= 3,
    ensures
        final(xs).len() == old(xs).len(),
        r.0 <= r.1 <= r.2 <= final(xs).len(),
        forall|k: int| 0 <= k < r.0 ==> final(xs)[k] == 0,
        forall|k: int| r.0 <= k < r.1 ==> final(xs)[k] == 1,
        forall|k: int| r.1 <= k < r.2 ==> final(xs)[k] == 2,
        forall|k: int| r.2 <= k < final(xs).len() ==> final(xs)[k] == 3,
        final(xs)@.to_multiset() == old(xs)@.to_multiset(),
// SPEC-END
{
    let n = xs.len();
    let mut lo: usize = 0;
    let mut mid: usize = 0;
    let mut c: usize = 0;
    let mut hi: usize = n;
    while c < hi
        invariant
            xs.len() == n,
            lo <= mid <= c <= hi <= n,
            forall|k: int| 0 <= k < lo ==> xs[k] == 0,
            forall|k: int| lo <= k < mid ==> xs[k] == 1,
            forall|k: int| mid <= k < c ==> xs[k] == 2,
            forall|k: int| hi <= k < n ==> xs[k] == 3,
            forall|k: int| c <= k < hi ==> xs[k] <= 3,
            xs@.to_multiset() == old(xs)@.to_multiset(),
        decreases hi - c,
    {
        let v = xs[c];
        if v == 0 {
            proof { lemma_swap_ms(xs@, mid as int, c as int); }
            xs.swap(mid, c);
            proof { lemma_swap_ms(xs@, lo as int, mid as int); }
            xs.swap(lo, mid);
            lo += 1;
            mid += 1;
            c += 1;
        } else if v == 1 {
            proof { lemma_swap_ms(xs@, mid as int, c as int); }
            xs.swap(mid, c);
            mid += 1;
            c += 1;
        } else if v == 2 {
            c += 1;
        } else {
            proof { lemma_swap_ms(xs@, (hi - 1) as int, c as int); }
            xs.swap(c, hi - 1);
            hi -= 1;
        }
    }
    (lo, mid, hi)
}

proof fn lemma_swap_ms(s: Seq<u32>, i: int, j: int)
    requires 0 <= i < s.len(), 0 <= j < s.len(),
    ensures s.update(i, s[j]).update(j, s[i]).to_multiset() == s.to_multiset(),
{
    if i != j {
        lemma_upd_ms(s, i, s[j]);
        lemma_upd_ms(s.update(i, s[j]), j, s[i]);
        let m = s.to_multiset();
        assert(m.count(s[i]) > 0) by { lemma_count_pos(s, i); }
        assert(m.remove(s[i]).insert(s[j]).remove(s[j]).insert(s[i]) =~= m);
    } else {
        assert(s.update(i, s[j]) =~= s);
        assert(s.update(i, s[j]).update(j, s[i]) =~= s);
    }
}

proof fn lemma_count_pos(s: Seq<u32>, i: int)
    requires 0 <= i < s.len(),
    ensures s.to_multiset().count(s[i]) > 0,
    decreases s.len(),
{
    if i < s.len() - 1 {
        lemma_count_pos(s.drop_last(), i);
    }
}

proof fn lemma_upd_ms(s: Seq<u32>, i: int, x: u32)
    requires 0 <= i < s.len(),
    ensures s.update(i, x).to_multiset() == s.to_multiset().remove(s[i]).insert(x),
    decreases s.len(),
{
    let t = s.update(i, x);
    let n = s.len();
    if i == n - 1 {
        assert(t.drop_last() =~= s.drop_last());
        assert(t.to_multiset() =~= s.to_multiset().remove(s[i]).insert(x));
    } else {
        assert(t.drop_last() =~= s.drop_last().update(i, x));
        lemma_upd_ms(s.drop_last(), i, x);
        lemma_count_pos(s.drop_last(), i);
        assert(t.to_multiset() =~= s.to_multiset().remove(s[i]).insert(x));
    }
}

fn main() {}

} // verus!
