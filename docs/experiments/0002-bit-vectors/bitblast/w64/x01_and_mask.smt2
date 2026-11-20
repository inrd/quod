(set-logic ALL)
; route b, width 64: x & mask(w/4) <= mask
(declare-const x (_ BitVec 64))(declare-const y (_ BitVec 64))
(assert (not (bvule (bvand x #x000000000000ffff) #x000000000000ffff)))(check-sat)
