(set-logic ALL)
; Route (a): machine integers as Int with a range; bitwise operators as
; uninterpreted functions; lines ending in "; inst" are qd_bv_int lemma
; instances (see ../library.smt2 for the schemas).
(define-const W Int 4294967296)
(declare-fun and32 (Int Int) Int)(declare-fun or32 (Int Int) Int)(declare-fun xor32 (Int Int) Int)
(declare-fun add32 (Int Int) Int)(declare-fun sub32 (Int Int) Int)
(declare-fun shl32 (Int Int) Int)(declare-fun lshr32 (Int Int) Int)
; x02: x >> 4 < 0x10000000
(declare-const x Int)(assert (and (<= 0 x) (< x W)))
(assert (= (lshr32 x 4) (div x 16))) ; inst lshr
(assert (not (< (lshr32 x 4) 268435456)))
(check-sat)
