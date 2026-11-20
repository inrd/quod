(set-logic ALL)
; Route (a): machine integers as Int with a range; bitwise operators as
; uninterpreted functions; lines ending in "; inst" are qd_bv_int lemma
; instances (see ../library.smt2 for the schemas).
(define-const W Int 4294967296)
(declare-fun and32 (Int Int) Int)(declare-fun or32 (Int Int) Int)(declare-fun xor32 (Int Int) Int)
(declare-fun add32 (Int Int) Int)(declare-fun sub32 (Int Int) Int)
(declare-fun shl32 (Int Int) Int)(declare-fun lshr32 (Int Int) Int)
; x03: x != 0 => x & (x - 1) < x  (code-side x - 1 is sub_wrap)
(declare-const x Int)(assert (and (<= 0 x) (< x W)))
(assert (distinct x 0))
(assert (= (sub32 x 1) (mod (- x 1) W))) ; inst sub
(assert (and (<= (and32 x (sub32 x 1)) x) (<= (and32 x (sub32 x 1)) (sub32 x 1)))) ; inst and-le
(assert (not (< (and32 x (sub32 x 1)) x)))
(check-sat)
