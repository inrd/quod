(set-logic ALL)
; route b, width 64: x ^ x == 0
(declare-const x (_ BitVec 64))(declare-const y (_ BitVec 64))
(assert (not (= (bvxor x x) #x0000000000000000)))(check-sat)
