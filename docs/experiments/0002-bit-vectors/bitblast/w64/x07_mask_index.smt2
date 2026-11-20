(set-logic ALL)
; route b, width 64: x & 0xf < 16
(declare-const x (_ BitVec 64))(declare-const y (_ BitVec 64))
(assert (not (bvult (bvand x #x000000000000000f) #x0000000000000010)))(check-sat)
