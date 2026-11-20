(set-logic ALL)
; route b, width 64: x << 1 == x + x
(declare-const x (_ BitVec 64))(declare-const y (_ BitVec 64))
(assert (not (= (bvshl x #x0000000000000001) (bvadd x x))))(check-sat)
