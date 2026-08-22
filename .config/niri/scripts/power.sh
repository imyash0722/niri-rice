#!/bin/bash
# power.sh — Rofi power menu for niri
options="Lock\nLogout\nSuspend\nReboot\nShutdown"
chosen=$(echo -e "$options" | rofi -dmenu -i -p "Power" -lines 5)

case "$chosen" in
    Lock)     "$HOME/.config/niri/scripts/lock.sh" ;;
    Logout)   niri msg action quit ;;
    Suspend)  systemctl suspend ;;
    Reboot)   systemctl reboot ;;
    Shutdown) systemctl poweroff ;;
esac
