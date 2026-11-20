(set-logic ALL)
; route b, width 16: x != 0 => x & (x-1) < x
(declare-const x (_ BitVec 16))(declare-const y (_ BitVec 16))
(assert (distinct x #x0000))(assert (not (bvult (bvand x (bvsub x #x0001)) x)))(check-sat)
