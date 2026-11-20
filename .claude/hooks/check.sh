#!/usr/bin/env bash
# Stop hook: before Claude ends a turn that changed the working tree, run the
# fast repository checks. On failure, exit 2 so Claude sees the output and
# fixes it. Slow checks (clippy, tests) stay manual; see CLAUDE.md.
set -u

input=$(cat)
# Do not loop: if this stop was already caused by this hook, let it through.
case "$input" in
  *'"stop_hook_active":true'* | *'"stop_hook_active": true'*) exit 0 ;;
esac

cd "${CLAUDE_PROJECT_DIR:-$(dirname "$0")/../..}" || exit 0

# Nothing changed: nothing to check.
[ -z "$(git status --porcelain 2>/dev/null)" ] && exit 0

fail=0
fmt=$(cargo fmt --all --check 2>&1) || { fail=1; fmt="cargo fmt --all --check failed; run \`cargo fmt --all\`.
$(echo "$fmt" | head -40)"; }
lint=$(cargo xtask lint 2>&1) || fail=1

if [ $fail -ne 0 ]; then
  { [ -n "$fmt" ] && echo "$fmt"; echo "$lint"; } >&2
  exit 2
fi
exit 0
