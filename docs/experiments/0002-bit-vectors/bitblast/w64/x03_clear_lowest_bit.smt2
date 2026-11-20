(set-logic ALL)
; route b, width 64: x != 0 => x & (x-1) < x
(declare-const x (_ BitVec 64))(declare-const y (_ BitVec 64))
(assert (distinct x #x0000000000000000))(assert (not (bvult (bvand x (bvsub x #x0000000000000001)) x)))(check-sat)
