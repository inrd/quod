// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// Sorts xs in place.
// SPEC-BEGIN
method insertion_sort(xs: array<u32>)
  requires xs.Length < 0x1_0000_0000_0000_0000
  modifies xs
  ensures forall i, j :: 0 <= i <= j < xs.Length ==> xs[i] <= xs[j]
  ensures multiset(xs[..]) == old(multiset(xs[..]))
// SPEC-END
{
  // ?h: implementation goes here
}
