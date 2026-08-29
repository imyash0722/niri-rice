#!/bin/bash
# Apply a theme from ~/.config/niri/themes/

THEMES_DIR="$HOME/.config/niri/themes"
THEME="$1"

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

# 1. Update Niri colors
cp "$THEMES_DIR/$THEME/niri.kdl" "$THEMES_DIR/active-niri.kdl"

# 2. Update Waybar colors
cp "$THEMES_DIR/$THEME/waybar.css" "$HOME/.config/waybar/style.css"

# 3. Update Wallpaper path
echo "$THEMES_DIR/$THEME/wallpaper.mp4" > "$THEMES_DIR/active-wallpaper.txt"

# 4. Reload services
niri msg action do-screen-transition
killall -SIGUSR2 waybar
"$HOME/.config/niri/scripts/reload.sh"

echo "Theme '$THEME' applied successfully!"
