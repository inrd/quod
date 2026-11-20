(set-logic ALL)
; route b, width 16: x << 1 == x + x
(declare-const x (_ BitVec 16))(declare-const y (_ BitVec 16))
(assert (not (= (bvshl x #x0001) (bvadd x x))))(check-sat)
