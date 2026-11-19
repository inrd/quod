(set-logic ALL)
; invariant 0 <= i <= n preserved by i' = i + 1 under guard i < n
(declare-const i Int)(declare-const n Int)
(assert (and (<= 0 i) (<= i n) (< i n)))
(assert (not (and (<= 0 (+ i 1)) (<= (+ i 1) n))))
(check-sat)
