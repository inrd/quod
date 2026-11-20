(set-logic ALL)
; route b, width 32: x & 0xf < 16
(declare-const x (_ BitVec 32))(declare-const y (_ BitVec 32))
(assert (not (bvult (bvand x #x0000000f) #x00000010)))(check-sat)
