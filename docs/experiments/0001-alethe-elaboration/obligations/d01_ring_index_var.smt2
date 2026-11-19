(set-logic ALL)
; (head + len) % N < N, N variable
(declare-const h Int)(declare-const l Int)(declare-const N Int)
(assert (and (< 0 N) (<= 0 h) (< h N) (<= 0 l) (<= l N)))
(assert (not (and (<= 0 (mod (+ h l) N)) (< (mod (+ h l) N) N))))
(check-sat)
