(set-logic ALL)
; Route (a): machine integers as Int with a range; bitwise operators as
; uninterpreted functions; lines ending in "; inst" are qd_bv_int lemma
; instances (see ../library.smt2 for the schemas).
(define-const W Int 4294967296)
(declare-fun and32 (Int Int) Int)(declare-fun or32 (Int Int) Int)(declare-fun xor32 (Int Int) Int)
(declare-fun add32 (Int Int) Int)(declare-fun sub32 (Int Int) Int)
(declare-fun shl32 (Int Int) Int)(declare-fun lshr32 (Int Int) Int)
; c04: int(x) + int(y) <= MAX => int(add_wrap(x, y)) == int(x) + int(y)
(declare-const x Int)(declare-const y Int)(assert (and (<= 0 x) (< x W) (<= 0 y) (< y W)))
(assert (<= (+ x y) 4294967295))
(assert (= (add32 x y) (mod (+ x y) W))) ; inst add
(assert (not (= (add32 x y) (+ x y))))
(check-sat)
