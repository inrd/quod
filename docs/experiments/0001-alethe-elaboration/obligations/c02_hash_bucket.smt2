(set-logic ALL)
(declare-const h (_ BitVec 32))
(assert (not (and (<= 0 (ubv_to_int (bvand h #x000003ff))) (< (ubv_to_int (bvand h #x000003ff)) 1024))))
(check-sat)
