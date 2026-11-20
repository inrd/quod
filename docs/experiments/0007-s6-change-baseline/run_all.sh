#!/bin/sh
# Run every task in both verifiers (skips runs that already have a session).
#   run_all.sh [parallelism]
set -eu
cd "$(dirname "$0")"
for f in tasks/dafny/*.dfy tasks/verus/*.rs; do
  lang=$(basename "$(dirname "$f")"); t=$(basename "$f" | cut -d_ -f1)
  [ -s "runs/$lang/$t/session.json" ] || echo "$lang $f runs/$lang/$t"
done | xargs -P "${1:-4}" -L 1 sh -c './run.sh "$0" "$1" "$2"; echo "done $0 $2"'
