(set-logic ALL)
; route b, width 32: x >> 4 < 2^(w-4)
(declare-const x (_ BitVec 32))(declare-const y (_ BitVec 32))
(assert (not (bvult (bvlshr x #x00000004) #x10000000)))(check-sat)
