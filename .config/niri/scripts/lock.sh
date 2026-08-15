#!/bin/bash

# A script to lock the screen using quickshell
if [ -f "$HOME/.local/share/quickshell-lockscreen/lock.sh" ]; then
    "$HOME/.local/share/quickshell-lockscreen/lock.sh"
else
    echo "Quickshell lockscreen not found. Please install qylock."
fi
