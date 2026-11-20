(set-logic ALL)
; route b, width 8: x ^ x == 0
(declare-const x (_ BitVec 8))(declare-const y (_ BitVec 8))
(assert (not (= (bvxor x x) #x00)))(check-sat)
