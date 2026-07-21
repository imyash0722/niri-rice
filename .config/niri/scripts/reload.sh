#!/bin/bash

# Reload Waybar
pkill -x waybar
nohup waybar -c ~/.config/niri/waybar/config -s ~/.config/niri/waybar/style.css >/dev/null 2>&1 &

# Reload Wallpaper
pkill -x swww-daemon
sleep 1
swww-daemon &
sleep 1
swww img ~/Wallpapers/4000x4000__Hakui_By-pass-ezgif.com-crop.gif
