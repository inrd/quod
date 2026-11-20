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
  var hi: int := xs.Length - 1;
  while lo < hi
    invariant 0 <= lo && hi < xs.Length && lo <= hi + 1
    invariant forall a, b :: 0 <= a < b < xs.Length && (a < lo || b > hi) ==> xs[a] as int + xs[b] as int != target
    decreases hi - lo
  {
    var s := xs[lo] as int + xs[hi] as int;
    if s == target {
      found := true; i := lo; j := hi;
      return;
    } else if s < target {
      forall b | lo < b <= hi ensures xs[lo] as int + xs[b] as int != target {
        assert xs[b] <= xs[hi];
      }
      lo := lo + 1;
    } else {
      forall a | lo <= a < hi ensures xs[a] as int + xs[hi] as int != target {
        assert xs[lo] <= xs[a];
      }
      hi := hi - 1;
    }
  }
}
