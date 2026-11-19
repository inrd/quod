(set-logic ALL)
; Ring { head, len } push: len' = len + 1 under len < N
(declare-datatypes ((Ring 0)) (((mk (rhead Int) (rlen Int)))))
(declare-const rb Ring)(declare-const N Int)
(assert (and (<= 0 (rlen rb)) (< (rlen rb) N)))
(assert (not (let ((rb2 (mk (rhead rb) (+ (rlen rb) 1)))) (and (= (rlen rb2) (+ (rlen rb) 1)) (<= (rlen rb2) N)))))
(check-sat)
