#!/usr/bin/env python3
"""Experiment 0007: build the change tasks.

    python3 -I build_tasks.py

Each task starts from the agent's verified solution of an experiment 0006
task (same verifier), replaces the description comment and the SPEC block
with a changed specification, and keeps the old body, helpers and proofs.
Writes tasks/<lang>/<task>_<name>.<ext>.
"""
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "..", "0006-s6-hard-baseline", "runs")
EXT = {"dafny": "dfy", "verus": "rs"}

D_SORTED = "forall i, j :: 0 <= i <= j < {x}.Length ==> {x}[i] <= {x}[j]"
V_SORTED = "forall|i: int, j: int| 0 <= i <= j < {x}.len() ==> #[trigger] {x}[i] <= #[trigger] {x}[j]"
D_LINKS = """  requires next.Length < 0x1_0000_0000_0000_0000
  requires forall i :: 0 <= i < |order| ==> order[i] < next.Length
  requires forall i, j :: 0 <= i < j < |order| ==> order[i] != order[j]
  requires head == (if |order| == 0 then next.Length else order[0])
  requires forall i :: 0 <= i < |order| - 1 ==> next[order[i]] == order[i + 1]
  requires |order| > 0 ==> next[order[|order| - 1]] == next.Length"""
V_LINKS = """        forall|i: int| 0 <= i < order.len() ==> order[i] < N.len(),
        forall|i: int, j: int| 0 <= i < j < order.len() ==> order[i] != order[j],
        head == (if order.len() == 0 { N.len() } else { order[0] }),
        forall|i: int| 0 <= i < order.len() - 1 ==> N[order[i] as int] == order[i + 1],
        order.len() > 0 ==> N[order[order.len() - 1] as int] == N.len(),"""

# task id: (source task, description, {lang: spec})
TASKS = {
    "c01_sort_desc": ("h01", "Sorts xs in place in descending order (was: ascending).", {
        "dafny": """method insertion_sort(xs: array<u32>)
  requires xs.Length < 0x1_0000_0000_0000_0000
  modifies xs
  ensures forall i, j :: 0 <= i <= j < xs.Length ==> xs[i] >= xs[j]
  ensures multiset(xs[..]) == old(multiset(xs[..]))""",
        "verus": """fn insertion_sort(xs: &mut [u32])
    ensures
        final(xs).len() == old(xs).len(),
        forall|i: int, j: int| 0 <= i <= j < final(xs).len() ==> #[trigger] final(xs)[i] >= #[trigger] final(xs)[j],
        final(xs)@.to_multiset() == old(xs)@.to_multiset(),"""}),
    "c02_four_flag": ("h02", "Four-way partition of values 0, 1, 2, 3 in one pass, by swapping\n// (was: three values, two bounds).", {
        "dafny": """method dutch_flag(xs: array<u32>) returns (a: usize, b: usize, c: usize)
  requires xs.Length < 0x1_0000_0000_0000_0000
  requires forall k :: 0 <= k < xs.Length ==> xs[k] <= 3
  modifies xs
  ensures a <= b <= c <= xs.Length
  ensures forall k :: 0 <= k < a ==> xs[k] == 0
  ensures forall k :: a <= k < b ==> xs[k] == 1
  ensures forall k :: b <= k < c ==> xs[k] == 2
  ensures forall k :: c <= k < xs.Length ==> xs[k] == 3
  ensures multiset(xs[..]) == old(multiset(xs[..]))""",
        "verus": """fn dutch_flag(xs: &mut [u32]) -> (r: (usize, usize, usize))
    requires
        forall|k: int| 0 <= k < old(xs).len() ==> old(xs)[k] <= 3,
    ensures
        final(xs).len() == old(xs).len(),
        r.0 <= r.1 <= r.2 <= final(xs).len(),
        forall|k: int| 0 <= k < r.0 ==> final(xs)[k] == 0,
        forall|k: int| r.0 <= k < r.1 ==> final(xs)[k] == 1,
        forall|k: int| r.1 <= k < r.2 ==> final(xs)[k] == 2,
        forall|k: int| r.2 <= k < final(xs).len() ==> final(xs)[k] == 3,
        final(xs)@.to_multiset() == old(xs)@.to_multiset(),"""}),
    "c03_count_lt": ("h03", "Counts the elements strictly less than key (was: equal to key).", {
        "dafny": """function count_lt(s: seq<u32>, key: u32): nat
{
  if |s| == 0 then 0 else (if s[0] < key then 1 else 0) + count_lt(s[1..], key)
}

method count_lt_key(xs: array<u32>, key: u32) returns (r: usize)
  requires xs.Length < 0x1_0000_0000_0000_0000
  ensures r == count_lt(xs[..], key)""",
        "verus": """spec fn count_lt(s: Seq<u32>, key: u32) -> nat
    decreases s.len(),
{
    if s.len() == 0 {
        0
    } else {
        (if s[0] < key { 1nat } else { 0nat }) + count_lt(s.subrange(1, s.len() as int), key)
    }
}

fn count_lt_key(xs: &[u32], key: u32) -> (r: usize)
    ensures
        r == count_lt(xs@, key),"""}),
    "c04_sum_sq": ("h04", "Sums the squares of the elements into a 64-bit result (was: plain sum);\n// elements are below 2^16.", {
        "dafny": """function sum_sq(s: seq<u32>): int
{
  if |s| == 0 then 0 else s[0] * s[0] + sum_sq(s[1..])
}

method sum_sq_u64(xs: array<u32>) returns (r: u64)
  requires xs.Length < 0x1_0000_0000
  requires forall k :: 0 <= k < xs.Length ==> xs[k] < 0x1_0000
  ensures r == sum_sq(xs[..])""",
        "verus": """spec fn sum_sq(s: Seq<u32>) -> int
    decreases s.len(),
{
    if s.len() == 0 {
        0
    } else {
        s[0] * s[0] + sum_sq(s.subrange(1, s.len() as int))
    }
}

fn sum_sq_u64(xs: &[u32]) -> (r: u64)
    requires
        xs.len() < 0x1_0000_0000,
        forall|k: int| 0 <= k < xs.len() ==> xs[k] < 0x1_0000,
    ensures
        r == sum_sq(xs@),"""}),
    "c05_two_diff": ("h05", "Finds i < j with xs[j] - xs[i] == target in a sorted array (two pointers, linear time)\n// (was: xs[i] + xs[j] == target).", {
        "dafny": """method two_diff_sorted(xs: array<u32>, target: u64) returns (found: bool, i: usize, j: usize)
  requires xs.Length < 0x1_0000_0000_0000_0000
  requires forall a, b :: 0 <= a <= b < xs.Length ==> xs[a] <= xs[b]
  ensures found ==> i < j < xs.Length && xs[j] as int - xs[i] as int == target
  ensures !found ==> forall a, b :: 0 <= a < b < xs.Length ==> xs[b] as int - xs[a] as int != target""",
        "verus": """fn two_diff_sorted(xs: &[u32], target: u64) -> (r: (bool, usize, usize))
    requires
        forall|i: int, j: int| 0 <= i <= j < xs.len() ==> #[trigger] xs[i] <= #[trigger] xs[j],
    ensures
        r.0 ==> r.1 < r.2 < xs.len() && xs[r.2 as int] - xs[r.1 as int] == target,
        !r.0 ==> forall|a: int, b: int| 0 <= a < b < xs.len() ==> #[trigger] xs[b] - #[trigger] xs[a] != target,"""}),
    "c06_pool_find": ("h07", "Position of the first node holding key in an index-linked list in a pool,\n// or the list length if none (was: list length). next[n] is the link of node n,\n// val[n] its value; next.Length is the null index. order is the ghost list of nodes.", {
        "dafny": """method pool_list_find(next: array<usize>, val: array<u32>, head: usize, key: u32, ghost order: seq<usize>) returns (r: usize)
""" + D_LINKS + """
  requires val.Length == next.Length
  ensures r <= |order|
  ensures forall p :: 0 <= p < r ==> val[order[p]] != key
  ensures r < |order| ==> val[order[r]] == key""",
        "verus": """fn pool_list_find(next: &[usize], val: &[u32], head: usize, key: u32, Ghost(order): Ghost<Seq<usize>>) -> (r: usize)
    requires
""" + V_LINKS.replace("N", "next") + """
        val.len() == next.len(),
    ensures
        r <= order.len(),
        forall|p: int| 0 <= p < r ==> val[order[p] as int] != key,
        r < order.len() ==> val[order[r as int] as int] == key,"""}),
    "c07_pool_rev_len": ("h08", "Reverses an index-linked list in a pool in place; returns the new head and\n// the list length (was: new head only). next[n] is the link of node n;\n// next.Length is the null index. order is the ghost list of nodes from head.", {
        "dafny": """method pool_list_reverse(next: array<usize>, head: usize, ghost order: seq<usize>) returns (r: usize, len: usize)
""" + D_LINKS + """
  modifies next
  ensures len == |order|
  ensures r == (if |order| == 0 then next.Length else order[|order| - 1])
  ensures forall i :: 1 <= i < |order| ==> next[order[i]] == order[i - 1]
  ensures |order| > 0 ==> next[order[0]] == next.Length
  ensures forall n :: 0 <= n < next.Length && n !in order ==> next[n] == old(next[n])""",
        "verus": """fn pool_list_reverse(next: &mut [usize], head: usize, Ghost(order): Ghost<Seq<usize>>) -> (r: (usize, usize))
    requires
""" + V_LINKS.replace("N", "old(next)") + """
    ensures
        final(next).len() == old(next).len(),
        r.1 == order.len(),
        r.0 == (if order.len() == 0 { final(next).len() } else { order[order.len() - 1] }),
        forall|i: int| 1 <= i < order.len() ==> final(next)[order[i] as int] == order[i - 1],
        order.len() > 0 ==> final(next)[order[0] as int] == final(next).len(),
        forall|n: int| 0 <= n < final(next).len() && !order.contains(n as usize) ==> final(next)[n] == old(next)[n],"""}),
    "c08_merge_into": ("h10", "Merges two sorted arrays into a caller-provided output array\n// (was: into a newly allocated one).", {
        "dafny": """method merge(a: array<u32>, b: array<u32>, out: array<u32>)
  requires a.Length + b.Length < 0x1_0000_0000_0000_0000
  requires out.Length == a.Length + b.Length
  requires out != a && out != b
  requires """ + D_SORTED.format(x="a") + """
  requires """ + D_SORTED.format(x="b") + """
  modifies out
  ensures """ + D_SORTED.format(x="out") + """
  ensures multiset(out[..]) == multiset(a[..]) + multiset(b[..])""",
        "verus": """fn merge(a: &[u32], b: &[u32], out: &mut [u32])
    requires
        old(out).len() == a.len() + b.len(),
        """ + V_SORTED.format(x="a") + """,
        """ + V_SORTED.format(x="b") + """,
    ensures
        final(out).len() == old(out).len(),
        """ + V_SORTED.format(x="final(out)") + """,
        final(out)@.to_multiset() == a@.to_multiset().add(b@.to_multiset()),"""}),
}


def build(lang, src_text, desc, spec):
    src_text = src_text.replace("Experiment 0006 baseline task", "Experiment 0007 change task")
    b = src_text.index("// SPEC-BEGIN\n")
    e = src_text.index("// SPEC-END")
    # The description: the comment lines right above SPEC-BEGIN.
    lines = src_text[:b].split("\n")[:-1]
    k = len(lines)
    while k > 0 and lines[k - 1].startswith("//") and "Experiment" not in lines[k - 1]:
        k -= 1
    head = "\n".join(lines[:k]) + "\n"
    old = src_text[b:e]
    # 0006 kept spec helpers (Node, view, count, sum) inside the block; keep type
    # declarations, replace the rest.
    keep = ""
    if lang == "dafny" and "class Node" in old:
        keep = old[old.index("class Node"):old.index("}\n", old.index("class Node")) + 2] + "\n"
    note = "// CHANGE: " + desc.replace("\n// ", "\n//   ") + "\n"
    return head + note + "// SPEC-BEGIN\n" + keep + spec + "\n" + src_text[e:]


def main():
    for tid, (src, desc, specs) in TASKS.items():
        for lang, spec in specs.items():
            p = os.path.join(SRC, lang, src, "final." + EXT[lang])
            with open(p, encoding="utf-8") as f:
                text = f.read()
            out = os.path.join(HERE, "tasks", lang, "%s.%s" % (tid, EXT[lang]))
            with open(out, "w", encoding="utf-8") as f:
                f.write(build(lang, text, desc, spec))


if __name__ == "__main__":
    main()
