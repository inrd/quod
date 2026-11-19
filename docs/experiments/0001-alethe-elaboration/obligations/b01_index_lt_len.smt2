(set-logic ALL)
; xs[i] with 0 <= i < n, n <= xs.len
(declare-const i Int)(declare-const n Int)(declare-const len Int)
(assert (and (<= 0 i) (< i n) (<= n len)))
(assert (not (< i len)))
(check-sat)
