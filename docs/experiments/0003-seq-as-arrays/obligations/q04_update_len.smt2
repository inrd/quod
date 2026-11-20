(set-logic ALL)
; q04: update preserves length. With an explicit length the generator gives
; a[i := v] the length term a_len, so the obligation is syntactic.
(declare-const a (Array Int Int))(declare-const a_len Int)(declare-const i Int)(declare-const v Int)
(declare-const b (Array Int Int))(declare-const b_len Int)
(assert (<= 0 a_len))
(assert (and (<= 0 i) (< i a_len)))
(assert (and (= b (store a i v)) (= b_len a_len)))
(assert (not (= b_len a_len)))
(check-sat)
