(set-logic ALL)
; Route (a): machine integers as Int with a range; bitwise operators as
(define-const W Int 4294967296)
(declare-fun and32 (Int Int) Int)(declare-fun or32 (Int Int) Int)(declare-fun xor32 (Int Int) Int)
(declare-fun add32 (Int Int) Int)(declare-fun sub32 (Int Int) Int)
(declare-fun shl32 (Int Int) Int)(declare-fun lshr32 (Int Int) Int)
; x06: x ^ x == 0 (64-bit)
(declare-fun xor64 (Int Int) Int)(declare-const x Int)(assert (and (<= 0 x) (< x 18446744073709551616)))
(assert (not (= (xor64 x x) 0)))
; qd_bv_int lemma library for route (a), unsigned 32-bit (and xor-self at 64).
; Each schema is stated for the shift/mask amounts the corpus uses; in the
; kernel, k would be a schema parameter and 2^k a literal from qd_evaluate.
; Inputs are assumed in range [0, W) by the generator's type facts.
(assert (forall ((x Int) (y Int)) (! (=> (and (<= 0 x) (< x W) (<= 0 y) (< y W)) (and (<= 0 (and32 x y)) (< (and32 x y) W) (<= (and32 x y) x) (<= (and32 x y) y))) :pattern ((and32 x y))))) ; and-range, and-le
(assert (forall ((x Int) (y Int)) (! (=> (and (<= 0 x) (< x W) (<= 0 y) (< y W)) (and (< (or32 x y) W) (>= (or32 x y) x) (>= (or32 x y) y))) :pattern ((or32 x y))))) ; or-range, or-ge
(assert (forall ((x Int)) (! (= (xor32 x x) 0) :pattern ((xor32 x x))))) ; xor-self
(assert (forall ((x Int)) (! (= (xor64 x x) 0) :pattern ((xor64 x x))))) ; xor-self (64)
(assert (forall ((x Int) (y Int)) (! (= (add32 x y) (mod (+ x y) W)) :pattern ((add32 x y))))) ; add
(assert (forall ((x Int) (y Int)) (! (= (sub32 x y) (mod (- x y) W)) :pattern ((sub32 x y))))) ; sub
(assert (forall ((x Int)) (! (= (shl32 x 1) (mod (* x 2) W)) :pattern ((shl32 x 1))))) ; shl k=1
(assert (forall ((x Int)) (! (= (shl32 x 8) (mod (* x 256) W)) :pattern ((shl32 x 8))))) ; shl k=8
(assert (forall ((x Int)) (! (= (lshr32 x 4) (div x 16)) :pattern ((lshr32 x 4))))) ; lshr k=4
(assert (forall ((x Int)) (! (= (lshr32 x 16) (div x 65536)) :pattern ((lshr32 x 16))))) ; lshr k=16
(assert (forall ((x Int)) (! (= (lshr32 x 24) (div x 16777216)) :pattern ((lshr32 x 24))))) ; lshr k=24
(assert (forall ((x Int)) (! (= (and32 x 15) (mod x 16)) :pattern ((and32 x 15))))) ; and-mask k=4
(assert (forall ((x Int)) (! (= (and32 x 255) (mod x 256)) :pattern ((and32 x 255))))) ; and-mask k=8
(assert (forall ((x Int)) (! (= (and32 x 1023) (mod x 1024)) :pattern ((and32 x 1023))))) ; and-mask k=10
(assert (forall ((x Int) (y Int)) (! (=> (and (= (mod x 256) 0) (<= 0 y) (< y 256)) (= (or32 x y) (+ x y))) :pattern ((or32 x y))))) ; or-disjoint k=8
(check-sat)
