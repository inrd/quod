(set-logic ALL)
; route b, width 8: x & mask(w/4) <= mask
(declare-const x (_ BitVec 8))(declare-const y (_ BitVec 8))
(assert (not (bvule (bvand x #x03) #x03)))(check-sat)
