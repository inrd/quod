(set-logic ALL)
(declare-const x (_ BitVec 32))
(assert (not (<= (ubv_to_int (bvand x #x000000ff)) 255)))
(check-sat)
