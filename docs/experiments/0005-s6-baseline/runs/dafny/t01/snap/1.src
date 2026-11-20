// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// SPEC-BEGIN
method max_index(xs: array<u32>) returns (r: usize)
  requires xs.Length < 0x1_0000_0000_0000_0000
  requires xs.Length > 0
  ensures r < xs.Length
  ensures forall k :: 0 <= k < xs.Length ==> xs[k] <= xs[r]
// SPEC-END
{
  var best: int := 0;
  var i: int := 1;
  while i < xs.Length
    invariant 1 <= i <= xs.Length
    invariant 0 <= best < i
    invariant forall k :: 0 <= k < i ==> xs[k] <= xs[best]
  {
    if xs[i] > xs[best] { best := i; }
    i := i + 1;
  }
  r := best;
}
