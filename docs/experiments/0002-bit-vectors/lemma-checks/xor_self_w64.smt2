(set-logic ALL)
; schema xor_self, 64-bit, as a pure bit-vector formula (expected unsat)
; a ^ a == 0
(declare-const a (_ BitVec 64))(declare-const b (_ BitVec 64))
(assert (not (= (bvxor a a) (_ bv0 64))))
(check-sat)
