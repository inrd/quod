// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// Removes duplicates from a sorted array in place; returns the new length.
// SPEC-BEGIN
method dedup_sorted(xs: array<u32>) returns (k: usize)
  requires xs.Length < 0x1_0000_0000_0000_0000
  requires forall a, b :: 0 <= a <= b < xs.Length ==> xs[a] <= xs[b]
  modifies xs
  ensures k <= xs.Length
  ensures forall a, b :: 0 <= a < b < k ==> xs[a] < xs[b]
  ensures forall v :: v in old(xs[..]) <==> v in xs[..k]
// SPEC-END
{
  // ?h: implementation goes here
}
