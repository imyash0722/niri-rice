#!/bin/bash

# A script to lock the screen using the current wallpaper (extracts first frame if GIF/Video)
LOCK_IMG="/tmp/lockscreen_bg.png"

# Find running wallpaper process and extract file path
WALLPAPER=""

if pgrep -x mpvpaper >/dev/null; then
    # mpvpaper takes the file as the last argument
    WALLPAPER=$(tr '\0' '\n' < /proc/$(pgrep -n mpvpaper)/cmdline | awk 'NF' | tail -n 1)
elif pgrep -x swaybg >/dev/null; then
    # swaybg uses -i <image>
    WALLPAPER=$(tr '\0' '\n' < /proc/$(pgrep -n swaybg)/cmdline | grep -A1 -- "-i" | tail -n 1)
fi

# Apply the wallpaper to swaylock
if [ -n "$WALLPAPER" ] && [ -f "$WALLPAPER" ]; then
    MIME_TYPE=$(file --mime-type -b "$WALLPAPER")
    
    if [[ "$MIME_TYPE" == video/* ]] || [[ "$MIME_TYPE" == image/gif ]]; then
        # It's animated. Extract a single frame for the lockscreen
        ffmpeg -y -i "$WALLPAPER" -vframes 1 -q:v 2 "$LOCK_IMG" -loglevel quiet
        swaylock -i "$LOCK_IMG"
    else
        # It's already a static image
        swaylock -i "$WALLPAPER"
    fi
else
    # Fallback to plain black if no wallpaper is found
    swaylock -c 000000
fi
