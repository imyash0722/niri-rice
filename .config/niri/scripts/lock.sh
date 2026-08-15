#!/bin/bash

# A script to lock the screen using hyprlock or quickshell

if [ -f "$HOME/.local/share/quickshell-lockscreen/lock.sh" ]; then
    # Use qylock quickshell lockscreen if installed
    "$HOME/.local/share/quickshell-lockscreen/lock.sh"
else
    # Fallback to hyprlock
    hyprlock
fi
