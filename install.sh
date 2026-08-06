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
safe_link() {
    local src="$1"
    local dst="$2"

    # Already correctly linked — nothing to do
    if [ -L "$dst" ] && [ "$(readlink "$dst")" = "$src" ]; then
        return 0
    fi

    # Something else is there — back it up
    if [ -e "$dst" ] || [ -L "$dst" ]; then
        echo "  Backing up existing: $dst -> ${dst}.bak"
        mv "$dst" "${dst}.bak"
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
        safe_link "$src_item" "$dst_dir/$name"
    done
}

# ---------------------------------------------------------------------------
# 1. Dependencies
# ---------------------------------------------------------------------------
echo "[1/5] Installing required dependencies..."
paru -S --needed --noconfirm \
    ungoogled-chromium-bin dolphin konsole typora-free-with-plugin vesktop \
    niri waybar rofi-wayland foot fastfetch ly networkmanager plasma-nm \
    kde-cli-tools plasma-pa bluedevil satty btop neovim zsh \
    firefox-developer-edition awww hyprlock grim slurp wl-clipboard cliphist \
    cava starship mako hypridle ffmpeg jq brightnessctl playerctl rofi-rbw \
    wtype obs-studio imagemagick kwallet kanshi ttf-jetbrains-mono ttf-roboto \
    ttf-hack eza bat batctl-tui fzf zoxide ripgrep fd ttf-font-awesome \
    ttf-meslo-nerd breeze breeze-icons plasma-integration power-profiles-daemon \
    lazygit yazi tealdeer qimgv haruna ark okular libreoffice-still zapzap \
    xcb-util-cursor xwayland-satellite

# ---------------------------------------------------------------------------
# 2. Symlink config files (live — git pull auto-updates everything)
# ---------------------------------------------------------------------------
echo ""
echo "[2/5] Symlinking config files..."

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

# ~/.zshrc
if [ -f "$REPO_DIR/.zshrc" ]; then
    safe_link "$REPO_DIR/.zshrc" "$HOME/.zshrc"
fi

# ---------------------------------------------------------------------------
# 3. Ly display manager theme (needs sudo — copy, not symlink)
# ---------------------------------------------------------------------------
if [ -f "$REPO_DIR/ly.unused/config.ini" ]; then
    echo ""
    echo "[3/5] Applying Tokyo Night Ly theme (requires sudo)..."
    sudo cp "$REPO_DIR/ly.unused/config.ini" /etc/ly/config.ini
    sudo systemctl enable ly@tty2.service || true
else
    echo "[3/5] Ly config not found, skipping..."
fi

# ---------------------------------------------------------------------------
# 4. Script permissions
# ---------------------------------------------------------------------------
echo ""
echo "[4/5] Setting script permissions..."
[ -d "$HOME/.config/niri/scripts" ] && chmod +x "$HOME/.config/niri/scripts"/*.sh 2>/dev/null || true
[ -d "$HOME/.config/hypr/scripts" ] && chmod +x "$HOME/.config/hypr/scripts"/*.sh 2>/dev/null || true

# ---------------------------------------------------------------------------
# 5. System services
# ---------------------------------------------------------------------------
echo ""
echo "[5/5] Enabling system services..."
sudo systemctl enable --now power-profiles-daemon || true

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
echo "=========================================="
echo " Setup complete! Restart or log out and"
echo " select Niri/KDE from your display manager."
echo "=========================================="
