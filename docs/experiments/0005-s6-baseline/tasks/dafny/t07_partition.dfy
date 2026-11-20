// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// Rearranges xs in place (a permutation; not stated in the spec, which
// matches the Quod slice subset, but required of the implementation).
// SPEC-BEGIN
method partition(xs: array<u32>, pivot: u32) returns (r: usize)
  requires xs.Length < 0x1_0000_0000_0000_0000
  modifies xs
  ensures r <= xs.Length
  ensures forall k :: 0 <= k < r ==> xs[k] < pivot
  ensures forall k :: r <= k < xs.Length ==> xs[k] >= pivot
// SPEC-END
{
  // ?h: implementation goes here
}
