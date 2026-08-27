#!/bin/bash
# mt7921e-sleep.sh — MediaTek MT7922 Wi-Fi sleep/hibernate hook
# Install to: /usr/lib/systemd/system-sleep/
# Requires: chmod +x
#
# Fixes MT7922 (mt7921e driver) firmware crash on resume by fully unloading
# the entire driver stack before sleep and reloading it after wake.

LOG="/tmp/mt7921e-sleep.log"
echo "[$(date)] called with: $1 $2" >> "$LOG"

# Helper: find the wireless interface name dynamically
get_iface() {
    ip link show | awk -F': ' '/^[0-9]+: wl/{print $2}' | head -1
}

# Full ordered module stack (leaf → root, for unloading)
MODULES=(mt7921e mt7921_common mt792x_lib mt76_connac_lib mt76)

case "$1" in
    pre)
        IFACE=$(get_iface)
        if [[ -n "$IFACE" ]]; then
            echo "[$(date)] pre: taking $IFACE down" >> "$LOG"
            /usr/bin/ip link set "$IFACE" down 2>>"$LOG"
            sleep 0.3
        else
            echo "[$(date)] pre: no wireless interface found (already down?)" >> "$LOG"
        fi

        echo "[$(date)] pre: unloading driver stack" >> "$LOG"
        for mod in "${MODULES[@]}"; do
            /usr/bin/modprobe -r "$mod" 2>>"$LOG" && \
                echo "[$(date)] pre: unloaded $mod" >> "$LOG" || \
                echo "[$(date)] pre: $mod not loaded (ok)" >> "$LOG"
        done
        ;;

    post)
        echo "[$(date)] post: reloading driver stack" >> "$LOG"
        # Only need to load the top-level module; kernel pulls in deps
        /usr/bin/modprobe mt7921e 2>>"$LOG"

        # Wait for the interface to appear (up to 5s)
        for i in $(seq 1 10); do
            IFACE=$(get_iface)
            [[ -n "$IFACE" ]] && break
            sleep 0.5
        done

        if [[ -z "$IFACE" ]]; then
            echo "[$(date)] post: ERROR — interface never appeared after 5s" >> "$LOG"
            exit 1
        fi

        echo "[$(date)] post: bringing $IFACE up" >> "$LOG"
        /usr/bin/ip link set "$IFACE" up 2>>"$LOG"

        # Ensure power save stays OFF (MT7922 firmware bug)
        sleep 0.5
        /usr/bin/iw dev "$IFACE" set power_save off 2>>"$LOG" && \
            echo "[$(date)] post: power_save=off applied to $IFACE" >> "$LOG"
        ;;
esac
