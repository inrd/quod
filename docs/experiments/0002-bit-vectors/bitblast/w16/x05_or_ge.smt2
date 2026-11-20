(set-logic ALL)
; route b, width 16: x | y >= x
(declare-const x (_ BitVec 16))(declare-const y (_ BitVec 16))
(assert (not (bvuge (bvor x y) x)))(check-sat)
