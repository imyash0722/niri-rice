#!/bin/bash
# Apply a theme from ~/.config/niri/themes/

THEMES_DIR="$HOME/.config/niri/themes"
THEME="$1"
CONFIG_FILE="$HOME/.config/niri/config.kdl"

if [ -z "$THEME" ]; then
    echo "Usage: $0 <theme-name>"
    echo "Available themes:"
    ls -1 "$THEMES_DIR" | grep -v "active-" | grep -v '\.'
    exit 1
fi

if [ ! -d "$THEMES_DIR/$THEME" ]; then
    echo "Error: Theme '$THEME' not found."
    exit 1
fi

echo "Applying theme: $THEME..."

# 1. Update Niri Animations (Top-level include)
grep "^include" "$THEMES_DIR/$THEME/niri.kdl" > "$THEMES_DIR/active-animations.kdl"

# 2. Update Niri Colors (Modify config.kdl directly)
ACTIVE_COLOR=$(grep "^ *active-color" "$THEMES_DIR/$THEME/niri.kdl" | awk -F'"' '{print $2}')
INACTIVE_COLOR=$(grep "^ *inactive-color" "$THEMES_DIR/$THEME/niri.kdl" | awk -F'"' '{print $2}')

if [ -n "$ACTIVE_COLOR" ] && [ -n "$INACTIVE_COLOR" ]; then
    sed -i -E "s/active-color \".*\"/active-color \"$ACTIVE_COLOR\"/g" "$CONFIG_FILE"
    sed -i -E "s/inactive-color \".*\"/inactive-color \"$INACTIVE_COLOR\"/g" "$CONFIG_FILE"
fi

# 3. Waybar colors are dynamically updated via colors.css in matugen-theme.py

# 4. Update Wallpaper path
echo "$THEMES_DIR/$THEME/wallpaper.mp4" > "$THEMES_DIR/active-wallpaper.txt"

# 5. Extract Matugen colors directly from theme preview frame or wallpaper and apply full desktop theme
WALL_IMG="$THEMES_DIR/$THEME/wallpaper.png"
if [ ! -f "$WALL_IMG" ]; then
    WALL_IMG="$THEMES_DIR/$THEME/wallpaper.mp4"
fi

rice-ctl theme set "$WALL_IMG" --size 32

echo "Theme '$THEME' applied successfully!"
