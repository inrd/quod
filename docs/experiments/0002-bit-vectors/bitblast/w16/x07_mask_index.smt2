(set-logic ALL)
; route b, width 16: x & 0xf < 16
(declare-const x (_ BitVec 16))(declare-const y (_ BitVec 16))
(assert (not (bvult (bvand x #x000f) #x0010)))(check-sat)
