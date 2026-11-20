(set-logic ALL)
; route b, width 16: x & mask(w/4) <= mask
(declare-const x (_ BitVec 16))(declare-const y (_ BitVec 16))
(assert (not (bvule (bvand x #x000f) #x000f)))(check-sat)
