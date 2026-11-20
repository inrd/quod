(set-logic ALL)
; schema or_disjoint_8, 64-bit, as a pure bit-vector formula (expected unsat)
; int(a) mod 256 == 0 and int(b) < 256 => int(a | b) == int(a) + int(b): no carry, so bvadd is exact
(declare-const a (_ BitVec 64))(declare-const b (_ BitVec 64))
(assert (not (=> (and (= (bvurem a (_ bv256 64)) (_ bv0 64)) (bvult b (_ bv256 64))) (and (= (bvor a b) (bvadd a b)) (bvuge (bvadd a b) a)))))
(check-sat)
