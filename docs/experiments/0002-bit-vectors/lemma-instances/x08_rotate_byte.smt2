(set-logic ALL)
; Route (a): machine integers as Int with a range; bitwise operators as
; uninterpreted functions; lines ending in "; inst" are qd_bv_int lemma
; instances (see ../library.smt2 for the schemas).
(define-const W Int 4294967296)
(declare-fun and32 (Int Int) Int)(declare-fun or32 (Int Int) Int)(declare-fun xor32 (Int Int) Int)
(declare-fun add32 (Int Int) Int)(declare-fun sub32 (Int Int) Int)
(declare-fun shl32 (Int Int) Int)(declare-fun lshr32 (Int Int) Int)
; x08: rotl(x, 8) >> 24 == (x >> 16) & 0xff
(declare-const x Int)(assert (and (<= 0 x) (< x W)))
(assert (= (shl32 x 8) (mod (* x 256) W))) ; inst shl
(assert (= (lshr32 x 24) (div x 16777216))) ; inst lshr
(assert (=> (and (= (mod (shl32 x 8) 256) 0) (< (lshr32 x 24) 256)) (= (or32 (shl32 x 8) (lshr32 x 24)) (+ (shl32 x 8) (lshr32 x 24))))) ; inst or-disjoint
(assert (= (lshr32 (or32 (shl32 x 8) (lshr32 x 24)) 24) (div (or32 (shl32 x 8) (lshr32 x 24)) 16777216))) ; inst lshr
(assert (= (lshr32 x 16) (div x 65536))) ; inst lshr
(assert (= (and32 (lshr32 x 16) 255) (mod (lshr32 x 16) 256))) ; inst and-mask
(assert (not (= (lshr32 (or32 (shl32 x 8) (lshr32 x 24)) 24) (and32 (lshr32 x 16) 255))))
(check-sat)
