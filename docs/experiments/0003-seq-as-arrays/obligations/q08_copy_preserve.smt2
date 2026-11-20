(set-logic ALL)
; q08: copy loop invariant preserved after b[i] := a[i]
(declare-const a (Array Int Int))(declare-const a_len Int)
(declare-const b (Array Int Int))(declare-const b_len Int)(declare-const i Int)
(assert (and (<= 0 a_len) (<= 0 b_len)))
(assert (and (<= 0 i) (< i b_len) (= a_len b_len)))
(assert (forall ((k Int)) (=> (and (<= 0 k) (< k i)) (= (select b k) (select a k)))))
(assert (not (forall ((k Int)) (=> (and (<= 0 k) (< k (+ i 1))) (= (select (store b i (select a i)) k) (select a k))))))
(check-sat)
