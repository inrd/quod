(set-logic ALL)
; route b, width 64: rotl(x, w/4) >> (3w/4) == (x >> w/2) & mask(w/4)
(declare-const x (_ BitVec 64))(declare-const y (_ BitVec 64))
(assert (not (= (bvlshr (bvor (bvshl x #x0000000000000010) (bvlshr x #x0000000000000030)) #x0000000000000030) (bvand (bvlshr x #x0000000000000020) #x000000000000ffff))))(check-sat)
