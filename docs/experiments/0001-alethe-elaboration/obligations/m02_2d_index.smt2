(set-logic ALL)
; r * C + c < R * C
(declare-const r Int)(declare-const c Int)(declare-const R Int)(declare-const C Int)
(assert (and (<= 0 r) (< r R) (<= 0 c) (< c C)))
(assert (not (< (+ (* r C) c) (* R C))))
(check-sat)
