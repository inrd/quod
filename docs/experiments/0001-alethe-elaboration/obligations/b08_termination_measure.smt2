(set-logic ALL)
; decreases n - i
(declare-const i Int)(declare-const n Int)
(assert (< i n))
(assert (not (and (>= (- n i) 0) (< (- n (+ i 1)) (- n i)))))
(check-sat)
