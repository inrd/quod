(set-logic ALL)
(declare-const x (_ BitVec 32))
(assert (not (and (<= 0 (ubv_to_int x)) (<= (ubv_to_int x) 4294967295))))
(check-sat)
