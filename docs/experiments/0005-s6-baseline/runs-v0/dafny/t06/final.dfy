// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// SPEC-BEGIN
method scale(xs: array<u32>, f: u32)
  requires xs.Length < 0x1_0000_0000_0000_0000
  requires forall k :: 0 <= k < xs.Length ==> xs[k] * f <= 4294967295
  modifies xs
  ensures forall k :: 0 <= k < xs.Length ==> xs[k] == old(xs[k]) * f
// SPEC-END
{
  var i := 0;
  while i < xs.Length
    invariant 0 <= i <= xs.Length
    invariant forall k :: 0 <= k < i ==> xs[k] == old(xs[k]) * f
    invariant forall k :: i <= k < xs.Length ==> xs[k] == old(xs[k])
  {
    xs[i] := xs[i] * f;
    i := i + 1;
  }
}
