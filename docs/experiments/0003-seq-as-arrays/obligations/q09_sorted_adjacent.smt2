(set-logic ALL)
; q09: sortedness implies adjacent order
(declare-const a (Array Int Int))(declare-const a_len Int)(declare-const i Int)
(assert (<= 0 a_len))
(assert (forall ((j Int) (k Int)) (=> (and (<= 0 j) (< j k) (< k a_len)) (<= (select a j) (select a k)))))
(assert (and (<= 0 i) (< (+ i 1) a_len)))
(assert (not (<= (select a i) (select a (+ i 1)))))
(check-sat)
