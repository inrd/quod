(set-logic ALL)
; s05 (new): s04 with the view's quantifiers over base indices m = lo + k,
; so the trigger (select a m) has no arithmetic inside
(declare-const a (Array Int Int))(declare-const a_len Int)
(declare-const lo Int)(declare-const hi Int)(declare-const i Int)
(assert (<= 0 a_len))
(assert (and (<= 0 lo) (<= lo hi) (<= hi a_len)))
(assert (and (<= 0 i) (< i (- hi lo))))
(assert (forall ((m Int)) (=> (and (<= lo m) (< m (+ lo i))) (= (select a m) 0))))
(assert (not (forall ((m Int)) (=> (and (<= lo m) (< m (+ lo (+ i 1)))) (= (select (store a (+ lo i) 0) m) 0)))))
(check-sat)
