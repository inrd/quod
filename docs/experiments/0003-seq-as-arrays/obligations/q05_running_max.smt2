(set-logic ALL)
; q05: running-maximum invariant preserved
(declare-const a (Array Int Int))(declare-const a_len Int)(declare-const i Int)(declare-const m Int)
(assert (<= 0 a_len))
(assert (and (<= 0 i) (< i a_len)))
(assert (forall ((k Int)) (=> (and (<= 0 k) (< k i)) (>= m (select a k)))))
(assert (not (forall ((k Int)) (=> (and (<= 0 k) (< k (+ i 1))) (>= (ite (> (select a i) m) (select a i) m) (select a k))))))
(check-sat)
