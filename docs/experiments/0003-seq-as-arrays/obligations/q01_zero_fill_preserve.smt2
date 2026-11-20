(set-logic ALL)
; q01: zero-fill loop invariant preserved after a[i] := 0
(declare-const a (Array Int Int))(declare-const a_len Int)(declare-const i Int)
(assert (<= 0 a_len))
(assert (and (<= 0 i) (< i a_len)))
(assert (forall ((k Int)) (=> (and (<= 0 k) (< k i)) (= (select a k) 0))))
(assert (not (forall ((k Int)) (=> (and (<= 0 k) (< k (+ i 1))) (= (select (store a i 0) k) 0)))))
(check-sat)
