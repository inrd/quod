(set-logic ALL)
; route b, width 32: x != 0 => x & (x-1) < x
(declare-const x (_ BitVec 32))(declare-const y (_ BitVec 32))
(assert (distinct x #x00000000))(assert (not (bvult (bvand x (bvsub x #x00000001)) x)))(check-sat)
