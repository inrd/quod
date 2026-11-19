(set-logic QF_BV)
(declare-const h (_ BitVec 32))
(assert (not (bvult (bvand h #x0000000f) #x00000010)))
(check-sat)
