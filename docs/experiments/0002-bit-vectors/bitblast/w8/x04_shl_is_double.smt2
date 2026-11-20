(set-logic ALL)
; route b, width 8: x << 1 == x + x
(declare-const x (_ BitVec 8))(declare-const y (_ BitVec 8))
(assert (not (= (bvshl x #x01) (bvadd x x))))(check-sat)
