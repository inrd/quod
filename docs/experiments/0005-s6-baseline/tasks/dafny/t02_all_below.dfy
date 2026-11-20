// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// SPEC-BEGIN
method all_below(xs: array<u32>, bound: u32) returns (r: bool)
  requires xs.Length < 0x1_0000_0000_0000_0000
  ensures r ==> forall k :: 0 <= k < xs.Length ==> xs[k] < bound
  ensures !r ==> exists k :: 0 <= k < xs.Length && xs[k] >= bound
// SPEC-END
{
  // ?h: implementation goes here
}
