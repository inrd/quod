// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// Finds i < j with xs[i] as int + xs[j] as int == target in a sorted array (two pointers, linear time).
// SPEC-BEGIN
method two_sum_sorted(xs: array<u32>, target: u64) returns (found: bool, i: usize, j: usize)
  requires xs.Length < 0x1_0000_0000_0000_0000
  requires forall a, b :: 0 <= a <= b < xs.Length ==> xs[a] <= xs[b]
  ensures found ==> i < j < xs.Length && xs[i] as int + xs[j] as int == target
  ensures !found ==> forall a, b :: 0 <= a < b < xs.Length ==> xs[a] as int + xs[b] as int != target
// SPEC-END
{
  // ?h: implementation goes here
}
