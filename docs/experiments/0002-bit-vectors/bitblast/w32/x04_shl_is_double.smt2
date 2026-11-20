(set-logic ALL)
; route b, width 32: x << 1 == x + x
(declare-const x (_ BitVec 32))(declare-const y (_ BitVec 32))
(assert (not (= (bvshl x #x00000001) (bvadd x x))))(check-sat)
