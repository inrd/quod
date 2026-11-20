(set-logic ALL)
; schema xor_self, 32-bit, as a pure bit-vector formula (expected unsat)
; a ^ a == 0
(declare-const a (_ BitVec 32))(declare-const b (_ BitVec 32))
(assert (not (= (bvxor a a) (_ bv0 32))))
(check-sat)
