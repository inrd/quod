// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// SPEC-BEGIN
method lower_bound(xs: array<u32>, key: u32) returns (r: usize)
  requires xs.Length < 0x1_0000_0000_0000_0000
  requires forall i :: 0 <= i < xs.Length ==> forall j :: i <= j < xs.Length ==> xs[i] <= xs[j]
  ensures r <= xs.Length
  ensures forall k :: 0 <= k < r ==> xs[k] < key
  ensures forall k :: r <= k < xs.Length ==> xs[k] >= key
// SPEC-END
{
  var lo: int := 0;
  var hi: int := xs.Length;
  while lo < hi
    invariant 0 <= lo <= hi <= xs.Length
    invariant forall k :: 0 <= k < lo ==> xs[k] < key
    invariant forall k :: hi <= k < xs.Length ==> xs[k] >= key
    decreases hi - lo
  {
    var mid := lo + (hi - lo) / 2;
    if xs[mid] < key {
      lo := mid + 1;
    } else {
      hi := mid;
    }
  }
  r := lo;
}
