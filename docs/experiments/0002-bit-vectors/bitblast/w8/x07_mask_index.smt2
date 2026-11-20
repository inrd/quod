(set-logic ALL)
; route b, width 8: x & 0xf < 16
(declare-const x (_ BitVec 8))(declare-const y (_ BitVec 8))
(assert (not (bvult (bvand x #x0f) #x10)))(check-sat)
