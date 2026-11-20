(set-logic ALL)
; route b, width 8: x | y >= x
(declare-const x (_ BitVec 8))(declare-const y (_ BitVec 8))
(assert (not (bvuge (bvor x y) x)))(check-sat)
