#!/bin/bash

# Only run if we are in Niri
if [ "$XDG_CURRENT_DESKTOP" != "niri" ] && [ -z "$NIRI_SOCKET" ]; then
    echo "Not running in Niri. Aborting reload."
    exit 1
fi

# 1. Hot-reload Waybar CSS or launch if not running
if pgrep -x waybar >/dev/null; then
    killall -SIGUSR2 waybar 2>/dev/null || true
else
    nohup waybar >/dev/null 2>&1 &
fi

# 2. Reload Wallpaper cleanly (via awww)
WALL_PATH="$(cat "$HOME/.config/niri/themes/active-wallpaper.txt" 2>/dev/null)"
if [ -n "$WALL_PATH" ] && [ -f "$WALL_PATH" ]; then
    pgrep -x awww-daemon >/dev/null || nohup awww-daemon >/dev/null 2>&1 &
    awww img "$WALL_PATH" \
        --transition-type center \
        --transition-pos center \
        --transition-duration 1.2 \
        --transition-fps 120 \
        --transition-bezier .25,1,.5,1 2>/dev/null || true
fi

# 3. Reload Mako notifications & Foot
makoctl reload 2>/dev/null || true

# 4. Trigger smooth screen transition in Niri
niri msg action do-screen-transition 2>/dev/null || true

