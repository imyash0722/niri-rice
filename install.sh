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
# Usage: force_link <abs-source> <abs-dest>
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
# Copy a source dir into dst, merging contents recursively (no symlink).
# Used for entries on filesystems without symlink support (e.g. exFAT).
# ---------------------------------------------------------------------------
copy_dir_merge() {
    local src="$1"
    local dst="$2"
    mkdir -p "$dst"
    cp -r "$src"/. "$dst"/
    echo "  Copied (merged): $dst <- $src"
}

# ---------------------------------------------------------------------------
# Link every top-level entry inside repo's .config/ into ~/.config/
# Entries listed in NO_SYMLINK_ITEMS are deep-copied instead of symlinked
# because ~/.config/systemd must be a real dir on btrfs (exFAT can't symlink).
# ---------------------------------------------------------------------------
NO_SYMLINK_ITEMS=("systemd")

link_dir_contents() {
    local src_dir="$1"   # e.g. /path/to/repo/.config
    local dst_dir="$2"   # e.g. ~/.config

    mkdir -p "$dst_dir"

    for src_item in "$src_dir"/*; do
        [ -e "$src_item" ] || continue   # skip if glob matched nothing
        local name
        name="$(basename "$src_item")"

        # Items that must be copied rather than symlinked
        local no_sym=0
        for skip in "${NO_SYMLINK_ITEMS[@]}"; do
            [ "$name" = "$skip" ] && no_sym=1 && break
        done

        if [ "$no_sym" = "1" ]; then
            copy_dir_merge "$src_item" "$dst_dir/$name"
        else
            force_link "$src_item" "$dst_dir/$name"
        fi
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
    cage alacritty fish zsh hypridle hyprlock btop cava mpv \
    kde-cli-tools plasma-pa bluedevil bt-dualboot satty neovim \
    firefox-developer-edition grim slurp wl-clipboard cliphist \
    starship mako ffmpeg jq brightnessctl playerctl rofi-rbw \
    wtype obs-studio imagemagick kwallet kanshi ttf-jetbrains-mono ttf-roboto \
    ttf-hack eza bat batctl-tui fzf zoxide ripgrep fd otf-font-awesome \
    ttf-meslo-nerd ttf-jetbrains-mono-nerd matugen \
    noto-fonts noto-fonts-emoji noto-fonts-cjk ttf-dejavu ttf-liberation \
    breeze breeze-gtk breeze-icons plasma-integration tlp tlp-rdw powerdevil \
    kde-gtk-config kscreen kwalletmanager kwallet-pam kdialog kio-admin \
    kdeconnect kdegraphics-thumbnailers gwenview spectacle kate kcalc krita meld \
    lazygit yazi tealdeer qimgv haruna ark okular libreoffice-still zapzap video2gif \
    xcb-util-cursor xwayland-satellite kde-applications-meta \
    sddm qt6-declarative qt6-5compat qt6-svg qt6-multimedia \
    qt6-multimedia-ffmpeg gst-plugins-base gst-plugins-good gst-plugins-bad \
    gst-plugins-ugly gst-libav gst-plugin-pipewire gst-plugin-va \
    waybar-module-pacman-updates-git awww powertop opentabletdriver \
    plasma6-wallpapers-smart-video-wallpaper-reborn \
    pavucontrol wireplumber pipewire-alsa pipewire-pulse \
    openssh ufw rsync wget unzip unrar \
    vscodium nodejs npm python python-pynvim \
    keyd libinput-tools solaar || echo "  [!] Some packages failed to install — continuing anyway..."

# ---------------------------------------------------------------------------
# 1.1 Universal input, peripheral managers & USB power rules
# ---------------------------------------------------------------------------
echo -e "\n[1.1/6] Configuring universal input, peripheral managers, and USB power rules..."
if [ -d "$REPO_DIR/etc" ]; then
    sudo mkdir -p /etc/keyd /etc/udev/rules.d /etc/modprobe.d /etc/bluetooth
    sudo cp -f "$REPO_DIR/etc/keyd/default.conf" /etc/keyd/default.conf 2>/dev/null || true
    sudo cp -f "$REPO_DIR/etc/udev/rules.d/50-usb-power.rules" /etc/udev/rules.d/50-usb-power.rules 2>/dev/null || true
    if [ -d "$REPO_DIR/etc/modprobe.d" ]; then
        sudo cp -f "$REPO_DIR/etc/modprobe.d/"* /etc/modprobe.d/ 2>/dev/null || true
    fi
    if [ -d "$REPO_DIR/etc/bluetooth" ]; then
        sudo cp -f "$REPO_DIR/etc/bluetooth/"* /etc/bluetooth/ 2>/dev/null || true
    fi
    if [ -d "$REPO_DIR/etc/pam.d" ]; then
        sudo cp -f "$REPO_DIR/etc/pam.d/"* /etc/pam.d/ 2>/dev/null || true
    fi
    sudo udevadm control --reload 2>/dev/null || true
    sudo udevadm trigger --subsystem-match=usb 2>/dev/null || true
    sudo systemctl enable --now keyd 2>/dev/null || true
fi

# ---------------------------------------------------------------------------
# 1.2 SSH, Tailscale, Remote Access & Power Save Exceptions
# ---------------------------------------------------------------------------
echo -e "\n[1.2/6] Configuring SSH, Tailscale, firewall, and power save exceptions..."
sudo systemctl enable --now sshd tailscaled ufw 2>/dev/null || true
sudo ufw allow in on tailscale0 to any port 22 2>/dev/null || true
sudo ufw --force enable 2>/dev/null || true
sudo tailscale set --ssh 2>/dev/null || true

# Deploy logind power save exceptions & TLP thermal configuration
if [ -d "$REPO_DIR/etc/systemd/logind.conf.d" ]; then
    sudo mkdir -p /etc/systemd/logind.conf.d
    sudo cp -f "$REPO_DIR/etc/systemd/logind.conf.d/"* /etc/systemd/logind.conf.d/ 2>/dev/null || true
    sudo systemctl kill -s HUP systemd-logind 2>/dev/null || true
fi
if [ -f "$REPO_DIR/etc/tlp.conf" ]; then
    sudo cp -f "$REPO_DIR/etc/tlp.conf" /etc/tlp.conf 2>/dev/null || true
    sudo systemctl enable --now tlp 2>/dev/null || true
    sudo tlp start 2>/dev/null || true
fi

# Deploy sleep & hibernation configuration
if [ -d "$REPO_DIR/etc/systemd/sleep.conf.d" ]; then
    sudo mkdir -p /etc/systemd/sleep.conf.d
    sudo cp -f "$REPO_DIR/etc/systemd/sleep.conf.d/"* /etc/systemd/sleep.conf.d/ 2>/dev/null || true
fi
if [ -f "$REPO_DIR/etc/systemd/system/hibernate-setup.service" ]; then
    sudo cp -f "$REPO_DIR/etc/systemd/system/hibernate-setup.service" /etc/systemd/system/hibernate-setup.service 2>/dev/null || true
    sudo systemctl daemon-reload 2>/dev/null || true
    sudo systemctl enable --now hibernate-setup.service 2>/dev/null || true
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

# Build & install native Rust rice-ctl
if [ -d "$REPO_DIR/crates/rice-ctl" ]; then
    echo "  Compiling and installing native Rust rice-ctl..."
    cargo build --release --manifest-path "$REPO_DIR/crates/rice-ctl/Cargo.toml"
    mkdir -p "$HOME/.local/bin"
    cp "$REPO_DIR/crates/rice-ctl/target/release/rice-ctl" "$HOME/.local/bin/rice-ctl"
    
    # Create convenience symlinks to rice-ctl multicall binary
    for name in wall.sh rng.sh caf.sh cal.sh barsel.sh powermenu.sh ss.sh note.sh yazi-note.sh filerofi.sh mem.sh dnd-toggle.sh cycle-power-profile battery-rofi battery.sh browse.sh wifi.sh rofi-ssh power-manager fingerprint-setup setup-hibernate-resume.sh idle-suspend.sh lock.sh monitor-setup.sh reload.sh mute-debounce mic-debounce set-animation.sh set-theme.sh apply-wallpaper.sh; do
        ln -sf "$HOME/.local/bin/rice-ctl" "$HOME/.local/bin/$name"
    done
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
# 4. Binary permissions
# ---------------------------------------------------------------------------
echo ""
echo "[4/6] Setting binary and script permissions..."
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
# 6. Apply initial theme & wallpaper (Matugen dynamic extraction via rice-ctl)
# ---------------------------------------------------------------------------
echo ""
echo "[6/6] Applying initial theme & extracting Matugen Material You colors via rice-ctl..."
if command -v rice-ctl &>/dev/null; then
    rice-ctl theme load blue || true
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
echo "=========================================="
echo " Setup complete! Restart or log out and"
echo " select Niri/KDE from your display manager."
echo "=========================================="
