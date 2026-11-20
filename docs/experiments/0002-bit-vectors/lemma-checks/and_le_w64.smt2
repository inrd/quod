(set-logic ALL)
; schema and_le, 64-bit, as a pure bit-vector formula (expected unsat)
; int(a & b) <= int(a), int(b): bvule is <= on ubv_to_int
(declare-const a (_ BitVec 64))(declare-const b (_ BitVec 64))
(assert (not (and (bvule (bvand a b) a) (bvule (bvand a b) b))))
(check-sat)
