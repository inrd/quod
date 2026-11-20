// Experiment 0006 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// Merges two sorted arrays into a new sorted array.
// SPEC-BEGIN
method merge(a: array<u32>, b: array<u32>) returns (out: array<u32>)
  requires a.Length + b.Length < 0x1_0000_0000_0000_0000
  requires forall i, j :: 0 <= i <= j < a.Length ==> a[i] <= a[j]
  requires forall i, j :: 0 <= i <= j < b.Length ==> b[i] <= b[j]
  ensures fresh(out)
  ensures out.Length == a.Length + b.Length
  ensures forall i, j :: 0 <= i <= j < out.Length ==> out[i] <= out[j]
  ensures multiset(out[..]) == multiset(a[..]) + multiset(b[..])
// SPEC-END
{
  // ?h: implementation goes here
}
