#!/bin/bash

# Reload Waybar
pkill -x waybar
nohup waybar >/dev/null 2>&1 &

# Reload Wallpaper (mpvpaper - GPU hardware decoded)
pkill mpvpaper || true
sleep 0.5
mpvpaper -o 'no-audio loop hwdec=auto panscan=1.0' '*' ~/Wallpapers/silent-katana-forest-samurai-live-wallpaper-wallsflow-com.mp4 &
