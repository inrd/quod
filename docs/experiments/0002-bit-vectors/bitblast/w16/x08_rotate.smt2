(set-logic ALL)
; route b, width 16: rotl(x, w/4) >> (3w/4) == (x >> w/2) & mask(w/4)
(declare-const x (_ BitVec 16))(declare-const y (_ BitVec 16))
(assert (not (= (bvlshr (bvor (bvshl x #x0004) (bvlshr x #x000c)) #x000c) (bvand (bvlshr x #x0008) #x000f))))(check-sat)
