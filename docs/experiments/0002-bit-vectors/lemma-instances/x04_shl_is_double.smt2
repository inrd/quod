(set-logic ALL)
; Route (a): machine integers as Int with a range; bitwise operators as
; uninterpreted functions; lines ending in "; inst" are qd_bv_int lemma
; instances (see ../library.smt2 for the schemas).
(define-const W Int 4294967296)
(declare-fun and32 (Int Int) Int)(declare-fun or32 (Int Int) Int)(declare-fun xor32 (Int Int) Int)
(declare-fun add32 (Int Int) Int)(declare-fun sub32 (Int Int) Int)
(declare-fun shl32 (Int Int) Int)(declare-fun lshr32 (Int Int) Int)
; x04: x << 1 == x + x  (wrapping, as bit-vectors)
(declare-const x Int)(assert (and (<= 0 x) (< x W)))
(assert (= (shl32 x 1) (mod (* x 2) W))) ; inst shl
(assert (= (add32 x x) (mod (+ x x) W))) ; inst add
(assert (not (= (shl32 x 1) (add32 x x))))
(check-sat)
