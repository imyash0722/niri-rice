#!/usr/bin/env bash

# Directories
WALLPAPER_DIRS=(
    "$HOME/Pictures/Wall"
    "$HOME/.config/niri/themes/blue"
    "$HOME/.config/niri/themes/cyan"
    "$HOME/.config/niri/themes/green"
    "$HOME/.config/niri/themes/pink"
)
CACHE_DIR="$HOME/.cache/thumbnails/wallpapers"
mkdir -p "$CACHE_DIR"

TMPFILE=$(mktemp)

# Gather all wallpapers
for dir in "${WALLPAPER_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        fd --max-depth 1 --type f -e jpg -e jpeg -e png -e webp -e gif -e mp4 . "$dir" >> "$TMPFILE"
    fi
done

if [ ! -s "$TMPFILE" ]; then
    notify-send "Wallpaper Picker" "No wallpapers found in ~/Pictures/Wall"
    rm -f "$TMPFILE"
    exit 0
fi

# Sort and filter unique paths
sort -u "$TMPFILE" -o "$TMPFILE"

# Prepare entries with thumbnails
INDEX=$(
    while read -r filepath; do
        filename=$(basename "$filepath")
        name_no_ext="${filename%.*}"
        thumb="$filepath"

        # Generate thumbnail for MP4 / video files
        if [[ "$filepath" =~ \.(mp4|mkv|webm|mov|avi|gif)$ ]]; then
            thumb="$CACHE_DIR/${name_no_ext}_thumb.png"
            if [ ! -f "$thumb" ]; then
                ffmpeg -y -ss 00:00:01 -i "$filepath" -vframes 1 -vf "scale=320:200:force_original_aspect_ratio=increase,crop=320:200" "$thumb" 2>/dev/null || \
                ffmpeg -y -i "$filepath" -vframes 1 -vf "scale=320:200:force_original_aspect_ratio=increase,crop=320:200" "$thumb" 2>/dev/null
            fi
        fi

        printf '%s\x00icon\x1f%s\n' "$name_no_ext" "$thumb"
    done < "$TMPFILE" |
    rofi -dmenu -p "󰸉 Wallpapers" -i -show-icons -format i -theme "$HOME/.config/rofi/wallpaper-select.rasi"
)

[ -z "$INDEX" ] && {
    rm -f "$TMPFILE"
    exit 0
}

SELECTED=$(sed -n "$((INDEX + 1))p" "$TMPFILE")
rm -f "$TMPFILE"

if [ -n "$SELECTED" ] && [ -f "$SELECTED" ]; then
    notify-send -a "Wallpaper Manager" -i "preferences-desktop-wallpaper" "Applying Wallpaper..." "$(basename "$SELECTED")"
    "$HOME/.config/niri/scripts/apply-wallpaper.sh" "$SELECTED"
fi
