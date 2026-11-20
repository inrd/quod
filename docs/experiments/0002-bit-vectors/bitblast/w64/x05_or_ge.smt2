(set-logic ALL)
; route b, width 64: x | y >= x
(declare-const x (_ BitVec 64))(declare-const y (_ BitVec 64))
(assert (not (bvuge (bvor x y) x)))(check-sat)
