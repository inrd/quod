(set-logic ALL)
; s01 (new): s = &a[lo..hi]; s[k] for k < s.len is in bounds of a
(declare-const a_len Int)(declare-const lo Int)(declare-const hi Int)(declare-const k Int)
(assert (<= 0 a_len))
(assert (and (<= 0 lo) (<= lo hi) (<= hi a_len)))
(assert (and (<= 0 k) (< k (- hi lo))))
(assert (not (and (<= 0 (+ lo k)) (< (+ lo k) a_len))))
(check-sat)
