(set-logic QF_BV)
(declare-const x (_ BitVec 32))
(assert (not (= (bvlshr (bvor (bvshl x #x00000008) (bvlshr x #x00000018)) #x00000018) (bvand (bvlshr x #x00000010) #x000000ff))))
(check-sat)
