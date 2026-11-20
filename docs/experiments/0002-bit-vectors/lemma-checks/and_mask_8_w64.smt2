(set-logic ALL)
; schema and_mask_8, 64-bit, as a pure bit-vector formula (expected unsat)
; int(a & 0xff) == int(a) mod 256: bvurem is defined as mod on ubv_to_int
(declare-const a (_ BitVec 64))(declare-const b (_ BitVec 64))
(assert (not (= (bvand a (_ bv255 64)) (bvurem a (_ bv256 64)))))
(check-sat)
