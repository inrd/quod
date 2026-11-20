// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

ghost function ToSet(s: seq<usize>): set<usize>
{
  set x: usize | x in s
}

lemma SetCard(s: seq<usize>)
  requires forall i, j :: 0 <= i < j < |s| ==> s[i] != s[j]
  ensures |ToSet(s)| == |s|
{
  if |s| > 0 {
    SetCard(s[..|s|-1]);
    assert s[|s|-1] !in s[..|s|-1] by {
      if s[|s|-1] in s[..|s|-1] {
        var i :| 0 <= i < |s|-1 && s[..|s|-1][i] == s[|s|-1];
        assert s[i] == s[|s|-1];
      }
    }
    var T := ToSet(s[..|s|-1]);
    assert |T| == |s| - 1;
    assert s[|s|-1] !in T;
    assert ToSet(s) == T + {s[|s|-1]};
    assert |T + {s[|s|-1]}| == |T| + 1;
  }
}

lemma Bound(s: seq<usize>, n: int)
  requires n >= 0
  requires forall i :: 0 <= i < |s| ==> s[i] < n
  requires forall i, j :: 0 <= i < j < |s| ==> s[i] != s[j]
  ensures |s| <= n
{
  SetCard(s);
  var full := set k: int {:autotriggers false} |0 <= k < n;
  assert ToSet(s) <= full;
  assert |full| == n by {
    SetRange(n);
  }
  var S := ToSet(s);
  assert full == S + (full - S);
  assert |full| == |S| + |full - S|;
}

lemma SetRange(n: int)
  requires n >= 0
  ensures |set k: int {:autotriggers false} |0 <= k < n| == n
  decreases n
{
  if n > 0 {
    SetRange(n - 1);
    assert (set k: int {:autotriggers false} |0 <= k < n) == (set k: int {:autotriggers false} |0 <= k < n - 1) + {n - 1};
  }
}

// Length of an index-linked list in a pool. next[n] is the link of node n;
// next.Length is the null index. order is the ghost list of nodes from head.
// SPEC-BEGIN
method pool_list_len(next: array<usize>, head: usize, ghost order: seq<usize>) returns (r: usize)
  requires next.Length < 0x1_0000_0000_0000_0000
  requires forall i :: 0 <= i < |order| ==> order[i] < next.Length
  requires forall i, j :: 0 <= i < j < |order| ==> order[i] != order[j]
  requires head == (if |order| == 0 then next.Length else order[0])
  requires forall i :: 0 <= i < |order| - 1 ==> next[order[i]] == order[i + 1]
  requires |order| > 0 ==> next[order[|order| - 1]] == next.Length
  ensures r == |order|
// SPEC-END
{
  var cur := head;
  var count: usize := 0;
  while cur != next.Length
    invariant 0 <= count <= |order|
    invariant cur == (if count == |order| then next.Length else order[count])
    decreases |order| - count
  {
    assert count < |order|;
    Bound(order, next.Length);
    count := count + 1;
    cur := next[cur];
  }
  r := count;
}
