(set-logic ALL)
; s02 (new): t = &s[lo2..hi2] with s = &a[lo..hi]; t[k] reads a[lo + lo2 + k]
; and is in bounds. Views compose offsets; no new array term.
(declare-const a (Array Int Int))(declare-const a_len Int)
(declare-const lo Int)(declare-const hi Int)(declare-const lo2 Int)(declare-const hi2 Int)(declare-const k Int)
(assert (<= 0 a_len))
(assert (and (<= 0 lo) (<= lo hi) (<= hi a_len)))
(assert (and (<= 0 lo2) (<= lo2 hi2) (<= hi2 (- hi lo))))
(assert (and (<= 0 k) (< k (- hi2 lo2))))
(assert (not (and (<= 0 (+ (+ lo lo2) k)) (< (+ (+ lo lo2) k) a_len)
                  (= (select a (+ lo (+ lo2 k))) (select a (+ (+ lo lo2) k))))))
(check-sat)
