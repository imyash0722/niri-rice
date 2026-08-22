#!/usr/bin/env bash
# monitor-setup.sh
# Automatically applies per-output scaling based on native resolution.
# Called on startup and on every monitor hotplug event.
#
# Scale rules:
#   4K  (height >= 2160):  1.5
#   2K  (height >= 1440):  1.25
#   1080p and below:       1.0

set -euo pipefail

RUNTIME="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"

# Wait for Niri socket (includes PID in name: niri.wayland-1.<pid>.sock)
for i in $(seq 1 20); do
    NIRI_SOCK="$(ls "$RUNTIME"/niri.wayland-1.*.sock 2>/dev/null | head -1)"
    [[ -n "$NIRI_SOCK" ]] && break
    sleep 0.5
done

if [[ -z "${NIRI_SOCK:-}" ]]; then
    echo "monitor-setup: Niri socket not found, giving up." >&2
    exit 1
fi

# Dump outputs to temp file (avoids stdin conflict with embedded Python)
TMPFILE="$(mktemp /tmp/niri-outputs.XXXXXX.json)"
trap 'rm -f "$TMPFILE"' EXIT

niri msg -j outputs > "$TMPFILE"

python3 - "$TMPFILE" << 'PYEOF'
import json, subprocess, sys

with open(sys.argv[1]) as f:
    data = json.load(f)

# Niri returns a dict keyed by output name
outputs = list(data.values()) if isinstance(data, dict) else data

for output in outputs:
    name = output["name"]
    mode_idx = output.get("current_mode")

    if mode_idx is None:
        print(f"  {name}: off, skipping")
        continue

    mode = output["modes"][mode_idx]
    width, height = mode["width"], mode["height"]

    if height >= 2160:
        scale = "1.5"
        tier  = "4K"
    elif height >= 1440:
        scale = "1.25"
        tier  = "2K"
    else:
        scale = "1.0"
        tier  = "1080p"

    print(f"  {name}: {width}x{height} ({tier}) → scale {scale}")
    result = subprocess.run(
        ["niri", "msg", "output", name, "scale", scale],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print(f"    ERROR: {result.stderr.strip()}", file=sys.stderr)
PYEOF
