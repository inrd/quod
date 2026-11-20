(set-logic ALL)
; q15 (new): copy loop exit implies b == a. Sequence equality is emitted as
; equal lengths plus pointwise equality on [0, len), not as array equality.
(declare-const a (Array Int Int))(declare-const a_len Int)
(declare-const b (Array Int Int))(declare-const b_len Int)(declare-const i Int)
(assert (and (<= 0 a_len) (<= 0 b_len)))
(assert (and (= i b_len) (= a_len b_len)))
(assert (forall ((k Int)) (=> (and (<= 0 k) (< k i)) (= (select b k) (select a k)))))
(assert (not (and (= b_len a_len) (forall ((k Int)) (=> (and (<= 0 k) (< k b_len)) (= (select b k) (select a k)))))))
(check-sat)
