#!/usr/bin/env bash

PROFILES=(
    "󰌪  Power Saver (TLP Battery)"
    "  Balanced (TLP Balanced)"
    "󱓞  Performance (TLP AC Mode)"
)

CHOICE=$(
    printf '%s\n' "${PROFILES[@]}" | rofi \
        -dmenu \
        -i \
        -p "Power Profile" \
        -theme-str '
        window { width: 250px; }
        listview { lines: 3; }
        '
)

case "$CHOICE" in
*"Performance"*)
    powerprofilesctl set performance 2>/dev/null || true
    if [ -f /home/pineapple/.askpass.sh ]; then
        SUDO_ASKPASS=/home/pineapple/.askpass.sh sudo -A tlp ac 2>/dev/null || true
    fi
    notify-send -a "Power Profile" -i "battery-charging" "Performance Mode" "TLP set to AC / max performance"
    ;;
*"Balanced"*)
    powerprofilesctl set balanced 2>/dev/null || true
    if [ -f /home/pineapple/.askpass.sh ]; then
        SUDO_ASKPASS=/home/pineapple/.askpass.sh sudo -A tlp start 2>/dev/null || true
    fi
    notify-send -a "Power Profile" -i "battery" "Balanced Mode" "TLP set to automatic balanced mode"
    ;;
*"Power Saver"*)
    powerprofilesctl set power-saver 2>/dev/null || true
    if [ -f /home/pineapple/.askpass.sh ]; then
        SUDO_ASKPASS=/home/pineapple/.askpass.sh sudo -A tlp bat 2>/dev/null || true
    fi
    notify-send -a "Power Profile" -i "battery-caution" "Power Saver Mode" "TLP set to battery conservation"
    ;;
esac
pkill -sigrtmin+21 waybar 2>/dev/null || true
