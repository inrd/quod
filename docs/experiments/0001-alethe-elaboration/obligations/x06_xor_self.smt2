(set-logic QF_BV)
(declare-const x (_ BitVec 64))
(assert (not (= (bvxor x x) #x0000000000000000)))
(check-sat)
