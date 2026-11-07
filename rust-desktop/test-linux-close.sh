#!/usr/bin/env bash
set -euo pipefail
openbox >/tmp/drawio-openbox.log 2>&1 &
wm_pid=$!
"${1:?Pass the Drawio executable}" >/tmp/drawio-close.log 2>&1 &
app_pid=$!
trap 'kill "$app_pid" "$wm_pid" 2>/dev/null || true; cat /tmp/drawio-close.log' EXIT
window=$(xdotool search --sync --onlyvisible --name '^Drawio$' | head -n 1)
wmctrl -ic "$window"
dialog=$(xdotool search --sync --onlyvisible --name '^Close Drawio\?$' | head -n 1)
xdotool windowactivate --sync "$dialog" key Escape
sleep 1
kill -0 "$app_pid" # Cancel must leave the editor running.
if xdotool search --onlyvisible --name '^Close Drawio\?$'; then
  echo 'Cancel did not dismiss the confirmation' >&2
  exit 1
fi
wmctrl -ic "$window"
dialog=$(xdotool search --sync --onlyvisible --name '^Close Drawio\?$' | head -n 1)
xdotool windowactivate --sync "$dialog" key Return
wait "$app_pid" # OK must close the last window and exit successfully.
echo 'Linux Cancel -> Close -> process exit passed'
