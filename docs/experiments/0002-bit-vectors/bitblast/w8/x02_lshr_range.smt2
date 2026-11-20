(set-logic ALL)
; route b, width 8: x >> 4 < 2^(w-4)
(declare-const x (_ BitVec 8))(declare-const y (_ BitVec 8))
(assert (not (bvult (bvlshr x #x04) #x10)))(check-sat)
