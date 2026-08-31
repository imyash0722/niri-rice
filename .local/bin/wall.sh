#!/usr/bin/env bash

# Dedicated Wallpaper directory
WALL_DIR="$HOME/Pictures/Wall"
CACHE_DIR="$HOME/.cache/thumbnails/wallpapers"
mkdir -p "$CACHE_DIR"

if [ ! -d "$WALL_DIR" ]; then
    notify-send "Wallpaper Picker" "Folder ~/Pictures/Wall not found"
    exit 1
fi

TMPFILE=$(mktemp)

# Gather all wallpapers exclusively from ~/Pictures/Wall
fd --max-depth 1 --type f -e jpg -e jpeg -e png -e webp -e gif -e mp4 . "$WALL_DIR" | sort -u > "$TMPFILE"

if [ ! -s "$TMPFILE" ]; then
    notify-send "Wallpaper Picker" "No wallpapers found in ~/Pictures/Wall"
    rm -f "$TMPFILE"
    exit 0
fi

# Prepare entries with full-card 16:10 thumbnails and clean titles
INDEX=$(
    while read -r filepath; do
        filename=$(basename "$filepath")
        name_no_ext="${filename%.*}"
        thumb="$CACHE_DIR/${name_no_ext}_card.png"

        # Format title cleanly (e.g. cyber_samurai_red -> Cyber Samurai Red)
        display_name=$(echo "$name_no_ext" | tr '_' ' ' | tr '-' ' ' | sed -E 's/\b([a-z])/\U\1/g')

        # Generate uniform 16:10 aspect ratio card thumbnail
        if [ ! -f "$thumb" ]; then
            if [[ "$filepath" =~ \.(mp4|mkv|webm|mov|avi|gif)$ ]]; then
                ffmpeg -y -ss 00:00:01 -i "$filepath" -vframes 1 -vf "scale=480:300:force_original_aspect_ratio=increase,crop=480:300" "$thumb" 2>/dev/null || \
                ffmpeg -y -i "$filepath" -vframes 1 -vf "scale=480:300:force_original_aspect_ratio=increase,crop=480:300" "$thumb" 2>/dev/null
            else
                ffmpeg -y -i "$filepath" -vf "scale=480:300:force_original_aspect_ratio=increase,crop=480:300" "$thumb" 2>/dev/null || thumb="$filepath"
            fi
        fi

        printf '%s\x00icon\x1f%s\n' "$display_name" "$thumb"
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
    sel_name=$(basename "$SELECTED")
    display_sel=$(echo "${sel_name%.*}" | tr '_' ' ' | tr '-' ' ' | sed -E 's/\b([a-z])/\U\1/g')
    notify-send -a "Wallpaper Manager" -i "preferences-desktop-wallpaper" "Applying Wallpaper..." "$display_sel"
    "$HOME/.config/niri/scripts/apply-wallpaper.sh" "$SELECTED"
fi
