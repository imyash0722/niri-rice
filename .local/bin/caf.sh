#!/usr/bin/env bash

# File flag for caffeine state
FLAG="/tmp/caffeine_active"

toggle() {
    if [ -f "$FLAG" ]; then
        # Turn OFF Caffeine -> Restore normal sleep and TLP balanced/bat
        rm -f "$FLAG"
        pkill -f "systemd-inhibit.*caffeine" 2>/dev/null || true
        
        # Restore cool / low-power platform profile
        echo "low-power" > /sys/firmware/acpi/platform_profile 2>/dev/null || true
        
        notify-send -a "Power Manager" -i "preferences-system-power" "Sleep Mode: Normal" "Auto-sleep enabled, cool profile active"
    else
        # Turn ON Caffeine -> Disable sleep & idle, set performance
        touch "$FLAG"
        
        # Inhibit systemd idle, sleep, and lid switch
        nohup systemd-inhibit --what=idle:sleep:handle-lid-switch --who="Caffeine" --why="User requested no sleep" sleep infinity >/dev/null 2>&1 &
        
        # Set performance platform profile
        echo "performance" > /sys/firmware/acpi/platform_profile 2>/dev/null || true
        
        notify-send -a "Power Manager" -i "caffeine" "Sleep Mode: Caffeinated" "Sleep & idle disabled (Performance mode)"
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
