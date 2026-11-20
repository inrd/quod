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
  var i := 0;
  var j := 0;
  while j < xs.Length
    invariant 0 <= i <= j <= xs.Length
    invariant forall k :: 0 <= k < i ==> xs[k] < pivot
    invariant forall k :: i <= k < j ==> xs[k] >= pivot
    decreases xs.Length - j
  {
    if xs[j] < pivot {
      var t := xs[i];
      xs[i] := xs[j];
      xs[j] := t;
      i := i + 1;
    }
    j := j + 1;
  }
  r := i;
}
