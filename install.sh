#!/usr/bin/env bash
set -e

echo "=========================================="
echo " Niri Rice - Installation & Setup Script"
echo "=========================================="

echo "[1/4] Installing required dependencies..."
# This command requires paru (AUR helper) to be installed
paru -S --needed --noconfirm ungoogled-chromium-bin dolphin konsole typora-free-with-plugin vesktop niri waybar rofi-wayland foot fastfetch ly networkmanager plasma-nm kde-cli-tools plasma-pa bluedevil satty btop neovim zsh firefox-developer-edition awww hyprlock grim slurp wl-clipboard cliphist cava starship mako hypridle ffmpeg jq brightnessctl playerctl rofi-rbw wtype obs-studio imagemagick kwallet kanshi ttf-jetbrains-mono ttf-roboto eza bat batctl-tui fzf zoxide ripgrep fd ttf-font-awesome ttf-meslo-nerd breeze breeze-icons plasma-integration power-profiles-daemon lazygit yazi tealdeer qimgv haruna ark okular libreoffice-still zapzap xcb-util-cursor xwayland-satellite

echo "[2/4] Copying config files..."
mkdir -p ~/.config
rsync -a .config/ ~/.config/
if [ -d ".local" ]; then
    mkdir -p ~/.local
    rsync -a .local/ ~/.local/
fi

if [ -f ".zshrc" ]; then
    cp .zshrc ~/
fi

if [ -f "ly/config.ini" ]; then
    echo "  Applying Tokyo Night Ly theme (requires sudo)..."
    sudo cp ly/config.ini /etc/ly/config.ini
    sudo systemctl enable ly@tty2.service || true
fi

echo "[3/4] Copying wallpapers..."
mkdir -p ~/Wallpapers
rsync -a Wallpapers/ ~/Wallpapers/

echo "[4/5] Setting script permissions..."
chmod +x ~/.config/niri/scripts/*.sh
chmod +x ~/.config/hypr/scripts/*.sh

echo "[5/5] Enabling system services..."
sudo systemctl enable --now power-profiles-daemon || true


echo "=========================================="
echo " Niri Rice Installation Complete!"
echo "=========================================="

read -p "Do you want to run the coursework tools installation script (install_tools.sh)? [y/N]: " run_tools
if [[ "$run_tools" =~ ^[Yy]$ ]]; then
    ./install_tools.sh
fi

echo "=========================================="
echo " Setup complete! Restart or log out and select Niri from your display manager."
echo "=========================================="
