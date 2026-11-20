(set-logic ALL)
; route b, width 8: x != 0 => x & (x-1) < x
(declare-const x (_ BitVec 8))(declare-const y (_ BitVec 8))
(assert (distinct x #x00))(assert (not (bvult (bvand x (bvsub x #x01)) x)))(check-sat)
