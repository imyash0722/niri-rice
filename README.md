<div align="center">

<img src="https://readme-typing-svg.demolab.com?font=JetBrains+Mono&weight=700&size=32&duration=3000&pause=1000&color=7FC8FF&center=true&vCenter=true&width=600&lines=niri-rice;A+niri+Wayland+Desktop+Rice" alt="niri-rice" />

<br/>

![niri](https://img.shields.io/badge/niri-26.04-7fc8ff?style=for-the-badge&logo=wayland&logoColor=white)
![Waybar](https://img.shields.io/badge/Waybar-customised-ffc87f?style=for-the-badge)
![Theme](https://img.shields.io/badge/Theme-Tokyo%20Night-1a1b26?style=for-the-badge&logoColor=white)
![Shell](https://img.shields.io/badge/Shell-ZSH-89b4fa?style=for-the-badge&logo=gnu-bash&logoColor=white)
![Migrated](https://img.shields.io/badge/Migrated%20from-SwayFX-a9b1d6?style=for-the-badge)

*A scrollable, tiling Wayland desktop built on the niri compositor — migrated from SwayFX. Animated wallpapers, Tokyo Night colours, and a fully custom Waybar.*

</div>

---

## ✨ Features

- 🌊 **niri** — Scrollable, infinite-canvas tiling Wayland compositor
- 🎬 **mpvpaper** — Animated GIF / video wallpapers rendered by MPV
- 🖥️ **Waybar** — Custom status bar with workspace indicators and interactive tray (Bluetooth, Network, Volume, Battery)
- 🚀 **Rofi** — App launcher with a squared-nord dark theme
- 📸 **Satty** — Screenshot annotation tool (Niri native screenshot support via `Mod+Shift+S`)
- 🔒 **Swaylock** — Smart lockscreen: auto-extracts a frame from your animated wallpaper as the lock background
- 🎵 **Cava** — Audio spectrum visualizer
- 🐾 **Foot** — Fast, GPU-rendered Wayland terminal
- ⭐ **Starship** — Cross-shell prompt
- 📋 **cliphist** — Clipboard history manager (`Mod+V` to open)

---

## 📸 Screenshots

> *Add your screenshots to `.github/screenshots/` and they'll appear here.*

| Desktop | Waybar | Overview |
|:---:|:---:|:---:|
| *(coming soon)* | *(coming soon)* | *(coming soon)* |

---

## 📦 Dependencies

```bash
paru -S niri waybar rofi-wayland foot fastfetch ly networkmanager \
        bluetui pipemixer satty btop neovim zsh \
        firefox-developer-edition mpvpaper swaylock-effects \
        grim slurp wl-clipboard cliphist wl-paste \
        cava starship dunst swayidle ffmpeg jq \
        brightnessctl playerctl
```

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
#    Search for "mpvpaper" in ~/.config/niri/config.kdl and update the path

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
│   │   │   ├── lock.sh         # Smart lockscreen (extracts wallpaper frame)
│   │   │   ├── power.sh        # Rofi power menu (lock/suspend/reboot/shutdown)
│   │   │   └── reload.sh       # Reload Waybar + mpvpaper (Mod+Shift+R)
│   │   └── waybar/             # Niri-specific Waybar config + CSS
│   ├── waybar/                 # Shared Waybar modules and scripts
│   ├── rofi/                   # App launcher theme and config
│   ├── foot/                   # Terminal emulator (Tokyo Night theme)
│   ├── dunst/                  # Notification daemon
│   ├── swaylock/               # Lock screen config
│   ├── cava/                   # Audio visualizer
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
| Float ↔ Tile Focus | `Mod+\`` |
| Screenshot (region) | `Mod+Shift+S` |
| Screenshot (screen) | `Mod+S` |
| Screenshot (window) | `Mod+Ctrl+S` |
| Color Picker | `Mod+Shift+P` |
| Hotkey Overlay | `Mod+/` |
| Lock Screen | `Mod+Shift+Q` → Lock |
| Reload Waybar + Wallpaper | `Mod+Shift+R` |
| Close Window | `Mod+Q` |

---

## 🔐 Smart Lockscreen

The `lock.sh` script automatically detects your current wallpaper:
- If it's a **GIF or video** (`mpvpaper`) → extracts a single frame with `ffmpeg` and uses it as the lockscreen background
- If it's a **static image** (`swaybg`) → uses it directly

This means your lockscreen always matches your wallpaper without any manual configuration.

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

*Made with 💙 on Arch Linux*

</div>
