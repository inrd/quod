(set-logic ALL)
; route b, width 64: x >> 4 < 2^(w-4)
(declare-const x (_ BitVec 64))(declare-const y (_ BitVec 64))
(assert (not (bvult (bvlshr x #x0000000000000004) #x1000000000000000)))(check-sat)
