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
mpvpaper -o 'no-audio loop hwdec=auto scale=lanczos cscale=lanczos dscale=lanczos correct-downscaling=yes linear-downscaling=yes sigmoid-upscaling=yes' '*' ~/Pictures/Wall/silent-katana-forest-samurai-live-wallpaper-wallsflow-com.mp4 &
