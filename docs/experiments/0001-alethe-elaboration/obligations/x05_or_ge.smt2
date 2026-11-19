(set-logic QF_BV)
(declare-const x (_ BitVec 32))(declare-const y (_ BitVec 32))
(assert (not (bvuge (bvor x y) x)))
(check-sat)
