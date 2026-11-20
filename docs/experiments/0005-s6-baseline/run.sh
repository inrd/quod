#!/bin/sh
# Experiment 0005: one agent run on one baseline task.
#   run.sh <dafny|verus> <task-file> <out-dir>
# Copies the task into a fresh directory outside the repository, gives a
# headless Claude Code session only Read/Edit/Write and ./verify, and stores
# the session JSON, every ./verify snapshot and the final file in <out-dir>.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
TOOLS=$(cd "$HERE/../../../tools/bin" && pwd)
: "${MODEL:=claude-sonnet-5-5}" "${BUDGET:=5}"
lang=$1; task=$2; out=$3
file=$(basename "$task")
work=$(mktemp -d "${TMPDIR:-/tmp}/qd0005.XXXXXX")
mkdir -p "$work/.snap" "$out"
out=$(cd "$out" && pwd)
cp "$task" "$work/$file"
sed "s|LANG_|$lang|g; s|FILE_|$file|g; s|TOOLS_|$TOOLS|g" "$HERE/verify.sh" >"$work/verify"
chmod 755 "$work/verify"
case $lang in dafny) L=Dafny ;; verus) L=Verus ;; esac
prompt=$(sed "s|LANG_|$L|g; s|FILE_|$file|g" "$HERE/prompt.txt")
start=$(date +%s)
( cd "$work" && claude -p "$prompt" --model "$MODEL" --safe-mode --strict-mcp-config \
    --no-session-persistence --output-format json --max-budget-usd "$BUDGET" \
    --tools "Bash,Read,Edit,Write" --allowedTools "Bash(./verify)" Read Edit Write \
    --permission-mode dontAsk >"$out/session.json" 2>"$out/session.err" ) || true
echo $(( $(date +%s) - start )) >"$out/wall_seconds"
# Final check, outside the agent's control; not counted as a call.
case $lang in
  dafny) ( cd "$work" && "$TOOLS/dafny" verify "$file" 2>&1 | grep -v "^Prover error" ) >"$out/final.out" || true ;;
  verus) ( cd "$work" && "$TOOLS/verus" "$file" ) >"$out/final.out" 2>&1 || true ;;
esac
cp "$work/$file" "$out/final.${file##*.}"
cp -R "$work/.snap" "$out/snap"
echo "$MODEL" >"$out/model"
rm -rf "$work"
