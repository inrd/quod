// Experiment 0007 change task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

lemma SetOf(s: seq<usize>, N: int)
  requires forall i :: 0 <= i < |s| ==> s[i] < N
  requires forall i, j :: 0 <= i < j < |s| ==> s[i] != s[j]
  ensures |set i | 0 <= i < |s| :: s[i]| == |s|
  ensures (set i | 0 <= i < |s| :: s[i]) <= (set x: int | 0 <= x < N)
{
  if |s| == 0 {
    assert (set i | 0 <= i < |s| :: s[i]) == {};
  } else {
    var t := s[..|s|-1];
    SetOf(t, N);
    assert (set i | 0 <= i < |s| :: s[i]) == (set i | 0 <= i < |t| :: t[i]) + {s[|s|-1]} by {
      forall x | x in (set i | 0 <= i < |s| :: s[i]) ensures x in (set i | 0 <= i < |t| :: t[i]) + {s[|s|-1]} {
        var i :| 0 <= i < |s| && s[i] == x;
        if i < |s|-1 { assert t[i] == x; }
      }
      forall x | x in (set i | 0 <= i < |t| :: t[i]) ensures x in (set i | 0 <= i < |s| :: s[i]) {
        var i :| 0 <= i < |t| && t[i] == x;
        assert s[i] == x;
      }
    }
    assert s[|s|-1] !in (set i | 0 <= i < |t| :: t[i]) by {
      if s[|s|-1] in (set i | 0 <= i < |t| :: t[i]) {
        var i :| 0 <= i < |t| && t[i] == s[|s|-1];
        assert s[i] == s[|s|-1];
      }
    }
  }
}

lemma SetRange(N: nat)
  ensures |set x: int | 0 <= x < N| == N
{
  if N == 0 {
    assert (set x: int | 0 <= x < N) == {};
  } else {
    SetRange(N-1);
    assert (set x: int | 0 <= x < N) == (set x: int | 0 <= x < N-1) + {N-1};
  }
}

lemma LenBound(s: seq<usize>, N: int)
  requires N >= 0
  requires forall i :: 0 <= i < |s| ==> s[i] < N
  requires forall i, j :: 0 <= i < j < |s| ==> s[i] != s[j]
  ensures |s| <= N
{
  SetOf(s, N);
  SetRange(N);
  var A := set i | 0 <= i < |s| :: s[i];
  var B := set x: int | 0 <= x < N;
  assert A <= B;
  assert B == A + (B - A);
  assert |A| == |A| ;
  assert |B| == |A| + |B - A|;
}

// CHANGE: Reverses an index-linked list in a pool in place; returns the new head and
//   the list length (was: new head only). next[n] is the link of node n;
//   next.Length is the null index. order is the ghost list of nodes from head.
// SPEC-BEGIN
method pool_list_reverse(next: array<usize>, head: usize, ghost order: seq<usize>) returns (r: usize, len: usize)
  requires next.Length < 0x1_0000_0000_0000_0000
  requires forall i :: 0 <= i < |order| ==> order[i] < next.Length
  requires forall i, j :: 0 <= i < j < |order| ==> order[i] != order[j]
  requires head == (if |order| == 0 then next.Length else order[0])
  requires forall i :: 0 <= i < |order| - 1 ==> next[order[i]] == order[i + 1]
  requires |order| > 0 ==> next[order[|order| - 1]] == next.Length
  modifies next
  ensures len == |order|
  ensures r == (if |order| == 0 then next.Length else order[|order| - 1])
  ensures forall i :: 1 <= i < |order| ==> next[order[i]] == order[i - 1]
  ensures |order| > 0 ==> next[order[0]] == next.Length
  ensures forall n :: 0 <= n < next.Length && n !in order ==> next[n] == old(next[n])
// SPEC-END
{
  LenBound(order, next.Length);
  var N := next.Length;
  ghost var k := 0;
  var n: usize := 0;
  var prev: usize := N;
  var cur: usize := head;
  while cur != N
    invariant 0 <= k <= |order|
    invariant n == k
    invariant prev == (if k == 0 then N else order[k-1])
    invariant cur == (if k == |order| then N else order[k])
    invariant forall i :: 1 <= i < k ==> next[order[i]] == order[i-1]
    invariant k > 0 ==> next[order[0]] == N
    invariant forall i :: k <= i < |order| - 1 ==> next[order[i]] == order[i+1]
    invariant k < |order| ==> next[order[|order|-1]] == N
    invariant forall n :: 0 <= n < N && n !in order ==> next[n] == old(next[n])
    decreases |order| - k
  {
    var nx := next[cur];
    next[cur] := prev;
    prev := cur;
    cur := nx;
    k := k + 1;
    n := n + 1;
  }
  r := prev;
  len := n;
}
