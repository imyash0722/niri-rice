#!/usr/bin/env bash
# update.sh - A simple script to pull all your live configurations into this repository for backing up to GitHub.

echo "Pulling live configurations into the repository..."

# Pull specific tracked folders from .config (excluding vesktop to avoid Discord tokens)
for item in bongocat btop cava fastfetch foot gtk-3.0 gtk-4.0 hypr mako niri rofi satty waybar xdg-desktop-portal zsh; do
    if [ -d "$HOME/.config/$item" ]; then
        cp -r "$HOME/.config/$item" .config/
    fi
done

# Manually copy only the safe config files for vesktop
mkdir -p .config/vesktop
for file in settings.json state.json; do
    if [ -f "$HOME/.config/vesktop/$file" ]; then
        cp "$HOME/.config/vesktop/$file" .config/vesktop/
    fi
done

# Pull specific tracked files from .config
for file in dolphinrc kdeglobals konsolerc mimeapps.list starship.toml; do
    if [ -f "$HOME/.config/$file" ]; then
        cp "$HOME/.config/$file" .config/
    fi
done

# Pull .local tracked content
cp -r "$HOME/.local/share/color-schemes/"* .local/share/color-schemes/ 2>/dev/null || true

# Pull standalone configs and wallpapers
cp "$HOME/.zshrc" .zshrc
cp -r "$HOME/Wallpapers/"* Wallpapers/ 2>/dev/null || true

echo "Done! Run 'git status' to see what changed, then commit and push."
