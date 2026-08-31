#!/usr/bin/env bash
WALLPAPER_DIR="$HOME/Pictures/Wall"
COLUMNS=5
THUMB_SIZE=180
TMPFILE=$(mktemp)

mkdir -p "$WALLPAPER_DIR"

fd --max-depth 1 --type f -e jpg -e jpeg -e png -e webp -e gif -e mp4 . "$WALLPAPER_DIR" |
sort > "$TMPFILE"

if [ ! -s "$TMPFILE" ]; then
    notify-send "Wallpaper Menu" "No wallpapers found in $WALLPAPER_DIR"
    rm "$TMPFILE"
    exit 0
fi

INDEX=$(
    cat "$TMPFILE" |
        while read -r filepath; do
            filename=$(basename "$filepath")
            printf '%s\x00icon\x1f%s\n' "$filename" "$filepath"
        done |
        rofi -dmenu -p "󰸉 Wallpaper" -i -show-icons -format i -theme-str "
            window {
                width: 75%;
                location: south;
                y-offset: -20;
                border-radius: 12px;
            }
            listview {
                columns: $COLUMNS;
                flow: horizontal;
                spacing: 12px;
                lines: 3;
            }
            element {
                orientation: vertical;
                padding: 8px;
                border-radius: 8px;
                spacing: 4px;
            }
            element-icon {
                size: ${THUMB_SIZE}px;
                border-radius: 6px;
            }
            element-text {
                padding: 4px 0 0 0;
                font-size: 13px;
                width: ${THUMB_SIZE}px;
                text-align: center;
                horizontal-align: 0.5;
                vertical-align: 0.5;
            }
        "
)

[ -z "$INDEX" ] && {
    rm "$TMPFILE"
    exit 0
}

SELECTED=$(sed -n "$((INDEX + 1))p" "$TMPFILE")
rm "$TMPFILE"

if [ -n "$SELECTED" ]; then
    echo "Applying selected wallpaper: $SELECTED"
    "$HOME/.config/niri/scripts/apply-wallpaper.sh" "$SELECTED"
fi
