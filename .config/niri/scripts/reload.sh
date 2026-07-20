#!/bin/bash

# Reload Waybar
pkill -x waybar
nohup waybar -c ~/.config/niri/waybar/config -s ~/.config/niri/waybar/style.css >/dev/null 2>&1 &

# Reload Wallpaper
pkill mpvpaper
nohup mpvpaper -o 'no-audio loop panscan=1.0' '*' /mnt/shared/PROJECTS/niri-rice/Wallpapers/4000x4000__Hakui_By-pass-ezgif.com-crop.gif >/dev/null 2>&1 &
