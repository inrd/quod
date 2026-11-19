(set-logic ALL)
; s = &p[lo..hi]; s[j] is p[lo + j]
(declare-const lo Int)(declare-const hi Int)(declare-const len Int)(declare-const j Int)
(assert (and (<= 0 lo) (<= lo hi) (<= hi len) (<= 0 j) (< j (- hi lo))))
(assert (not (and (<= 0 (+ lo j)) (< (+ lo j) len))))
(check-sat)
