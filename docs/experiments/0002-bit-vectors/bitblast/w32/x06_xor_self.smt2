(set-logic ALL)
; route b, width 32: x ^ x == 0
(declare-const x (_ BitVec 32))(declare-const y (_ BitVec 32))
(assert (not (= (bvxor x x) #x00000000)))(check-sat)
