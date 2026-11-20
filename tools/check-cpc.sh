#!/bin/sh
# Prove one SMT-LIB obligation with the pinned cvc5 and check the CPC proof
# with the pinned Ethos, the way cvc5's own regression tests do.
#
#   tools/check-cpc.sh <file.smt2> [extra cvc5 flags...]
#
# Prints "checked", "not unsat: <verdict>" or Ethos's error. Exit 0 only when
# Ethos accepts a proof of false. Run tools/fetch.sh first.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
BIN=$HERE/bin
[ -x "$BIN/cvc5" ] && [ -x "$BIN/ethos" ] && [ -d "$BIN/cpc" ] \
  || { echo "check-cpc.sh: run tools/fetch.sh first" >&2; exit 2; }
f=$1; shift

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
"$BIN/cvc5" --dump-proofs --proof-format-mode=cpc --proof-print-conclusion "$@" "$f" \
  </dev/null >"$tmp/out" 2>&1 || true

verdict=$(head -1 "$tmp/out")
if [ "$verdict" != unsat ]; then
  echo "not unsat: $verdict"
  exit 1
fi
# Body between the "unsat", "(" header lines and the final ")" line.
{
  printf '(include "%s/Cpc.eo")\n' "$BIN/cpc"
  printf '(include "%s/expert/CpcExpert.eo")\n' "$BIN/cpc"
  sed -e '1,2d' -e '$d' "$tmp/out"
} >"$tmp/proof.eo"

if "$BIN/ethos" --require-proof-of-false "$tmp/proof.eo" >"$tmp/ethos" 2>&1; then
  echo checked
else
  cat "$tmp/ethos"
  exit 1
fi
