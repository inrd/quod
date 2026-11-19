(set-logic ALL)
(define-fun-rec sumto ((n Int)) Int (ite (<= n 0) 0 (+ n (sumto (- n 1)))))
(assert (not (= (sumto 3) 6)))
(check-sat)
