#!/bin/sh
# Regenerates lemma-library/ (route a2: library axioms instead of instances)
# and bitblast/w{8,16,32,64}/ (route b) from lemma-instances/ and this script.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd); cd "$HERE"
mkdir -p lemma-library
for f in lemma-instances/*.smt2; do
  n=$(basename "$f"); grep -v '; inst' "$f" | grep -v '^(check-sat)' >lemma-library/$n
  grep -q xor64 lemma-library/$n || echo '(declare-fun xor64 (Int Int) Int)' >>lemma-library/$n
  cat library.smt2 >>lemma-library/$n; echo '(check-sat)' >>lemma-library/$n
done
hex() { printf "#x%0$(( $1 / 4 ))x" "$2"; }   # hex <width> <value>
for w in 8 16 32 64; do
  d=bitblast/w$w; mkdir -p $d; r=$((w / 4)); m=$(( (1 << r) - 1 ))
  if [ $w = 64 ]; then top=$(printf '#x%016x' $(( (1 << 60) )) ); else top=$(hex $w $(( 1 << (w - 4) ))); fi
  hdr() { printf '(set-logic ALL)\n; route b, width %s: %s\n(declare-const x (_ BitVec %s))(declare-const y (_ BitVec %s))\n' $w "$1" $w $w; }
  { hdr "x & mask(w/4) <= mask"; echo "(assert (not (bvule (bvand x $(hex $w $m)) $(hex $w $m))))(check-sat)"; } >$d/x01_and_mask.smt2
  { hdr "x >> 4 < 2^(w-4)"; echo "(assert (not (bvult (bvlshr x $(hex $w 4)) $top)))(check-sat)"; } >$d/x02_lshr_range.smt2
  { hdr "x != 0 => x & (x-1) < x"; echo "(assert (distinct x $(hex $w 0)))(assert (not (bvult (bvand x (bvsub x $(hex $w 1))) x)))(check-sat)"; } >$d/x03_clear_lowest_bit.smt2
  { hdr "x << 1 == x + x"; echo "(assert (not (= (bvshl x $(hex $w 1)) (bvadd x x))))(check-sat)"; } >$d/x04_shl_is_double.smt2
  { hdr "x | y >= x"; echo "(assert (not (bvuge (bvor x y) x)))(check-sat)"; } >$d/x05_or_ge.smt2
  { hdr "x ^ x == 0"; echo "(assert (not (= (bvxor x x) $(hex $w 0))))(check-sat)"; } >$d/x06_xor_self.smt2
  { hdr "x & 0xf < 16"; echo "(assert (not (bvult (bvand x $(hex $w 15)) $(hex $w 16))))(check-sat)"; } >$d/x07_mask_index.smt2
  { hdr "rotl(x, w/4) >> (3w/4) == (x >> w/2) & mask(w/4)"; echo "(assert (not (= (bvlshr (bvor (bvshl x $(hex $w $r)) (bvlshr x $(hex $w $((w - r))))) $(hex $w $((w - r)))) (bvand (bvlshr x $(hex $w $((w / 2)))) $(hex $w $m)))))(check-sat)"; } >$d/x08_rotate.smt2
done
