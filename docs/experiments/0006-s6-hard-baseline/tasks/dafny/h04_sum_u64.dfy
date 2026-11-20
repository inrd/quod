// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// Sums the elements into a 64-bit result.
// SPEC-BEGIN
function sum(s: seq<u32>): int
{
  if |s| == 0 then 0 else s[0] + sum(s[1..])
}

method sum_u64(xs: array<u32>) returns (r: u64)
  requires xs.Length < 0x1_0000_0000
  ensures r == sum(xs[..])
// SPEC-END
{
  // ?h: implementation goes here
}
