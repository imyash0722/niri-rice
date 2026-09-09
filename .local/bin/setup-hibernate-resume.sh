#!/usr/bin/env bash
set -euo pipefail
SWAPFILE="/swap/swapfile"
[ -f "$SWAPFILE" ] || exit 0

DEV=$(df --output=source "$SWAPFILE" | tail -n1)
MAJMIN=$(printf "%d:%d\n" "0x$(stat -c "%t" "$DEV")" "0x$(stat -c "%T" "$DEV")")
OFFSET=$(btrfs inspect-internal map-swapfile -r "$SWAPFILE" 2>/dev/null || true)

if [ -n "$MAJMIN" ] && [ -n "$OFFSET" ]; then
    echo "$MAJMIN" > /sys/power/resume
    echo "$OFFSET" > /sys/power/resume_offset
fi
