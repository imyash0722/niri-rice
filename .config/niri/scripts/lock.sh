#!/bin/bash

# A script to lock the screen using qylock
if [ -f "$HOME/.local/share/qylock/lock.sh" ]; then
    "$HOME/.local/share/qylock/lock.sh"
else
    echo "Quickshell lockscreen not found. Please install qylock."
fi
