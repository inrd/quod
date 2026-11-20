// Experiment 0007 change task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// CHANGE: Merges two sorted arrays into a caller-provided output array
//   (was: into a newly allocated one).
// SPEC-BEGIN
method merge(a: array<u32>, b: array<u32>, out: array<u32>)
  requires a.Length + b.Length < 0x1_0000_0000_0000_0000
  requires out.Length == a.Length + b.Length
  requires out != a && out != b
  requires forall i, j :: 0 <= i <= j < a.Length ==> a[i] <= a[j]
  requires forall i, j :: 0 <= i <= j < b.Length ==> b[i] <= b[j]
  modifies out
  ensures forall i, j :: 0 <= i <= j < out.Length ==> out[i] <= out[j]
  ensures multiset(out[..]) == multiset(a[..]) + multiset(b[..])
// SPEC-END
{
  out := new u32[a.Length + b.Length](_ => 0);
  var i := 0;
  var j := 0;
  while i < a.Length || j < b.Length
    invariant 0 <= i <= a.Length && 0 <= j <= b.Length
    invariant forall p, q :: 0 <= p <= q < i + j ==> out[p] <= out[q]
    invariant multiset(out[..i+j]) == multiset(a[..i]) + multiset(b[..j])
    invariant forall p :: 0 <= p < i + j ==> (i < a.Length ==> out[p] <= a[i]) && (j < b.Length ==> out[p] <= b[j])
    decreases a.Length + b.Length - i - j
  {
    var k := i + j;
    if j == b.Length || (i < a.Length && a[i] <= b[j]) {
      out[k] := a[i];
      assert a[..i+1] == a[..i] + [a[i]];
      assert out[..k+1] == out[..k] + [out[k]];
      i := i + 1;
    } else {
      out[k] := b[j];
      assert b[..j+1] == b[..j] + [b[j]];
      assert out[..k+1] == out[..k] + [out[k]];
      j := j + 1;
    }
  }
  assert a[..i] == a[..] && b[..j] == b[..] && out[..i+j] == out[..];
}
