<div align="center">

<img src="https://readme-typing-svg.demolab.com?font=JetBrains+Mono&weight=700&size=32&duration=3000&pause=1000&color=7FC8FF&center=true&vCenter=true&width=600&lines=niri-rice;A+niri+Wayland+Desktop+Rice" alt="niri-rice" />

<br/>

![niri](https://img.shields.io/badge/niri-26.04-7fc8ff?style=for-the-badge&logo=wayland&logoColor=white)
![Waybar](https://img.shields.io/badge/Waybar-customised-ffc87f?style=for-the-badge)
![Theme](https://img.shields.io/badge/Theme-Tokyo%20Night-1a1b26?style=for-the-badge&logoColor=white)
![Shell](https://img.shields.io/badge/Shell-ZSH-89b4fa?style=for-the-badge&logo=gnu-bash&logoColor=white)
![Migrated](https://img.shields.io/badge/Migrated%20from-SwayFX-a9b1d6?style=for-the-badge)
![Distro](https://img.shields.io/badge/Distro-CachyOS-00b4d8?style=for-the-badge&logo=archlinux&logoColor=white)

*A scrollable, tiling Wayland desktop built on the niri compositor — migrated from SwayFX. Animated wallpapers, Tokyo Night colours, and a fully custom Waybar.*

</div>

---

## ✨ Features

- 🌊 **niri** — Scrollable, infinite-canvas tiling Wayland compositor
- 🎬 **awww** — Fast and lightweight Wayland wallpaper daemon
- 🖥️ **Waybar** — Custom status bar with workspace indicators and interactive tray (Bluetooth, Network, Volume, Battery)
- 🚀 **Rofi** — App launcher with a custom 'blues' dark theme
- 📸 **Satty** — Screenshot annotation tool (Niri native screenshot support via `Mod+Shift+S`)
- 🔒 **hyprlock** — Modern lockscreen: beautifully blurred static backgrounds, dynamic battery module, and bold digital clock
- 🎵 **Cava** — Audio spectrum visualizer
- 🐾 **Foot** — Fast, GPU-rendered Wayland terminal
- ⭐ **Starship** — Cross-shell prompt
- 📋 **cliphist** — Clipboard history manager (`Mod+V` to open)

---

### 📦 Core Components
    * **Window Manager:** `niri`
    * **Status Bar:** `waybar`
    * **App Launcher:** `rofi-wayland`
    * **Wallpaper:** `awww`
    * **Lock Screen:** `hyprlock` & `hypridle`
    * **Notifications:** `mako`

### 🔧 Dependencies
You can install all necessary packages using `paru` or `yay`.

```bash
paru -S niri waybar rofi-wayland foot fastfetch ly networkmanager \
        bluetui pipemixer satty btop neovim zsh \
        firefox-developer-edition awww hyprlock \
        grim slurp wl-clipboard cliphist wl-paste \
        cava starship mako hypridle ffmpeg jq \
        brightnessctl playerctl ungoogled-chromium-bin dolphin \
        konsole typora-free-with-plugin vesktop rofi-rbw wtype blueman \
        obs-studio imagemagick kdeconnect kwallet kanshi \
        ttf-jetbrains-mono ttf-roboto ttf-font-awesome \
        ttf-meslo-nerd eza bat fzf zoxide ripgrep fd \
        breeze breeze-icons plasma-integration power-profiles-daemon iwd iwgtk \
        lazygit yazi tealdeer qimgv haruna ark okular
```

---

## 📸 Screenshots

> *Add your screenshots to `.github/screenshots/` and they'll appear here.*

| Desktop | Waybar | Overview |
|:---:|:---:|:---:|
| *(coming soon)* | *(coming soon)* | *(coming soon)* |

---

## 🚀 Installation

```bash
# 1. Clone the repo
git clone https://github.com/imyash0722/niri-rice.git ~/niri-rice
cd ~/niri-rice

# 2. Copy configs
rsync -a .config/ ~/.config/

# 3. Extract Neovim config
tar -xzf ~/.config/nvim.tar.gz -C ~/.config/
rm ~/.config/nvim.tar.gz

# 4. Make scripts executable
chmod +x ~/.config/niri/scripts/*.sh

# 5. Set your wallpaper path in config.kdl
#    Search for "awww" in ~/.config/niri/config.kdl and update the path

# 6. Start niri from your display manager (Ly auto-detects it)
```

---

## 🌳 Structure

```
niri-rice/
├── .gitignore
├── README.md
├── .config/
│   ├── niri/
│   │   ├── config.kdl          # Compositor config (keybinds, autostart, rules)
│   │   ├── scripts/
│   │   │   ├── lock.sh         # Smart lockscreen script
│   │   │   ├── power.sh        # Rofi power menu (lock/suspend/reboot/shutdown)
│   │   │   └── reload.sh       # Reload Waybar + awww (Mod+Shift+R)
│   │   └── waybar/             # Niri-specific Waybar config + CSS
│   ├── waybar/                 # Shared Waybar modules and scripts
│   ├── rofi/                   # App launcher theme and config
│   ├── foot/                   # Terminal emulator (Tokyo Night theme)
│   ├── hypr/                   # Hyprlock and hypridle configurations
│   ├── mako/                   # Notification daemon
│   ├── cava/                   # Audio spectrum visualizer
│   ├── mpv/                    # Media player config
│   ├── btop/                   # System monitor themes
│   ├── fastfetch/              # System info fetch
│   ├── satty/                  # Screenshot annotation
│   └── starship.toml           # Shell prompt config
└── Wallpapers/                 # Wallpapers (gitignored — store locally)
```

---

## ⌨️ Key Bindings

| Action | Shortcut |
|--------|----------|
| Terminal | `Mod+Return` |
| App Launcher | `Mod+D` |
| Browser | `Mod+Shift+E` |
| File Manager | `Mod+E` |
| Overview | `Mod+Tab` |
| Float ↔ Tile Focus | <kbd>Mod</kbd>+<kbd>`</kbd> |
| Screenshot (region) | `Mod+Shift+S` |
| Screenshot (screen) | `Mod+S` |
| Screenshot (window) | `Mod+Ctrl+S` |
| Color Picker | `Mod+Shift+P` |
| Hotkey Overlay | `Mod+/` |
| Lock Screen | `Mod+Shift+Q` → Lock |
| Reload Waybar + Wallpaper | `Mod+Shift+R` |
| Toggle Waybar Visibility| `Mod+A` |
| Close Window | `Mod+Q` |

---

## 🔐 Smart Lockscreen

The `hyprlock` configuration automatically uses a perfectly static, pre-extracted background from your wallpaper to keep the blur effect fast and smooth. It completely replaces the old `swaylock` screenshot method, avoiding any screen-tearing or see-through glitches! 

It also actively hooks into your system's raw battery capacity (via `BAT0`) using a custom script to display dynamic battery icons directly underneath the password prompt.

---

## 🔧 System Scripts (Hardware Fixes)

Inside the `system-scripts/` directory at the root of this repo, you will find a fix for the **MediaTek mt7921e Wi-Fi card** failing to wake from sleep:

- `mt7921e-sleep.sh`: Must be copied to `/usr/lib/systemd/system-sleep/` and made executable (`chmod +x`). It safely unloads the driver before sleep and reloads it on wake to prevent the card from dropping off the PCIe bus.

---

## 🎨 Colour Palette

| Role | Hex |
|------|-----|
| Background | `#1a1b26` |
| Foreground | `#c0caf5` |
| Blue | `#7fc8ff` |
| Orange | `#ffc87f` |
| Selection | `#2a2c3e` |

---

<div align="center">

*Original Sway dotfiles base by [jim-fx](https://github.com/jim-fx/.dotfiles).*<br>
*Made with 💙 on CachyOS*

</div>
