// Experiment 0007 change task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// CHANGE: Sorts xs in place in descending order (was: ascending).
// SPEC-BEGIN
method insertion_sort(xs: array<u32>)
  requires xs.Length < 0x1_0000_0000_0000_0000
  modifies xs
  ensures forall i, j :: 0 <= i <= j < xs.Length ==> xs[i] >= xs[j]
  ensures multiset(xs[..]) == old(multiset(xs[..]))
// SPEC-END
{
  var i := 1;
  while i < xs.Length
    invariant 1 <= i || xs.Length == 0
    invariant i <= xs.Length || xs.Length == 0
    invariant forall a, b :: 0 <= a <= b < i && b < xs.Length ==> xs[a] >= xs[b]
    invariant multiset(xs[..]) == old(multiset(xs[..]))
  {
    var j := i;
    while j > 0 && xs[j-1] < xs[j]
      invariant 0 <= j <= i < xs.Length
      invariant forall a, b :: 0 <= a <= b <= i && a != j && b != j ==> xs[a] >= xs[b]
      invariant forall a, b :: 0 <= a < j && j < b <= i ==> xs[a] >= xs[b]
      invariant forall b :: j < b <= i ==> xs[j] >= xs[b]
      invariant multiset(xs[..]) == old(multiset(xs[..]))
    {
      xs[j-1], xs[j] := xs[j], xs[j-1];
      j := j - 1;
    }
    assert j == 0 || xs[j-1] >= xs[j];
    assert forall a, b :: 0 <= a <= b <= i ==> xs[a] >= xs[b] by {
      forall a, b | 0 <= a <= b <= i ensures xs[a] >= xs[b] {
        if a != j && b != j {
        } else if a == j && b == j {
        } else if a == j {
          // b > j
          if j > 0 { assert xs[j-1] >= xs[b]; }
        } else {
          // b == j, a < j
          if a < j - 1 { assert xs[a] >= xs[j-1]; }
        }
      }
    }
    i := i + 1;
  }
}
