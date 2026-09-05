#!/usr/bin/env bash

# File flag for caffeine state
FLAG="/tmp/caffeine_active"

toggle() {
    if [ -f "$FLAG" ]; then
        # Turn OFF Caffeine -> Restore normal sleep and TLP balanced/bat
        rm -f "$FLAG"
        pkill -f "systemd-inhibit.*caffeine" 2>/dev/null || true
        
        # TLP / power profile to balanced/bat
        powerprofilesctl set balanced 2>/dev/null || true
        if command -v tlp &>/dev/null; then
            tlp-stat -s >/dev/null 2>&1 || true
        fi
        
        notify-send -a "Power Manager" -i "preferences-system-power" "Sleep Mode: Normal" "Automatic sleep & idle enabled (TLP Balanced)"
    else
        # Turn ON Caffeine -> Disable sleep & idle, set TLP performance/ac
        touch "$FLAG"
        
        # Inhibit systemd idle/sleep
        nohup systemd-inhibit --what=idle:sleep --who="Caffeine" --why="User requested no sleep" sleep infinity >/dev/null 2>&1 &
        
        # TLP / power profile to performance
        powerprofilesctl set performance 2>/dev/null || true
        
        notify-send -a "Power Manager" -i "caffeine" "Sleep Mode: Caffeinated" "Sleep & idle disabled (TLP Performance / AC)"
    fi
    pkill -RTMIN+13 waybar 2>/dev/null || true
}

status() {
    if [ -f "$FLAG" ]; then
        echo '{"text": " caffeinated", "class": "active", "tooltip": "Caffeine Active: Sleep Disabled (TLP AC/Perf)"}'
    else
        echo '{"text": "󰒲 sleep on", "class": "inactive", "tooltip": "Sleep Enabled: Auto-suspend active (TLP Balanced)"}'
    fi
}

case "$1" in
    toggle)
        toggle
        ;;
    status|*)
        status
        ;;
esac
