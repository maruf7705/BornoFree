#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ctl="$root/linux/bin/borno-fcitxctl"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
state="$tmp/current-im"
printf 'keyboard-us' > "$state"

cat > "$tmp/fcitx5-remote" <<'REMOTE'
#!/usr/bin/env bash
set -euo pipefail
state="${FCITX_TEST_STATE:?}"
case "${1:-}" in
  -n) cat "$state" ;;
  -s) printf '%s' "$2" > "$state" ;;
  *) exit 64 ;;
esac
REMOTE
chmod +x "$tmp/fcitx5-remote"

export FCITX5_REMOTE="$tmp/fcitx5-remote"
export FCITX_TEST_STATE="$state"
[[ "$("$ctl" status)" == "EN" ]]
[[ "$("$ctl" toggle)" == "অ" ]]
[[ "$(cat "$state")" == "borno" ]]
[[ "$("$ctl" bangla)" == "অ" ]]
[[ "$("$ctl" english)" == "EN" ]]
[[ "$(cat "$state")" == "keyboard-us" ]]
printf 'FCITXCTL_TEST=PASS\n'