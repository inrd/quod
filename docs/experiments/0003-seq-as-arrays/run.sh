#!/bin/sh
# Usage: run.sh <cvc5> <out-dir> [extra cvc5 flags...]
# Runs every obligation with Alethe proof output. Writes per obligation the raw
# output (<name>.proof) and a rule histogram (<name>.rules: "count rule [name]",
# where name is the RARE rewrite for rare_rewrite and the reason for hole).
# Prints one summary line per obligation: name, verdict, proof steps, note.
set -eu
CVC5=$1; OUT=$2; shift 2; HERE=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$OUT"
FLAGS="--tlimit=30000 --dump-proofs --proof-format-mode=alethe --proof-granularity=dsl-rewrite"
for f in "$HERE"/obligations/*.smt2; do
  n=$(basename "$f" .smt2)
  "$CVC5" $FLAGS "$@" "$f" </dev/null >"$OUT/$n.proof" 2>&1 || true
  verdict=$(head -1 "$OUT/$n.proof" | awk '{print $1}')
  case $verdict in unsat|sat|unknown) ;; *) verdict=timeout ;; esac
  grep -oE ':rule [A-Za-z0-9_-]+( :premises \([^)]*\))?( :args \("[^"]*")?' "$OUT/$n.proof" \
    | sed -E -e 's/^:rule //' -e 's/ :premises \([^)]*\)//' -e 's/ :args \("/ /' -e 's/"$//' \
    | awk '$1=="rare_rewrite"||$1=="hole"{print;next}{print $1}' \
    | sort | uniq -c | sort -rn >"$OUT/$n.rules" || true
  steps=$(grep -cE '^\((step|anchor|assume)' "$OUT/$n.proof" || true)
  note=$(grep -oE '\(error "[^"]*' "$OUT/$n.proof" | sed 's/(error "//' | head -1 || true)
  printf '%s\t%s\t%s\t%s\n' "$n" "$verdict" "$steps" "$note"
done
