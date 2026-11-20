(set-logic ALL)
; q02: loop invariant holds vacuously at i = 0
(declare-const a (Array Int Int))(declare-const a_len Int)(declare-const i Int)
(assert (<= 0 a_len))
(assert (= i 0))
(assert (not (forall ((k Int)) (=> (and (<= 0 k) (< k i)) (= (select a k) 0)))))
(check-sat)
