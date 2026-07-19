#!/bin/bash
# power.sh — niri version
# Replaces swaymsg exit with niri msg action quit
options="Lock\nLogout\nSuspend\nReboot\nShutdown"
chosen=$(echo -e "$options" | rofi -dmenu -i -p "Power" -lines 5)

case "$chosen" in
    Lock)     /home/pineapple/.config/niri/scripts/lock.sh ;;
    Logout)   niri msg action quit ;;
    Suspend)  systemctl suspend ;;
    Reboot)   systemctl reboot ;;
    Shutdown) systemctl poweroff ;;
esac
