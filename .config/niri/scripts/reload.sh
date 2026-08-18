#!/bin/bash

# Reload Waybar
pkill -x waybar
nohup waybar >/dev/null 2>&1 &

# Reload Wallpaper
awww img ~/Wallpapers/silent-katana-forest-samurai-live-wallpaper-wallsflow-com.gif
