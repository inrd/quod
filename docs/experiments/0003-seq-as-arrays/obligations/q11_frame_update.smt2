(set-logic ALL)
; q11: frame, an update at i leaves index j /= i unchanged
(declare-const a (Array Int Int))(declare-const a_len Int)(declare-const i Int)(declare-const j Int)(declare-const v Int)
(assert (<= 0 a_len))
(assert (and (<= 0 i) (< i a_len) (<= 0 j) (< j a_len) (distinct i j)))
(assert (not (= (select (store a i v) j) (select a j))))
(check-sat)
