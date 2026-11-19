(set-logic QF_BV)
(declare-const x (_ BitVec 32))
(assert (not (bvult (bvlshr x #x00000004) #x10000000)))
(check-sat)
