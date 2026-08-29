#!/bin/bash

# Only run if we are in Niri
if [ "$XDG_CURRENT_DESKTOP" != "niri" ] && [ -z "$NIRI_SOCKET" ]; then
    echo "Not running in Niri. Aborting reload."
    exit 1
fi

# Reload Waybar
pkill -x waybar
nohup waybar >/dev/null 2>&1 &

# Reload Wallpaper (mpvpaper - GPU hardware decoded)
pkill mpvpaper || true
sleep 0.5
mpvpaper -o 'no-audio loop-file=inf hwdec=auto video-unscaled=yes' '*' ~/Pictures/Wall/samurai-1200p-optimized.mp4 &
