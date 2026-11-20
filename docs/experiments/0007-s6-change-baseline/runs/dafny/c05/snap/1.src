// Experiment 0007 change task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// CHANGE: Finds i < j with xs[j] - xs[i] == target in a sorted array (two pointers, linear time)
//   (was: xs[i] + xs[j] == target).
// SPEC-BEGIN
method two_diff_sorted(xs: array<u32>, target: u64) returns (found: bool, i: usize, j: usize)
  requires xs.Length < 0x1_0000_0000_0000_0000
  requires forall a, b :: 0 <= a <= b < xs.Length ==> xs[a] <= xs[b]
  ensures found ==> i < j < xs.Length && xs[j] as int - xs[i] as int == target
  ensures !found ==> forall a, b :: 0 <= a < b < xs.Length ==> xs[b] as int - xs[a] as int != target
// SPEC-END
{
  found := false; i := 0; j := 0;
  var lo: int := 0;
  var hi: int := 0;
  while hi < xs.Length
    invariant 0 <= lo <= hi <= xs.Length
    invariant forall a, b :: 0 <= a < b < xs.Length && (a < lo || b < hi) ==> xs[b] as int - xs[a] as int != target
    decreases xs.Length - hi, hi - lo
  {
    if lo == hi {
      hi := hi + 1;
    } else {
      var d := xs[hi] as int - xs[lo] as int;
      if d == target {
        found := true; i := lo; j := hi;
        return;
      } else if d < target {
        forall a | lo <= a < hi ensures xs[hi] as int - xs[a] as int != target {
          assert xs[lo] <= xs[a];
        }
        hi := hi + 1;
      } else {
        forall b | hi <= b < xs.Length ensures xs[b] as int - xs[lo] as int != target {
          assert xs[hi] <= xs[b];
        }
        lo := lo + 1;
      }
    }
  }
}
