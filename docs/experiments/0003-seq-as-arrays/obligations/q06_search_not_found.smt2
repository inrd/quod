(set-logic ALL)
; q06: linear search, "not found so far" invariant preserved
(declare-const a (Array Int Int))(declare-const a_len Int)(declare-const i Int)(declare-const x Int)
(assert (<= 0 a_len))
(assert (and (<= 0 i) (< i a_len) (distinct (select a i) x)))
(assert (forall ((k Int)) (=> (and (<= 0 k) (< k i)) (distinct (select a k) x))))
(assert (not (forall ((k Int)) (=> (and (<= 0 k) (< k (+ i 1))) (distinct (select a k) x)))))
(check-sat)
