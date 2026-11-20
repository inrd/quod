(set-logic ALL)
; q03: invariant at exit (i = len) implies postcondition
(declare-const a (Array Int Int))(declare-const a_len Int)(declare-const i Int)
(assert (<= 0 a_len))
(assert (= i a_len))
(assert (forall ((k Int)) (=> (and (<= 0 k) (< k i)) (= (select a k) 0))))
(assert (not (forall ((k Int)) (=> (and (<= 0 k) (< k a_len)) (= (select a k) 0)))))
(check-sat)
