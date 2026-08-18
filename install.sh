#!/usr/bin/env bash
set -e

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================="
echo " Niri Rice - Installation & Setup Script"
echo "=========================================="
echo " Repo: $REPO_DIR"
echo ""

# ---------------------------------------------------------------------------
# Helper: create a symlink, backing up whatever was there before (if it
# wasn't already a symlink pointing at our repo).
# Usage: safe_link <abs-source> <abs-dest>
# ---------------------------------------------------------------------------
force_link() {
    local src="$1"
    local dst="$2"

    # Already correctly linked — nothing to do
    if [ -L "$dst" ] && [ "$(readlink "$dst")" = "$src" ]; then
        return 0
    fi

    # Forcefully remove whatever is currently there (no backups)
    if [ -e "$dst" ] || [ -L "$dst" ]; then
        echo "  Overwriting existing: $dst"
        rm -rf "$dst"
    fi

    mkdir -p "$(dirname "$dst")"
    ln -sf "$src" "$dst"
    echo "  Linked: $dst -> $src"
}

# ---------------------------------------------------------------------------
# Link every top-level entry inside repo's .config/ into ~/.config/
# This means: git pull = instant update, no re-running install needed.
# ---------------------------------------------------------------------------
link_dir_contents() {
    local src_dir="$1"   # e.g. /path/to/repo/.config
    local dst_dir="$2"   # e.g. ~/.config

    mkdir -p "$dst_dir"

    for src_item in "$src_dir"/*; do
        [ -e "$src_item" ] || continue   # skip if glob matched nothing
        local name
        name="$(basename "$src_item")"
        force_link "$src_item" "$dst_dir/$name"
    done
}

# ---------------------------------------------------------------------------
# 1. Dependencies
# ---------------------------------------------------------------------------
echo "[1/6] Installing required dependencies..."
paru -S --needed --noconfirm \
    google-chrome dolphin konsole \
    niri waybar rofi-wayland foot fastfetch networkmanager plasma-nm \
    kde-cli-tools plasma-pa bluedevil bt-dualboot satty btop neovim zsh \
    firefox-developer-edition awww grim slurp wl-clipboard cliphist \
    starship mako hypridle ffmpeg jq brightnessctl playerctl rofi-rbw \
    wtype obs-studio imagemagick kwallet kanshi ttf-jetbrains-mono ttf-roboto \
    ttf-hack eza bat batctl-tui fzf zoxide ripgrep fd ttf-font-awesome \
    ttf-meslo-nerd ttf-jetbrains-mono-nerd python-pywal \
    breeze breeze-icons plasma-integration power-profiles-daemon plasma-powerdevil \
    lazygit yazi tealdeer qimgv haruna ark okular libreoffice-still zapzap video2gif \
    xcb-util-cursor xwayland-satellite kde-applications-meta \
    sddm quickshell qt6-declarative qt6-5compat qt6-svg qt6-multimedia \
    qt6-multimedia-ffmpeg gst-plugins-base gst-plugins-good gst-plugins-bad \
    gst-plugins-ugly waybar-module-pacman-updates mpvpaper || echo "  [!] Some packages failed to install — continuing anyway..."

# ---------------------------------------------------------------------------
# 1.5. Clone qylock themes
# ---------------------------------------------------------------------------
echo -e "\n[1.5/5] Cloning qylock lockscreens/SDDM themes..."
if [ ! -d "$HOME/.local/share/qylock" ]; then
    git clone https://github.com/Darkkal44/qylock "$HOME/.local/share/qylock"
else
    echo "  qylock already exists in ~/.local/share/qylock, pulling latest..."
    git -C "$HOME/.local/share/qylock" pull
fi

# ---------------------------------------------------------------------------
# 2. Symlink config files (live — git pull auto-updates everything)
# ---------------------------------------------------------------------------
echo ""
echo "[2/6] Symlinking config files..."

# ~/.config/* -> repo/.config/*
link_dir_contents "$REPO_DIR/.config" "$HOME/.config"

# ~/.local/* -> repo/.local/*
if [ -d "$REPO_DIR/.local" ]; then
    # Go one level deeper so we link share/konsole etc individually
    for subdir in "$REPO_DIR/.local"/*/; do
        [ -d "$subdir" ] || continue
        local_sub="$(basename "$subdir")"
        link_dir_contents "$subdir" "$HOME/.local/$local_sub"
    done
fi

# Extract custom cursor themes
if [ -f "$REPO_DIR/setup_assets/cursors.tar.gz" ]; then
    echo "  Extracting custom cursor themes..."
    mkdir -p "$HOME/.local/share/icons"
    tar -xzf "$REPO_DIR/setup_assets/cursors.tar.gz" -C "$HOME/.local/share/icons/"
fi

# ~/.zshrc
if [ -f "$REPO_DIR/.zshrc" ]; then
    force_link "$REPO_DIR/.zshrc" "$HOME/.zshrc"
fi

# ---------------------------------------------------------------------------
# 2.5. Wallpaper symlink
# ---------------------------------------------------------------------------
echo ""
echo "[2.5/6] Setting up wallpaper..."
force_link "$REPO_DIR/Wallpapers" "$HOME/Wallpapers"
echo "  Wallpaper: Silent Katana Forest Samurai (WallsFlow)"

# ---------------------------------------------------------------------------
# 3. Enable Display Manager (SDDM)
# ---------------------------------------------------------------------------
echo ""
echo "[3/6] Enabling SDDM display manager..."
sudo systemctl disable ly.service || true
sudo systemctl enable sddm.service || true

# ---------------------------------------------------------------------------
# 4. Script permissions
# ---------------------------------------------------------------------------
echo ""
echo "[4/6] Setting script permissions..."
[ -d "$HOME/.config/niri/scripts" ] && chmod +x "$HOME/.config/niri/scripts"/*.sh 2>/dev/null || true
[ -d "$HOME/.config/hypr/scripts" ] && chmod +x "$HOME/.config/hypr/scripts"/*.sh 2>/dev/null || true
chmod +x "$HOME/.local/bin/"* 2>/dev/null || true

# ---------------------------------------------------------------------------
# 5. System services
# ---------------------------------------------------------------------------
echo ""
echo "[5/6] Enabling system services..."
sudo systemctl enable --now power-profiles-daemon || true

# ---------------------------------------------------------------------------
# 6. Apply wallpaper immediately (awww must be installed)
# ---------------------------------------------------------------------------
echo ""
echo "[6/6] Applying wallpaper..."
if command -v awww &>/dev/null; then
    mpvpaper -o 'no-audio loop hwdec=auto' '*' "$HOME/Wallpapers/silent-katana-forest-samurai-live-wallpaper-wallsflow-com.mp4" &
    
    echo "  Wallpaper applied!"
else
    echo "  [!] awww not found, wallpaper will apply on next login."
fi

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------
echo ""
echo "=========================================="
echo " Niri Rice Installation Complete!"
echo "=========================================="
echo ""
echo " Config files are SYMLINKED to the repo."
echo " Run 'git pull' inside $REPO_DIR at any"
echo " time to get the latest changes instantly."
echo ""

read -rp "Run the coursework tools install script (install_tools.sh)? [y/N]: " run_tools
if [[ "$run_tools" =~ ^[Yy]$ ]]; then
    bash "$REPO_DIR/install_tools.sh"
fi

echo ""
read -rp "Do you want to install Fingwit (fingerprint authentication utility)? [y/N]: " run_fingwit
if [[ "$run_fingwit" =~ ^[Yy]$ ]]; then
    echo "Installing Fingwit and fprintd..."
    paru -S --needed --noconfirm fingwit fprintd || echo "  [!] Failed to install Fingwit — continuing anyway..."
    sudo systemctl enable --now fprintd || true
    echo "Fingwit installed! You can run 'fingwit' from your app launcher to configure your fingerprints."
fi

echo ""
read -rp "Do you want to configure your qylock lockscreen and SDDM themes now? [Y/n]: " config_qylock
if [[ -z "$config_qylock" || "$config_qylock" =~ ^[Yy]$ ]]; then
    if [ -d "$HOME/.local/share/qylock" ]; then
        echo "Launching SDDM theme selector..."
        (cd "$HOME/.local/share/qylock" && chmod +x sddm.sh && ./sddm.sh || true)
        echo "Launching Quickshell lockscreen selector..."
        (cd "$HOME/.local/share/qylock" && chmod +x quickshell.sh && ./quickshell.sh || true)
    else
        echo "qylock repository not found in ~/.local/share/qylock."
    fi
fi

echo ""
echo "=========================================="
echo " Setup complete! Restart or log out and"
echo " select Niri/KDE from your display manager."
echo "=========================================="
