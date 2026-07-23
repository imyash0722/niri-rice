#!/bin/bash

# Reload Waybar
pkill -x waybar
nohup waybar >/dev/null 2>&1 &

# Reload Wallpaper
awww img ~/Wallpapers/4000x4000__Hakui_By-pass-ezgif.com-crop.gif
