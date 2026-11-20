#!/bin/sh
# Usage: classify.sh <out-dir>   (after run.sh)
# Classifies each Alethe proof by the worst rule class it uses, relative to
# ADR-0003's accepted rules:
#   A  accepted as listed in ADR-0003 (incl. Tseitin clause rules, subproof)
#   B  premise forms of accepted equality/conjunction rules (cong, trans, symm,
#      and_intro): mechanical elaboration into A
#   C  local, search-free but not accepted: named RARE rewrites, evaluate,
#      polynomial normalization, *_simplify, la_mult_* nonlinear lemmas
#   D  rejected by ADR-0003: hole, lia_generic, all_simplify, bit-blasting
#   -  no Alethe certificate (solver failed or printer unsupported)
set -eu
OUT=$1
A=" resolution th_resolution contraction reordering refl eq_reflexive eq_symmetric eq_transitive eq_congruent eq_congruent_pred la_generic la_disequality la_totality la_tautology forall_inst sko_forall sko_ex bind subproof true false not_not and and_pos and_neg or or_pos or_neg not_and not_or implies implies_pos implies_neg1 implies_neg2 not_implies1 not_implies2 equiv1 equiv2 equiv_pos1 equiv_pos2 equiv_neg1 equiv_neg2 not_equiv1 not_equiv2 xor1 xor2 not_xor1 not_xor2 xor_pos1 xor_pos2 xor_neg1 xor_neg2 ite1 ite2 not_ite1 not_ite2 ite_pos1 ite_pos2 ite_neg1 ite_neg2 "
B=" cong trans symm not_symm and_intro "
D=" hole lia_generic all_simplify trust "
for r in "$OUT"/*.rules; do
  n=$(basename "$r" .rules)
  if [ ! -s "$r" ]; then printf '%s\t-\n' "$n"; continue; fi
  awk -v A="$A" -v B="$B" -v D="$D" -v n="$n" '
    { rule=$2; k="C"
      if (index(A, " " rule " ")) k="A"
      else if (index(B, " " rule " ")) k="B"
      else if (index(D, " " rule " ") || rule ~ /^bv_bitblast/ || rule=="bv_bitwise_slicing") k="D"
      cnt[k]+=$1; tot+=$1
      if (k=="C" || k=="D") { name=(rule=="rare_rewrite"||rule=="hole") ? rule ":" $3 : rule; seen[k " " name]=1 } }
    END { w = cnt["D"] ? "D" : cnt["C"] ? "C" : cnt["B"] ? "B" : "A"
      list=""; for (s in seen) list = list " " s
      printf "%s\t%s\t%d\t%d\t%d\t%d\t%d\t%s\n", n, w, tot, cnt["A"], cnt["B"], cnt["C"], cnt["D"], list }' "$r"
done
