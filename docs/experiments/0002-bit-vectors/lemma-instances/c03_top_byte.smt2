(set-logic ALL)
; Route (a): machine integers as Int with a range; bitwise operators as
; uninterpreted functions; lines ending in "; inst" are qd_bv_int lemma
; instances (see ../library.smt2 for the schemas).
(define-const W Int 4294967296)
(declare-fun and32 (Int Int) Int)(declare-fun or32 (Int Int) Int)(declare-fun xor32 (Int Int) Int)
(declare-fun add32 (Int Int) Int)(declare-fun sub32 (Int Int) Int)
(declare-fun shl32 (Int Int) Int)(declare-fun lshr32 (Int Int) Int)
; c03: int(x >> 24) < 256
(declare-const x Int)(assert (and (<= 0 x) (< x W)))
(assert (= (lshr32 x 24) (div x 16777216))) ; inst lshr
(assert (not (< (lshr32 x 24) 256)))
(check-sat)
