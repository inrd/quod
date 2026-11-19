(set-logic ALL)
; u32 + u32 computed in u64
(declare-const x Int)(declare-const y Int)
(assert (and (<= 0 x) (<= x 4294967295) (<= 0 y) (<= y 4294967295)))
(assert (not (<= (+ x y) 18446744073709551615)))
(check-sat)
