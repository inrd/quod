(set-logic ALL)
; schema lshr_24, 32-bit, as a pure bit-vector formula (expected unsat)
; int(a >> 24) == int(a) div 2^24: bvudiv is defined as div on ubv_to_int
(declare-const a (_ BitVec 32))(declare-const b (_ BitVec 32))
(assert (not (= (bvlshr a (_ bv24 32)) (bvudiv a (_ bv16777216 32)))))
(check-sat)
