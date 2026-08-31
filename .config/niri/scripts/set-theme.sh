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

# 3. Waybar colors are dynamically updated via colors.css in pywal-cursor.py

# 4. Update Wallpaper path
echo "$THEMES_DIR/$THEME/wallpaper.mp4" > "$THEMES_DIR/active-wallpaper.txt"

# 5. Extract Pywal colors from wallpaper and apply matching 24px Moga-Neon cursor
if [ -x "$HOME/.config/niri/scripts/pywal-cursor.py" ]; then
    "$HOME/.config/niri/scripts/pywal-cursor.py" --wallpaper "$THEMES_DIR/$THEME/wallpaper.mp4" --size 32
fi

# 5.5 Update KDE Plasma Wallpaper (Static frame + Smart Video Wallpaper config)
if command -v plasma-apply-wallpaperimage &>/dev/null && [ -f "$THEMES_DIR/$THEME/wallpaper.png" ]; then
    plasma-apply-wallpaperimage "$THEMES_DIR/$THEME/wallpaper.png" &>/dev/null || true
fi
if command -v kwriteconfig6 &>/dev/null; then
    kwriteconfig6 --file plasma-org.kde.plasma.desktop-appletsrc --group Containments --group 210 --group Wallpaper --group "luisbocanegra.smart.video.wallpaper.reborn" --group General --key Video "file://$THEMES_DIR/$THEME/wallpaper.mp4" 2>/dev/null || true
fi

# 6. Reload services
if [ -n "$NIRI_SOCKET" ]; then
    niri msg action do-screen-transition
    killall -SIGUSR2 waybar
    "$HOME/.config/niri/scripts/reload.sh"
else
    echo "Not running in Niri. Configuration updated on disk for next boot."
fi

echo "Theme '$THEME' applied successfully!"
