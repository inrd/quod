(set-logic ALL)
; route b, width 16: x ^ x == 0
(declare-const x (_ BitVec 16))(declare-const y (_ BitVec 16))
(assert (not (= (bvxor x x) #x0000)))(check-sat)
