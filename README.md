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
- 📺 **Dynamic Display Scaling** — Custom multi-monitor daemon automatically scales outputs based on native resolution (`1.5x` for 4K, `1.25x` for 1440p, `1.0x` for 1080p).
- 🎬 **awww** — Fast and lightweight Wayland wallpaper daemon
- 🖥️ **Waybar** — Custom status bar with workspace indicators and interactive tray (Bluetooth, Network, Volume, Battery)
- 🚀 **Rofi** — App launcher with a custom 'blues' dark theme. Support for toggling and hardware keys (e.g. Copilot button).
- 📸 **Satty** — Screenshot annotation tool (Niri native screenshot support via `Mod+Shift+S`)
- 🔒 **qylock (quickshell)** — Modern lockscreen: beautifully blurred static backgrounds, dynamic battery module, and bold digital clock
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
* **Lock Screen:** `qylock (quickshell)` & `hypridle`
* **Notifications:** `mako`

### 🔧 Dependencies
You don't need to install dependencies manually! The included `install.sh` handles pulling everything from CachyOS/AUR natively, along with fonts, KDE utilities, gstreamer codecs, and display drivers.

---

## 📸 Screenshots

> *Add your screenshots to `.github/screenshots/` and they'll appear here.*

| Desktop | Waybar | Overview |
|:---:|:---:|:---:|
| *(coming soon)* | *(coming soon)* | *(coming soon)* |

---

## 🚀 Installation

This rice now features a **fully automated installation script** (`install.sh`) that installs dependencies, symlinks all configs, applies system-level hardware fixes, and configures powermanagement.

```bash
# 1. Clone the repo
git clone https://github.com/imyash0722/niri-rice.git ~/niri-rice
cd ~/niri-rice

# 2. Run the automated installer
./install.sh

# 3. Log out of your current session and select "niri" from the SDDM menu.
```

---

## 🌳 Structure

```
niri-rice/
├── install.sh                  # One-click automated installer
├── update.sh
├── .config/
│   ├── niri/
│   │   ├── config.kdl          # Compositor config (keybinds, autostart, rules)
│   │   ├── scripts/
│   │   │   ├── lock.sh         # Smart lockscreen script
│   │   │   ├── power.sh        # Rofi power menu
│   │   │   └── monitor-setup.sh# Dynamic display auto-scaling
│   │   └── waybar/             # Niri-specific Waybar config + CSS
│   ├── waybar/                 # Shared Waybar modules and scripts
│   ├── rofi/                   # App launcher theme and config
│   ├── foot/                   # Terminal emulator (Tokyo Night theme)
│   ├── mako/                   # Notification daemon
│   ├── nvim/                   # LazyVim Neovim configuration
│   └── systemd/user/           # Background services (monitor hotplug)
├── system-scripts/             # Hardware-specific system scripts
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

> *Note: Dedicated laptop hardware keys (e.g., Copilot, Calculator, Screen Lock, Mic Mute) are natively mapped in `config.kdl` to automatically trigger these same features.*

---

## 🔐 Smart Lockscreen

The `qylock` configuration automatically uses a beautifully animated video background (via Quickshell) from your wallpaper to keep the blur effect fast and smooth. It completely replaces the old `swaylock` screenshot method, avoiding any screen-tearing or see-through glitches! 

It also actively hooks into your system's raw battery capacity (via `BAT0`) using a custom script to display dynamic battery icons directly underneath the password prompt.

---

## 🔧 System & Hardware Fixes Included

The automated installer and configurations natively apply several kernel and driver-level fixes for standard Lenovo ThinkBook / HP hardware:

- **Hardware Numpad Enter:** Remaps the Numpad Enter key natively to `Return` at the kernel level using `systemd-hwdb` to bypass Wayland virtual keyboard lockscreen bugs.
- **MediaTek mt7921e Wi-Fi:** Unloads the driver before sleep and reloads it on wake to prevent the card from dropping off the PCIe bus.
- **Hardware Audio LEDs:** Uses `brightnessctl` bound directly to SysFS `platform::mute` and `platform::micmute` variables to physically sync the LED indicators on the keyboard with Wireplumber volumes.
- **Copilot Key:** Wayland natively interprets the hardware macro (`Super+Shift+F23`) and maps it to a fast-toggling `rofi` app launcher instance.
- **Power Delivery:** Sets `systemd-logind` to ignore lid-switch suspension when connected to AC power to keep background servers (e.g., Tailscale) alive remotely.

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
| **Silent Katana — Forest Samurai** (animated, used as desktop wallpaper) | [WallsFlow](https://wallsflow.com/live-wallpapers/anime/761-silent-katana-forest-samurai-live-wallpaper.html) |

> The wallpaper is **not bundled** in this repository due to its large file size (~361 MB GIF / ~44 MB MP4). Download it manually from the link above and place the GIF at:
> ```
> ~/niri-rice/Wallpapers/silent-katana-forest-samurai-live-wallpaper-wallsflow-com.gif
> ```
> The install script will then symlink `~/Wallpapers` and apply it automatically.

---

<div align="center">

*Original Sway dotfiles base by [jim-fx](https://github.com/.dotfiles).*<br>
*Made with 💙 on CachyOS*

</div>
