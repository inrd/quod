// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// SPEC-BEGIN
method insert_sorted(xs: array<u32>, n: usize, v: u32) returns (r: usize)
  requires xs.Length < 0x1_0000_0000_0000_0000
  requires n < xs.Length
  requires forall i :: 0 <= i < n ==> forall j :: i <= j < n ==> xs[i] <= xs[j]
  modifies xs
  ensures r <= n
  ensures xs[r] == v
  ensures forall k :: 0 <= k < r ==> xs[k] == old(xs[k])
  ensures forall k :: r + 1 <= k < n + 1 ==> xs[k] == old(xs[k - 1])
  ensures forall k :: n + 1 <= k < xs.Length ==> xs[k] == old(xs[k])
  ensures forall i :: 0 <= i < n + 1 ==> forall j :: i <= j < n + 1 ==> xs[i] <= xs[j]
// SPEC-END
{
  var i: usize := n;
  while i > 0 && xs[i-1] > v
    invariant 0 <= i <= n
    invariant forall k :: 0 <= k < i ==> xs[k] == old(xs[k])
    invariant forall k :: i + 1 <= k < n + 1 ==> xs[k] == old(xs[k - 1])
    invariant forall k :: n + 1 <= k < xs.Length ==> xs[k] == old(xs[k])
    invariant forall k :: i + 1 <= k < n + 1 ==> xs[k] > v
    decreases i
  {
    xs[i] := xs[i-1];
    i := i - 1;
  }
  xs[i] := v;
  r := i;
}
