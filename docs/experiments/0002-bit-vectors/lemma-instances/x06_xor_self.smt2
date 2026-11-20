(set-logic ALL)
; Route (a): machine integers as Int with a range; bitwise operators as
; uninterpreted functions; lines ending in "; inst" are qd_bv_int lemma
; instances (see ../library.smt2 for the schemas).
(define-const W Int 4294967296)
(declare-fun and32 (Int Int) Int)(declare-fun or32 (Int Int) Int)(declare-fun xor32 (Int Int) Int)
(declare-fun add32 (Int Int) Int)(declare-fun sub32 (Int Int) Int)
(declare-fun shl32 (Int Int) Int)(declare-fun lshr32 (Int Int) Int)
; x06: x ^ x == 0 (64-bit)
(declare-fun xor64 (Int Int) Int)(declare-const x Int)(assert (and (<= 0 x) (< x 18446744073709551616)))
(assert (= (xor64 x x) 0)) ; inst xor-self
(assert (not (= (xor64 x x) 0)))
(check-sat)
