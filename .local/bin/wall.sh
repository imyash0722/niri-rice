#!/usr/bin/env bash

# 1. Singleton Toggle: If already running, close and exit (prevents duplicate instances)
if pkill -f "rofi.*wallpaper-select.rasi" 2>/dev/null; then
    exit 0
fi

WALL_DIR="$HOME/Pictures/Wall"
CACHE_DIR="$HOME/.cache/thumbnails/wallpapers"
mkdir -p "$CACHE_DIR"

if [ ! -d "$WALL_DIR" ]; then
    notify-send "Wallpaper Picker" "Folder ~/Pictures/Wall not found"
    exit 1
fi

# 2. Ultra-Fast Instant Entry Streamer & Rofi Launcher (20ms startup)
INDEX=$(python3 -c "
import os, sys

wall_dir = os.path.expanduser('~/Pictures/Wall')
cache_dir = os.path.expanduser('~/.cache/thumbnails/wallpapers')
exts = ('.jpg', '.jpeg', '.png', '.webp', '.gif', '.mp4', '.mkv')

try:
    files = sorted([f for f in os.listdir(wall_dir) if f.lower().endswith(exts)])
    for f in files:
        base = os.path.splitext(f)[0]
        display = base.replace('_', ' ').replace('-', ' ').title()
        thumb = os.path.join(cache_dir, f'{base}_card.png')
        if not os.path.exists(thumb):
            thumb = os.path.join(wall_dir, f)
        sys.stdout.write(f'{display}\x00icon\x1f{thumb}\n')
except Exception:
    pass
" | rofi -dmenu -p "󰸉 Wallpapers" -i -show-icons -format i -theme "$HOME/.config/rofi/wallpaper-select.rasi")

# 3. Exit if user dismissed rofi with Escape or clicked outside
[ -z "$INDEX" ] && exit 0

# 4. Resolve selected file and apply
SELECTED=$(python3 -c "
import os, sys
wall_dir = os.path.expanduser('~/Pictures/Wall')
exts = ('.jpg', '.jpeg', '.png', '.webp', '.gif', '.mp4', '.mkv')
try:
    idx = int('$INDEX')
    files = sorted([f for f in os.listdir(wall_dir) if f.lower().endswith(exts)])
    if 0 <= idx < len(files):
        print(os.path.join(wall_dir, files[idx]))
except Exception:
    pass
")

if [ -n "$SELECTED" ] && [ -f "$SELECTED" ]; then
    sel_name=$(basename "$SELECTED")
    display_sel=$(echo "${sel_name%.*}" | tr '_' ' ' | tr '-' ' ' | sed -E 's/\b([a-z])/\U\1/g')
    notify-send -a "Wallpaper Manager" -i "preferences-desktop-wallpaper" "Applying Wallpaper..." "$display_sel"
    "$HOME/.config/niri/scripts/apply-wallpaper.sh" "$SELECTED"
fi
