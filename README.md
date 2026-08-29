<div align="center">

<img src="https://readme-typing-svg.demolab.com?font=JetBrains+Mono&weight=700&size=32&duration=3000&pause=1000&color=7FC8FF&center=true&vCenter=true&width=600&lines=niri-rice;A+niri+Wayland+Desktop+Rice" alt="niri-rice" />

<br/>

![niri](https://img.shields.io/badge/niri-26.04-7fc8ff?style=for-the-badge&logo=wayland&logoColor=white)
![Waybar](https://img.shields.io/badge/Waybar-customised-ffc87f?style=for-the-badge)
![Theme](https://img.shields.io/badge/Theme-Pywal%20Dynamic-ff69b4?style=for-the-badge&logoColor=white)
![Shell](https://img.shields.io/badge/Shell-ZSH-89b4fa?style=for-the-badge&logo=gnu-bash&logoColor=white)
![Migrated](https://img.shields.io/badge/Migrated%20from-SwayFX-a9b1d6?style=for-the-badge)
![Distro](https://img.shields.io/badge/Distro-CachyOS-00b4d8?style=for-the-badge&logo=archlinux&logoColor=white)

*A scrollable, tiling Wayland desktop built on the niri compositor. Features animated wallpapers, dynamic Pywal colours, and a fully custom Waybar.*

</div>

---

## ✨ Features

- 🌊 **niri** — Scrollable, infinite-canvas tiling Wayland compositor
- 📺 **Dynamic Display Scaling** — Monitor daemon auto-scales outputs on connect (`1.5x` for 4K, `1.25x` for 1440p, `1.0x` for 1080p)
- 🎬 **mpvpaper** — GPU-accelerated animated wallpapers (MP4/GIF) via mpv
- 🖥️ **Waybar** — Custom status bar with workspace indicators and interactive tray (Bluetooth, Network, Volume, Battery)
- 🚀 **Rofi** — App launcher with a custom dark theme; toggles instantly via hardware Copilot key
- 📸 **Satty** — Screenshot annotation tool (native Niri screenshot support via `Mod+Shift+S`)
- 🔒 **qylock (quickshell)** — Modern lockscreen with blurred background, dynamic battery indicator, and digital clock
- 🐾 **Foot** — Fast, GPU-rendered Wayland terminal
- ⭐ **Starship** — Cross-shell prompt
- 📋 **cliphist** — Clipboard history manager (`Mod+V` to open)
- 💻 **TTY Session** — Minimal fullscreen terminal session via `cage` — selectable from the SDDM login screen dropdown

---

### 📦 Core Components

| Role | Component |
|------|-----------|
| Window Manager | `niri` |
| Status Bar | `waybar` |
| App Launcher | `rofi` |
| Wallpaper | `mpvpaper` |
| Lock Screen | `qylock (quickshell)` + `hypridle` |
| Notifications | `mako` |
| Terminal | `foot` + `alacritty` (TTY session) |

### 🔧 Dependencies

You don't need to install dependencies manually! The included `install.sh` handles everything — packages, fonts, KDE utilities, GStreamer codecs, display drivers, udev rules, and system services.

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

# 2. Run the automated installer
./install.sh

# 3. Log out and select "niri" from the SDDM menu
#    Or select "Terminal Mode" for the minimal TTY session
```

> [!NOTE]
> The animated wallpaper (MP4) is **not bundled** due to file size. Download it from the [Wallpaper Credits](#️-wallpaper-credits) section and place the `.mp4` file in `~/niri-rice/Pictures/Wall/` before running the installer.

---

## 🌳 Structure

```
niri-rice/
├── install.sh                  # One-click automated installer
├── update.sh                   # Pull latest changes
├── .config/
│   ├── niri/
│   │   ├── config.kdl          # Compositor config (keybinds, autostart, rules)
│   │   ├── scripts/
│   │   │   ├── lock.sh         # Smart lockscreen invoker
│   │   │   ├── power.sh        # Rofi power menu (lock/suspend/reboot/shutdown)
│   │   │   ├── reload.sh       # Reload Waybar + wallpaper
│   │   │   ├── monitor-setup.sh# Dynamic display auto-scaling
│   │   │   ├── mute-debounce   # Mute LED sync debounce helper
│   │   │   └── mic-debounce    # Mic mute LED sync debounce helper
│   │   └── waybar/             # Niri-specific Waybar config + CSS
│   ├── waybar/                 # Shared Waybar modules and scripts
│   ├── rofi/                   # App launcher theme and config
│   ├── foot/                   # Foot terminal config
│   ├── hypr/                   # hypridle config (idle/lock daemon)
│   ├── mako/                   # Notification daemon config
│   ├── mpv/                    # mpv player config (used by mpvpaper)
│   ├── btop/                   # System monitor themes
│   ├── fastfetch/              # System info fetch config
│   ├── satty/                  # Screenshot annotation config
│   ├── vesktop/                # Discord (Vesktop) config
│   ├── nvim/                   # LazyVim Neovim configuration
│   └── systemd/user/           # Background services (monitor hotplug)
├── setup_assets/
│   ├── cursors.tar.gz          # Custom cursor themes
│   └── terminal.desktop        # SDDM "Terminal Mode" session (cage + alacritty)
├── system-scripts/             # Hardware-specific system scripts
│   └── mt7921e-sleep.sh        # MediaTek Wi-Fi sleep fix
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
| Lock Screen | `Mod+Shift+Q` |
| Reload Waybar + Wallpaper | `Mod+Shift+R` |
| Toggle Waybar | `Mod+A` |
| Close Window | `Mod+Q` |

> *Hardware keys (Copilot, Calculator, Screen Lock, Mic Mute, Volume) are natively mapped in `config.kdl`.*

---

## 🔐 Smart Lockscreen

`qylock` (quickshell) provides a modern lockscreen with a blurred wallpaper background, a bold digital clock, and a dynamic battery indicator sourced directly from `/sys/class/power_supply/BAT0/capacity`.

`hypridle` manages automatic lock and screen-off timeouts.

---

## 💻 TTY / Terminal Session

A minimal **"Terminal Mode"** session is available directly from the SDDM login screen. It launches `alacritty` inside a `cage` Wayland kiosk compositor — giving you a clean fullscreen terminal without loading any desktop environment. Useful for server administration, recovery, or lightweight work.

---

## 🔧 System & Hardware Fixes Included

The installer automatically applies several kernel and driver-level fixes:

| Fix | Description |
|-----|-------------|
| **Numpad Enter** | Remapped to `Return` at kernel level via `systemd-hwdb`, fixing quickshell lockscreen hang |
| **MediaTek mt7921e Wi-Fi** | Driver unloads before sleep, reloads on wake to prevent PCIe drop |
| **Hardware Audio LEDs** | `brightnessctl` syncs physical mute/mic-mute LEDs with Wireplumber state via SysFS |
| **Copilot Key** | `Super+Shift+F23` mapped to fast-toggling `rofi` app launcher via Niri `spawn-sh` |
| **Wi-Fi Power Save** | NetworkManager powersave disabled globally to prevent MediaTek crashes |
| **Monitor Auto-scaling** | udev hotplug triggers `niri-monitor-hotplug.service` to re-apply scale on plug/unplug |
| **Lid Switch** | `systemd-logind` ignores lid close on AC power to keep Tailscale/SSH alive |

---

## 🎨 Colour Palette

Colours are extracted from the wallpaper via **pywal** and applied to Waybar and Rofi automatically.

| Role | Hex |
|------|-----|
| Background | `#080814` |
| Foreground | `#c7c3c7` |
| Accent (Steel Blue) | `#3C465A` |
| Muted Purple | `#443C45` |
| Dark Mauve | `#50474F` |
| Mid Grey | `#857C86` |

---

## 🖼️ Wallpaper Credits

| Asset | Source |
|:------|:-------|
| **Silent Katana — Forest Samurai** (animated MP4, used as desktop wallpaper) | [WallsFlow](https://wallsflow.com/live-wallpapers/anime/761-silent-katana-forest-samurai-live-wallpaper.html) |

> Download the `.mp4` and place it at:
> ```
> ~/niri-rice/Pictures/Wall/6364907-1200p-optimized.mp4
> ```
> The installer will symlink `~/Wallpapers` and play it via `mpvpaper` automatically.

---

<div align="center">

*Original Sway dotfiles base by [jim-fx](https://github.com/.dotfiles).*<br>
*Made with love on CachyOS*

</div>
