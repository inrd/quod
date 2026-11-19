(set-logic ALL)
(declare-const x (_ BitVec 32))
(assert (not (< (ubv_to_int (bvlshr x #x00000018)) 256)))
(check-sat)
