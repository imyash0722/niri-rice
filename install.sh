#!/usr/bin/env bash
set -e

echo "=========================================="
echo " Niri Rice - Installation & Setup Script"
echo "=========================================="

echo "[1/4] Installing required dependencies..."
# This command requires paru (AUR helper) to be installed
paru -S --needed --noconfirm ungoogled-chromium-bin dolphin konsole typora vesktop niri waybar rofi-wayland foot fastfetch ly networkmanager bluetui pipemixer satty btop neovim zsh firefox-developer-edition mpvpaper swaylock-effects grim slurp wl-clipboard cliphist cava starship dunst swayidle ffmpeg jq brightnessctl playerctl wezterm rofi-rbw wtype blueman obs-studio imagemagick kdeconnect kwallet kanshi ttf-jetbrains-mono ttf-roboto eza bat fzf zoxide ripgrep fd ttf-font-awesome ttf-meslo-nerd breeze breeze-icons

echo "[2/4] Copying config files..."
mkdir -p ~/.config
rsync -a .config/ ~/.config/

if [ -f ".zshrc" ]; then
    cp .zshrc ~/
fi

echo "[3/4] Copying wallpapers..."
mkdir -p ~/Wallpapers
rsync -a Wallpapers/ ~/Wallpapers/

echo "[4/4] Setting script permissions..."
chmod +x ~/.config/niri/scripts/*.sh

echo "=========================================="
echo " Installation Complete!"
echo " Restart or log out and select Niri from your display manager."
echo "=========================================="
