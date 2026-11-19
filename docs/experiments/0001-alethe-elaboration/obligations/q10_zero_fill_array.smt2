(set-logic ALL)
; q01 with SMT-LIB arrays instead of sequences
(declare-const a (Array Int Int))(declare-const i Int)(declare-const len Int)
(assert (and (<= 0 i) (< i len)))
(assert (forall ((k Int)) (=> (and (<= 0 k) (< k i)) (= (select a k) 0))))
(assert (not (forall ((k Int)) (=> (and (<= 0 k) (< k (+ i 1))) (= (select (store a i 0) k) 0)))))
(check-sat)
