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
echo "[1/6] Detecting package manager and installing dependencies..."
if command -v paru &>/dev/null; then
    AUR_HELPER="paru"
elif command -v yay &>/dev/null; then
    AUR_HELPER="yay"
else
    AUR_HELPER="sudo pacman"
fi

$AUR_HELPER -Syu --needed --noconfirm \
    google-chrome dolphin konsole \
    niri waybar rofi foot fastfetch networkmanager plasma-nm \
    cage alacritty \
    kde-cli-tools plasma-pa bluedevil bt-dualboot satty btop neovim zsh \
    firefox-developer-edition grim slurp wl-clipboard cliphist \
    starship mako hypridle ffmpeg jq brightnessctl playerctl rofi-rbw \
    wtype obs-studio imagemagick kwallet kanshi ttf-jetbrains-mono ttf-roboto \
    ttf-hack eza bat batctl-tui fzf zoxide ripgrep fd otf-font-awesome \
    ttf-meslo-nerd ttf-jetbrains-mono-nerd python-pywal \
    noto-fonts noto-fonts-emoji noto-fonts-cjk ttf-dejavu ttf-liberation \
    breeze breeze-gtk breeze-icons plasma-integration tlp tlp-rdw powerdevil \
    kde-gtk-config kscreen kwalletmanager kwallet-pam kdialog kio-admin \
    kdeconnect kdegraphics-thumbnailers gwenview spectacle kate kcalc krita meld \
    lazygit yazi tealdeer qimgv haruna ark okular libreoffice-still zapzap video2gif \
    xcb-util-cursor xwayland-satellite kde-applications-meta \
    sddm quickshell qt6-declarative qt6-5compat qt6-svg qt6-multimedia \
    qt6-multimedia-ffmpeg gst-plugins-base gst-plugins-good gst-plugins-bad \
    gst-plugins-ugly gst-libav gst-plugin-pipewire gst-plugin-va \
    waybar-module-pacman-updates-git awww powertop opentabletdriver \
    plasma6-wallpapers-smart-video-wallpaper-reborn \
    pavucontrol wireplumber pipewire-alsa pipewire-pulse \
    openssh ufw rsync wget unzip unrar \
    xdg-user-dirs xdg-desktop-portal-gtk \
    vscodium nodejs npm python python-pynvim \
    libinput-tools || echo "  [!] Some packages failed to install — continuing anyway..."

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

# Copy start icons
if [ -d "$REPO_DIR/setup_assets/icons" ]; then
    echo "  Copying custom start icons..."
    cp -r "$REPO_DIR/setup_assets/icons/"* "$HOME/.local/share/icons/" 2>/dev/null || true
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
force_link "$REPO_DIR/Pictures/Wall" "$HOME/Pictures/Wall"
echo "  Wallpaper: Silent Katana Forest Samurai (WallsFlow)"

# ---------------------------------------------------------------------------
# 3. Enable Display Manager (SDDM)
# ---------------------------------------------------------------------------
echo ""
echo "[3/6] Enabling SDDM display manager..."
sudo systemctl disable ly.service || true
sudo systemctl enable sddm.service || true
sudo mkdir -p /usr/share/wayland-sessions
sudo cp "$REPO_DIR/setup_assets/terminal.desktop" "/usr/share/wayland-sessions/terminal.desktop"

# ---------------------------------------------------------------------------
# 4. Script permissions
# ---------------------------------------------------------------------------
echo ""
echo "[4/6] Setting script permissions..."
[ -d "$HOME/.config/niri/scripts" ] && chmod +x "$HOME/.config/niri/scripts"/* 2>/dev/null || true
[ -d "$HOME/.config/waybar/nuke-script" ] && chmod +x "$HOME/.config/waybar/nuke-script"/* 2>/dev/null || true
[ -d "$HOME/.local/bin" ] && chmod +x "$HOME/.local/bin"/* 2>/dev/null || true

# ---------------------------------------------------------------------------
# 5. System services & Battery Optimization
# ---------------------------------------------------------------------------
echo ""
echo "[5/6] Enabling system services & battery fixes..."
sudo systemctl enable --now tlp || true
sudo systemctl mask power-profiles-daemon || true

# Fix MediaTek Wi-Fi crashing during idle (disable NM powersave globally)
sudo bash -c 'cat << EOF > /etc/NetworkManager/conf.d/default-wifi-powersave-on.conf
[connection]
wifi.powersave = 2
EOF'
sudo systemctl restart NetworkManager || true

# Fix Numpad Enter by remapping it to standard Enter at the kernel level
sudo bash -c 'cat << EOF > /etc/udev/hwdb.d/99-numpad-enter.hwdb
evdev:atkbd:*
 KEYBOARD_KEY_9c=enter
 KEYBOARD_KEY_e01c=enter
EOF'
sudo systemd-hwdb update
sudo udevadm trigger

# Monitor auto-scaling on hotplug (triggers niri-monitor-hotplug.service in user session)
sudo bash -c 'cat << EOF > /etc/udev/rules.d/95-monitor-hotplug.rules
ACTION=="change", SUBSYSTEM=="drm", TAG+="systemd", ENV{SYSTEMD_USER_WANTS}="niri-monitor-hotplug.service"
EOF'
sudo udevadm control --reload-rules

# Enable monitor scaling systemd user service
systemctl --user daemon-reload
systemctl --user enable niri-monitor-setup.service || true

# Enable Powertop auto-tuning on boot (while exempting the buggy Wi-Fi card)
sudo bash -c 'cat << EOF > /etc/systemd/system/powertop.service
[Unit]
Description=Powertop tunings
After=multi-user.target network.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/powertop --auto-tune
ExecStartPost=/usr/bin/iw dev wlan0 set power_save off

[Install]
WantedBy=multi-user.target
EOF'
sudo systemctl daemon-reload || true
sudo systemctl enable --now powertop.service || true

# ---------------------------------------------------------------------------
# 6. Apply initial theme & wallpaper (Pywal dynamic extraction)
# ---------------------------------------------------------------------------
echo ""
echo "[6/6] Applying initial theme & extracting Pywal colors..."
if [ -x "$HOME/.config/niri/scripts/set-theme.sh" ]; then
    "$HOME/.config/niri/scripts/set-theme.sh" blue || true
elif [ -x "$HOME/.config/niri/scripts/apply-wallpaper.sh" ] && [ -f "$HOME/Pictures/Wall/silent_katana_samurai.mp4" ]; then
    "$HOME/.config/niri/scripts/apply-wallpaper.sh" "$HOME/Pictures/Wall/silent_katana_samurai.mp4" || true
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
