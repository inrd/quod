// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// Three-way partition of values 0, 1, 2 in one pass, by swapping.
// SPEC-BEGIN
method dutch_flag(xs: array<u32>) returns (lo: usize, hi: usize)
  requires xs.Length < 0x1_0000_0000_0000_0000
  requires forall k :: 0 <= k < xs.Length ==> xs[k] <= 2
  modifies xs
  ensures lo <= hi <= xs.Length
  ensures forall k :: 0 <= k < lo ==> xs[k] == 0
  ensures forall k :: lo <= k < hi ==> xs[k] == 1
  ensures forall k :: hi <= k < xs.Length ==> xs[k] == 2
  ensures multiset(xs[..]) == old(multiset(xs[..]))
// SPEC-END
{
  var l: int := 0;
  var m: int := 0;
  var h: int := xs.Length;
  while m < h
    invariant 0 <= l <= m <= h <= xs.Length
    invariant forall k :: 0 <= k < l ==> xs[k] == 0
    invariant forall k :: l <= k < m ==> xs[k] == 1
    invariant forall k :: h <= k < xs.Length ==> xs[k] == 2
    invariant forall k :: 0 <= k < xs.Length ==> xs[k] <= 2
    invariant multiset(xs[..]) == old(multiset(xs[..]))
    decreases h - m
  {
    if xs[m] == 0 {
      xs[m], xs[l] := xs[l], xs[m];
      l := l + 1;
      m := m + 1;
    } else if xs[m] == 1 {
      m := m + 1;
    } else {
      h := h - 1;
      xs[m], xs[h] := xs[h], xs[m];
    }
  }
  lo := l;
  hi := m;
}
