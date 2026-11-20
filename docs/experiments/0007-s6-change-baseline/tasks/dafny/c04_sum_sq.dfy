// Experiment 0007 change task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type u64 = x: int | 0 <= x < 0x1_0000_0000_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// CHANGE: Sums the squares of the elements into a 64-bit result (was: plain sum);
//   elements are below 2^16.
// SPEC-BEGIN
function sum_sq(s: seq<u32>): int
{
  if |s| == 0 then 0 else s[0] * s[0] + sum_sq(s[1..])
}

method sum_sq_u64(xs: array<u32>) returns (r: u64)
  requires xs.Length < 0x1_0000_0000
  requires forall k :: 0 <= k < xs.Length ==> xs[k] < 0x1_0000
  ensures r == sum_sq(xs[..])
// SPEC-END
{
  var i := 0;
  r := 0;
  while i < xs.Length
    invariant 0 <= i <= xs.Length
    invariant r == sum(xs[..i])
    invariant r <= i * 0xFFFF_FFFF
  {
    SumApp(xs[..i], [xs[i]]);
    assert xs[..i+1] == xs[..i] + [xs[i]];
    r := r + xs[i];
    i := i + 1;
  }
  assert xs[..i] == xs[..];
}

lemma SumApp(a: seq<u32>, b: seq<u32>)
  ensures sum(a + b) == sum(a) + sum(b)
  decreases |a|
{
  if |a| == 0 {
    assert a + b == b;
  } else {
    assert (a + b)[1..] == a[1..] + b;
    SumApp(a[1..], b);
  }
}
