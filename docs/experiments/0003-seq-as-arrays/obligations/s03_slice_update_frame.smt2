(set-logic ALL)
; s03 (new): writing s[i] through s = &mut a[lo..hi] leaves a[j] for j < lo unchanged
(declare-const a (Array Int Int))(declare-const a_len Int)
(declare-const lo Int)(declare-const hi Int)(declare-const i Int)(declare-const j Int)(declare-const v Int)
(assert (<= 0 a_len))
(assert (and (<= 0 lo) (<= lo hi) (<= hi a_len)))
(assert (and (<= 0 i) (< i (- hi lo)) (<= 0 j) (< j lo)))
(assert (not (= (select (store a (+ lo i) v) j) (select a j))))
(check-sat)
