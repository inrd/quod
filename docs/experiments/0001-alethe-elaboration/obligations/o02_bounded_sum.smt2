(set-logic ALL)
; u32 acc; invariant acc <= 1000 * i; elements <= 1000; n <= 1000
(declare-const acc Int)(declare-const i Int)(declare-const n Int)(declare-const a Int)
(assert (and (<= 0 acc) (<= acc (* 1000 i)) (<= 0 i) (< i n) (<= n 1000) (<= 0 a) (<= a 1000)))
(assert (not (and (<= (+ acc a) 4294967295) (<= (+ acc a) (* 1000 (+ i 1))))))
(check-sat)
