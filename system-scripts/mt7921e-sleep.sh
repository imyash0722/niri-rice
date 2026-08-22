#!/bin/bash
# mt7921e-sleep.sh — MediaTek Wi-Fi sleep/hibernate hook
# Install to: /usr/lib/systemd/system-sleep/
# chmod +x required

LOG="/tmp/mt7921e-sleep.log"
echo "[$(date)] $0 called with: $1 $2" >> "$LOG"

# Detect the wireless interface dynamically (handles wlan0, wlp3s0, etc.)
IFACE=$(ip link show | awk -F': ' '/^[0-9]+: wl/{print $2}' | head -1)

if [[ -z "$IFACE" ]]; then
    echo "[$(date)] No wireless interface found, skipping." >> "$LOG"
    exit 0
fi

case "$1" in
    pre)
        echo "[$(date)] pre-sleep: bringing $IFACE down" >> "$LOG"
        /usr/bin/ip link set "$IFACE" down 2>>"$LOG"
        sleep 0.5
        echo "[$(date)] pre-sleep: unloading mt7921e driver" >> "$LOG"
        /usr/bin/modprobe -r mt7921e 2>>"$LOG"
        /usr/bin/modprobe -r mt7921 2>>"$LOG" || true
        /usr/bin/modprobe -r mt76_connac_lib 2>>"$LOG" || true
        ;;
    post)
        echo "[$(date)] post-wake: reloading mt7921e driver" >> "$LOG"
        /usr/bin/modprobe mt7921e 2>>"$LOG"
        sleep 1
        echo "[$(date)] post-wake: bringing $IFACE up" >> "$LOG"
        /usr/bin/ip link set "$IFACE" up 2>>"$LOG"
        ;;
esac
