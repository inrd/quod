(set-logic ALL)
; route b, width 32: x & mask(w/4) <= mask
(declare-const x (_ BitVec 32))(declare-const y (_ BitVec 32))
(assert (not (bvule (bvand x #x000000ff) #x000000ff)))(check-sat)
