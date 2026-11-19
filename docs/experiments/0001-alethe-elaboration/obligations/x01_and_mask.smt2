(set-logic QF_BV)
(declare-const x (_ BitVec 32))
(assert (not (bvule (bvand x #x000000ff) #x000000ff)))
(check-sat)
