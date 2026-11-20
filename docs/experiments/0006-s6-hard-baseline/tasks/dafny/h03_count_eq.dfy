// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// Counts the elements equal to key.
// SPEC-BEGIN
function count(s: seq<u32>, key: u32): nat
{
  if |s| == 0 then 0 else (if s[0] == key then 1 else 0) + count(s[1..], key)
}

method count_eq(xs: array<u32>, key: u32) returns (r: usize)
  requires xs.Length < 0x1_0000_0000_0000_0000
  ensures r == count(xs[..], key)
// SPEC-END
{
  // ?h: implementation goes here
}
