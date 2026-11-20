(set-logic ALL)
; route b, width 32: rotl(x, w/4) >> (3w/4) == (x >> w/2) & mask(w/4)
(declare-const x (_ BitVec 32))(declare-const y (_ BitVec 32))
(assert (not (= (bvlshr (bvor (bvshl x #x00000008) (bvlshr x #x00000018)) #x00000018) (bvand (bvlshr x #x00000010) #x000000ff))))(check-sat)
