// Experiment 0005 baseline task. Do not edit between SPEC-BEGIN and SPEC-END.
type u32 = x: int | 0 <= x < 0x1_0000_0000
type usize = x: int | 0 <= x < 0x1_0000_0000_0000_0000

// SPEC-BEGIN
method copy_at(dst: array<u32>, src: array<u32>, off: usize)
  requires dst.Length < 0x1_0000_0000_0000_0000 && src.Length < 0x1_0000_0000_0000_0000
  requires dst != src
  requires off + src.Length <= dst.Length
  modifies dst
  ensures forall k :: 0 <= k < src.Length ==> dst[off + k] == src[k]
  ensures forall k :: 0 <= k < off ==> dst[k] == old(dst[k])
  ensures forall k :: off + src.Length <= k < dst.Length ==> dst[k] == old(dst[k])
// SPEC-END
{
  var i := 0;
  while i < src.Length
    invariant 0 <= i <= src.Length
    invariant forall k :: 0 <= k < i ==> dst[off + k] == src[k]
    invariant forall k :: 0 <= k < off ==> dst[k] == old(dst[k])
    invariant forall k :: off + src.Length <= k < dst.Length ==> dst[k] == old(dst[k])
  {
    dst[off + i] := src[i];
    i := i + 1;
  }
}
