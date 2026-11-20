(set-logic ALL)
; route b, width 16: x >> 4 < 2^(w-4)
(declare-const x (_ BitVec 16))(declare-const y (_ BitVec 16))
(assert (not (bvult (bvlshr x #x0004) #x1000)))(check-sat)
