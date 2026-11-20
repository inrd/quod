(set-logic ALL)
; route b, width 8: rotl(x, w/4) >> (3w/4) == (x >> w/2) & mask(w/4)
(declare-const x (_ BitVec 8))(declare-const y (_ BitVec 8))
(assert (not (= (bvlshr (bvor (bvshl x #x02) (bvlshr x #x06)) #x06) (bvand (bvlshr x #x04) #x03))))(check-sat)
