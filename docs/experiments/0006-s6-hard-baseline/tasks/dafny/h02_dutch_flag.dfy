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
  // ?h: implementation goes here
}
