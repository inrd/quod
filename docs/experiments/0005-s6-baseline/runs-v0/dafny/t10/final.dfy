// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// Pool free list: next[k] is the link of node k, next.Length is the null index;
// free[0] is the free-list head.
// SPEC-BEGIN
method pool_alloc(next: array<usize>, free: array<usize>) returns (r: usize)
  requires next.Length < 0x1_0000_0000_0000_0000
  requires next != free
  requires free.Length == 1
  requires free[0] <= next.Length
  requires forall k :: 0 <= k < next.Length ==> next[k] <= next.Length
  modifies free
  ensures old(free[0]) == next.Length ==> r == next.Length && free[0] == next.Length
  ensures old(free[0]) < next.Length ==> r == old(free[0]) && free[0] == next[r]
  ensures free[0] <= next.Length
// SPEC-END
{
  r := free[0];
  if r < next.Length {
    free[0] := next[r];
  }
}
