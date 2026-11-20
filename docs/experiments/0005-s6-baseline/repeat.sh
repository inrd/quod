#!/bin/sh
# Re-verify each final solution N times (default 5); prints lang task passes/N.
# Added after the runs (not pre-registered): Dafny t06 gave different verdicts
# on an unchanged file.
set -eu
cd "$(dirname "$0")"
T=$(cd ../../../tools/bin && pwd); N=${1:-5}
for f in runs/*/*/final.dfy runs/*/*/final.rs; do
  lang=$(echo "$f" | cut -d/ -f2); task=$(echo "$f" | cut -d/ -f3)
  d=$(mktemp -d); cp "$f" "$d/x.${f##*.}"; ok=0; i=0
  while [ $i -lt "$N" ]; do i=$((i+1))
    if [ "$lang" = dafny ]; then out=$(cd "$d" && "$T/dafny" verify x.dfy 2>&1 || true)
      echo "$out" | grep -q "verified, 0 errors" && ! echo "$out" | grep -q "Error:" && ok=$((ok+1))
    else out=$(cd "$d" && "$T/verus" x.rs 2>&1 || true)
      echo "$out" | grep -q "verified, 0 errors" && ! echo "$out" | grep -q "^error" && ok=$((ok+1))
    fi
  done
  rm -rf "$d"; printf '%s\t%s\t%s/%s\n' "$lang" "$task" "$ok" "$N"
done
