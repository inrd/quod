(set-logic QF_BV)
(declare-const x (_ BitVec 32))
(assert (not (= (bvshl x #x00000001) (bvadd x x))))
(check-sat)
