(set-logic ALL)
; route b, width 32: x | y >= x
(declare-const x (_ BitVec 32))(declare-const y (_ BitVec 32))
(assert (not (bvuge (bvor x y) x)))(check-sat)
