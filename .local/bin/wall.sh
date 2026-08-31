#!/usr/bin/env bash

# 1. Singleton Toggle: If already running, close and exit (prevents duplicate instances)
if pkill -f "rofi.*wallpaper-select.rasi" 2>/dev/null; then
    exit 0
fi

WALL_DIR="$HOME/Pictures/Wall"
THUMB_DIR="$WALL_DIR/.thumbnails"

if [ ! -d "$WALL_DIR" ]; then
    notify-send "Wallpaper Picker" "Folder ~/Pictures/Wall not found"
    exit 1
fi

mkdir -p "$THUMB_DIR"

# 2. Instant Precomputed Thumbnail & Entry Streamer (<10ms startup)
INDEX=$(python3 -c "
import os, sys

wall_dir = os.path.expanduser('~/Pictures/Wall')
thumb_dir = os.path.join(wall_dir, '.thumbnails')

if not os.path.exists(thumb_dir):
    os.makedirs(thumb_dir, exist_ok=True)

# Discover unique wallpaper concepts from .thumbnails or wallpapers
thumbs = sorted([f for f in os.listdir(thumb_dir) if f.endswith('.png')])
if not thumbs:
    # Fallback to files in wall_dir
    files = sorted([f for f in os.listdir(wall_dir) if not f.startswith('.') and not os.path.isdir(os.path.join(wall_dir, f))])
    concepts = sorted(list(set([f.rsplit('_16x', 1)[0] for f in files])))
else:
    concepts = [os.path.splitext(f)[0] for f in thumbs]

for c in concepts:
    display = c.replace('_', ' ').replace('-', ' ').title()
    thumb = os.path.join(thumb_dir, f'{c}.png')
    sys.stdout.write(f'{display}\x00icon\x1f{thumb}\n')
" | rofi -dmenu -p "󰸉 Wallpapers" -i -show-icons -format i -theme "$HOME/.config/rofi/wallpaper-select.rasi")

# 3. Exit if user dismissed rofi with Escape or clicked outside
[ -z "$INDEX" ] && exit 0

# 4. Resolve selected concept and pick 16:10 or 16:9 file
SELECTED=$(python3 -c "
import os, sys

wall_dir = os.path.expanduser('~/Pictures/Wall')
thumb_dir = os.path.join(wall_dir, '.thumbnails')

thumbs = sorted([f for f in os.listdir(thumb_dir) if f.endswith('.png')])
if not thumbs:
    files = sorted([f for f in os.listdir(wall_dir) if not f.startswith('.') and not os.path.isdir(os.path.join(wall_dir, f))])
    concepts = sorted(list(set([f.rsplit('_16x', 1)[0] for f in files])))
else:
    concepts = [os.path.splitext(f)[0] for f in thumbs]

try:
    idx = int('$INDEX')
    if 0 <= idx < len(concepts):
        concept = concepts[idx]
        
        # Prefer 16:10 for 16:10 displays (1920x1200 / 2560x1600 / 3840x2400)
        candidates = [
            os.path.join(wall_dir, f'{concept}_16x10.mp4'),
            os.path.join(wall_dir, f'{concept}_16x10.png'),
            os.path.join(wall_dir, f'{concept}_16x9.mp4'),
            os.path.join(wall_dir, f'{concept}_16x9.png'),
        ]
        for cand in candidates:
            if os.path.exists(cand):
                print(cand)
                break
except Exception:
    pass
")

if [ -n "$SELECTED" ] && [ -f "$SELECTED" ]; then
    sel_name=$(basename "$SELECTED")
    display_sel=$(echo "${sel_name%.*}" | sed 's/_16x[0-9]*//g' | tr '_' ' ' | tr '-' ' ' | sed -E 's/\b([a-z])/\U\1/g')
    notify-send -a "Wallpaper Manager" -i "preferences-desktop-wallpaper" "Applying Wallpaper..." "$display_sel"
    "$HOME/.config/niri/scripts/apply-wallpaper.sh" "$SELECTED"
fi
