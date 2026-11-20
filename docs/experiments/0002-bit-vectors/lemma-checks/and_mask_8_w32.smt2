(set-logic ALL)
; schema and_mask_8, 32-bit, as a pure bit-vector formula (expected unsat)
; int(a & 0xff) == int(a) mod 256: bvurem is defined as mod on ubv_to_int
(declare-const a (_ BitVec 32))(declare-const b (_ BitVec 32))
(assert (not (= (bvand a (_ bv255 32)) (bvurem a (_ bv256 32)))))
(check-sat)
