#!/bin/sh
# ./verify inside a run directory: checks the task file, logs every call.
# Installed into each run directory by run.sh as ./verify; LANG_ and FILE_
# are substituted there. Dafny 4.11's "Prover error: ... models" lines are
# a cosmetic counterexample-parsing bug with Z3 4.14 and are filtered out.
set -u
cd "$(dirname "$0")"
n=$(($(cat .calls 2>/dev/null || echo 0) + 1)); echo $n >.calls
cp FILE_ ".snap/$n.src"
case LANG_ in
  dafny) perl -e 'alarm shift; exec @ARGV' 300 TOOLS_/dafny verify FILE_ 2>&1 | grep -v '^Prover error' >".snap/$n.out" ;;
  verus) perl -e 'alarm shift; exec @ARGV' 300 TOOLS_/verus FILE_ >".snap/$n.out" 2>&1 ;;
esac
cat ".snap/$n.out"
