#!/bin/bash

# Prevent multiple lockscreen instances from fighting over session lock
if pgrep -f "quickshell.*lock_shell\.qml" >/dev/null 2>&1; then
    exit 0
fi

# A script to lock the screen using qylock (Quickshell lockscreen)
if [ -f "$HOME/.local/share/quickshell-lockscreen/lock.sh" ]; then
    exec "$HOME/.local/share/quickshell-lockscreen/lock.sh" "$@"
elif [ -f "$HOME/.local/share/qylock/quickshell-lockscreen/lock.sh" ]; then
    exec "$HOME/.local/share/qylock/quickshell-lockscreen/lock.sh" "$@"
elif [ -f "$HOME/.local/share/qylock/lock.sh" ]; then
    exec "$HOME/.local/share/qylock/lock.sh" "$@"
else
    echo "Quickshell lockscreen not found. Falling back to loginctl lock-session."
    loginctl lock-session
fi
