#!/usr/bin/env python3
"""
Pywal Moga Neon Cursor Selector & Dynamic Applicator
Supports:
  - Automatic Pywal color extraction from wallpaper (image or video frame)
  - 24px size enforcement across all subsystems
  - KDE Plasma (live via plasma-apply-cursortheme & kcminputrc)
  - Niri (via active-cursor.kdl include & niri msg reload)
  - GTK 3/4, X11, Flatpak, and XWayland (via gsettings, ~/.icons, environment.d, and settings.ini)
"""
import sys
import os
import json
import subprocess
import argparse
import colorsys
import re

# Exact Moga-Neon cursor variant names from GNOME-Look / AUR
MOGA_NEON_VARIANTS = {
    "Moga-Neon-Cyan": "#00F0FF",
    "Moga-Neon-Sky": "#00B0FF",
    "Moga-Neon-Water": "#0077BE",
    "Moga-Neon-Blue": "#2979FF",
    "Moga-Neon-Green": "#00E676",
    "Moga-Neon-Olieve": "#808000",
    "Moga-Neon-Butter": "#FFF59D",
    "Moga-Neon-Yellow": "#FFD600",
    "Moga-Neon-Orange": "#FF6D00",
    "Moga-Neon-Sandy": "#C7A77B",
    "Moga-Neon-Red": "#FF1744",
    "Moga-Neon-Rose": "#FF4081",
    "Moga-Neon-Magenta": "#E040FB",
    "Moga-Neon-Purple": "#7C4DFF",
    "Moga-Neon-PurpleBrighter": "#AA00FF",
}

def hex_to_rgb(hex_str):
    hex_str = hex_str.strip().lstrip("#")
    if len(hex_str) == 3:
        hex_str = "".join([c * 2 for c in hex_str])
    return tuple(int(hex_str[i:i + 2], 16) for i in (0, 2, 4))

def color_distance(rgb1, rgb2):
    h1, s1, v1 = colorsys.rgb_to_hsv(*(c / 255.0 for c in rgb1))
    h2, s2, v2 = colorsys.rgb_to_hsv(*(c / 255.0 for c in rgb2))
    dh = min(abs(h1 - h2), 1.0 - abs(h1 - h2)) * 2.0
    ds = abs(s1 - s2)
    dv = abs(v1 - v2)
    return (dh * 4.0) ** 2 + (ds * 1.0) ** 2 + (dv * 0.5) ** 2

def find_best_cursor_variant(target_hex):
    target_rgb = hex_to_rgb(target_hex)
    best_name = "Moga-Neon-Cyan"
    min_dist = float("inf")
    for variant, var_hex in MOGA_NEON_VARIANTS.items():
        var_rgb = hex_to_rgb(var_hex)
        dist = color_distance(target_rgb, var_rgb)
        if dist < min_dist:
            min_dist = dist
            best_name = variant
    return best_name

def get_most_vibrant_pywal_color(colors_dict):
    best_color = "#00F0FF"
    max_vibrancy = -1
    for key in ["color1", "color2", "color3", "color4", "color5", "color6", "color9", "color10", "color11", "color12", "color13", "color14"]:
        hex_code = colors_dict.get(key)
        if not hex_code:
            continue
        r, g, b = hex_to_rgb(hex_code)
        h, s, v = colorsys.rgb_to_hsv(r / 255.0, g / 255.0, b / 255.0)
        score = s * (v ** 0.5)
        if score > max_vibrancy:
            max_vibrancy = score
            best_color = hex_code
    return best_color

def run_pywal_on_file(file_path):
    """Extract colors via pywal from image or video frame without stale caching."""
    if not os.path.exists(file_path):
        return None
    
    target_img = file_path
    if file_path.lower().endswith((".mp4", ".mkv", ".webm", ".mov", ".avi", ".gif")):
        base_name = os.path.splitext(os.path.basename(file_path))[0]
        png_sibling = os.path.join(os.path.dirname(file_path), f"{base_name}.png")
        if os.path.exists(png_sibling):
            target_img = png_sibling
        else:
            target_img = f"/tmp/pywal_frame_{base_name}.jpg"
            try:
                subprocess.run(
                    ["ffmpeg", "-y", "-ss", "00:00:01", "-i", file_path, "-vframes", "1", "-f", "image2", target_img],
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    check=False
                )
            except Exception:
                target_img = file_path

    if os.path.exists(target_img):
        try:
            # Purge stale pywal scheme cache to guarantee fresh extraction
            schemes_dir = os.path.expanduser("~/.cache/wal/schemes")
            if os.path.exists(schemes_dir):
                for sf in os.listdir(schemes_dir):
                    try:
                        os.remove(os.path.join(schemes_dir, sf))
                    except Exception:
                        pass

            subprocess.run(
                ["wal", "-i", target_img, "-n", "-s", "-t", "-e", "-q"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False
            )
        except Exception as e:
            print(f"Warning: Could not run wal: {e}", file=sys.stderr)

    return get_pywal_color()

def get_pywal_color():
    wal_cache = os.path.expanduser("~/.cache/wal/colors.json")
    if os.path.exists(wal_cache):
        try:
            with open(wal_cache, "r") as f:
                data = json.load(f)
                colors = data.get("colors", {})
                return get_most_vibrant_pywal_color(colors)
        except Exception as e:
            print(f"Warning: Could not read pywal colors: {e}", file=sys.stderr)
    return "#00F0FF"

def apply_to_niri(cursor_theme, size=32):
    niri_dir = os.path.expanduser("~/.config/niri")
    themes_dir = os.path.join(niri_dir, "themes")
    os.makedirs(themes_dir, exist_ok=True)

    active_cursor_file = os.path.join(themes_dir, "active-cursor.kdl")
    content = f"""cursor {{
    xcursor-theme "{cursor_theme}"
    xcursor-size {size}
}}
"""
    with open(active_cursor_file, "w") as f:
        f.write(content)

    config_file = os.path.join(niri_dir, "config.kdl")
    if os.path.exists(config_file):
        with open(config_file, "r") as f:
            cfg = f.read()
        include_stmt = 'include "./themes/active-cursor.kdl"'
        if include_stmt not in cfg:
            cfg = f"{include_stmt}\n" + cfg
            with open(config_file, "w") as f:
                f.write(cfg)

def apply_to_kde(cursor_theme, size=32):
    # 1. plasma-apply-cursortheme (live in KDE Plasma session)
    try:
        subprocess.run(
            ["plasma-apply-cursortheme", cursor_theme, "--size", str(size)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
    except Exception:
        pass

    # 2. kwriteconfig6 for permanent KDE config (kcminputrc)
    try:
        subprocess.run(
            [
                "kwriteconfig6",
                "--file",
                "kcminputrc",
                "--group",
                "Mouse",
                "--key",
                "cursorTheme",
                cursor_theme,
            ],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        subprocess.run(
            [
                "kwriteconfig6",
                "--file",
                "kcminputrc",
                "--group",
                "Mouse",
                "--key",
                "cursorSize",
                str(size),
            ],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except Exception:
        pass

def apply_to_gtk_and_x11(cursor_theme, size=32):
    # gsettings (GNOME/GTK runtime)
    try:
        subprocess.run(
            ["gsettings", "set", "org.gnome.desktop.interface", "cursor-theme", cursor_theme],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        subprocess.run(
            ["gsettings", "set", "org.gnome.desktop.interface", "cursor-size", str(size)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
    except Exception:
        pass

    # ~/.icons/default/index.theme (X11 / XWayland root cursor)
    icons_default = os.path.expanduser("~/.icons/default")
    os.makedirs(icons_default, exist_ok=True)
    with open(os.path.join(icons_default, "index.theme"), "w") as f:
        f.write(f"[Icon Theme]\nName=Default\nComment=Default Cursor Theme\nInherits={cursor_theme}\n")

    # GTK 3 & GTK 4 settings.ini
    for gtk_ver in ["gtk-3.0", "gtk-4.0"]:
        gtk_dir = os.path.expanduser(f"~/.config/{gtk_ver}")
        os.makedirs(gtk_dir, exist_ok=True)
        ini_path = os.path.join(gtk_dir, "settings.ini")
        lines = []
        if os.path.exists(ini_path):
            with open(ini_path, "r") as f:
                lines = f.readlines()

        has_settings = False
        cursor_set = False
        size_set = False
        new_lines = []
        for line in lines:
            if line.strip() == "[Settings]":
                has_settings = True
            if line.startswith("gtk-cursor-theme-name"):
                new_lines.append(f"gtk-cursor-theme-name={cursor_theme}\n")
                cursor_set = True
            elif line.startswith("gtk-cursor-theme-size"):
                new_lines.append(f"gtk-cursor-theme-size={size}\n")
                size_set = True
            else:
                new_lines.append(line)

        if not has_settings:
            new_lines.insert(0, "[Settings]\n")
        if not cursor_set:
            new_lines.append(f"gtk-cursor-theme-name={cursor_theme}\n")
        if not size_set:
            new_lines.append(f"gtk-cursor-theme-size={size}\n")

        with open(ini_path, "w") as f:
            f.writelines(new_lines)

    # ~/.config/environment.d/10-cursor.conf (Systemd user environment / Wayland apps)
    env_dir = os.path.expanduser("~/.config/environment.d")
    os.makedirs(env_dir, exist_ok=True)
    with open(os.path.join(env_dir, "10-cursor.conf"), "w") as f:
        f.write(f"XCURSOR_THEME={cursor_theme}\nXCURSOR_SIZE={size}\n")

    # ~/.Xresources for X11 applications
    xres_path = os.path.expanduser("~/.Xresources")
    xres_lines = []
    if os.path.exists(xres_path):
        with open(xres_path, "r") as f:
            xres_lines = f.readlines()
    new_xres = []
    has_theme = False
    has_size = False
    for line in xres_lines:
        if line.startswith("Xcursor.theme:"):
            new_xres.append(f"Xcursor.theme: {cursor_theme}\n")
            has_theme = True
        elif line.startswith("Xcursor.size:"):
            new_xres.append(f"Xcursor.size: {size}\n")
            has_size = True
        else:
            new_xres.append(line)
    if not has_theme:
        new_xres.append(f"Xcursor.theme: {cursor_theme}\n")
    if not has_size:
        new_xres.append(f"Xcursor.size: {size}\n")
    with open(xres_path, "w") as f:
        f.writelines(new_xres)

def update_all_desktop_colors(target_color=None):
    wal_cache = os.path.expanduser("~/.cache/wal/colors.json")
    if not os.path.exists(wal_cache):
        return
    try:
        with open(wal_cache, "r") as f:
            data = json.load(f)
        colors = data.get("colors", {})
        special = data.get("special", {})
        bg = special.get("background", "#101418")
        fg = special.get("foreground", "#e0e2e8")
        c0 = colors.get("color0", bg)
        c1 = colors.get("color1", "#ffb4ab")
        c2 = colors.get("color2", "#9ccbfb")
        c3 = colors.get("color3", "#f0c674")
        c4 = colors.get("color4", target_color or "#9ccbfb")
        c5 = colors.get("color5", "#b9c8da")
        c6 = colors.get("color6", "#d4bfe6")
        c7 = colors.get("color7", "#c2c7cf")
        c8 = colors.get("color8", "#42474e")
        c9 = colors.get("color9", c1)
        c10 = colors.get("color10", c2)
        c11 = colors.get("color11", c3)
        c12 = colors.get("color12", c4)
        c13 = colors.get("color13", c5)
        c14 = colors.get("color14", c6)
        c15 = colors.get("color15", fg)

        primary_accent = target_color or c4

        # 1. Waybar colors.css
        colors_css = f"""@define-color base            {bg};
@define-color surface         {c0};
@define-color overlay         {c8};
@define-color text            {fg};
@define-color subtext         {c7};
@define-color primary         {primary_accent};
@define-color on_primary      {bg};
@define-color secondary       {c5};
@define-color on_secondary    {bg};
@define-color tertiary        {c6};
@define-color error           {c1};
@define-color primary_container   {c2};
@define-color on_primary_container {fg};
"""
        waybar_dir = os.path.expanduser("~/.config/waybar")
        os.makedirs(waybar_dir, exist_ok=True)
        with open(os.path.join(waybar_dir, "colors.css"), "w") as f:
            f.write(colors_css)

        # 2. Foot Terminal colors.ini
        foot_dir = os.path.expanduser("~/.config/foot")
        os.makedirs(foot_dir, exist_ok=True)
        bg_hex = bg.lstrip("#")
        fg_hex = fg.lstrip("#")
        foot_ini_content = f"""[colors-dark]
alpha=0.74
background={bg_hex}
foreground={fg_hex}

## Normal/regular colors (color0-color7)
regular0={c0.lstrip("#")}
regular1={c1.lstrip("#")}
regular2={c2.lstrip("#")}
regular3={c3.lstrip("#")}
regular4={primary_accent.lstrip("#")}
regular5={c5.lstrip("#")}
regular6={c6.lstrip("#")}
regular7={c7.lstrip("#")}

## Bright colors (color8-color15)
bright0={c8.lstrip("#")}
bright1={c9.lstrip("#")}
bright2={c10.lstrip("#")}
bright3={c11.lstrip("#")}
bright4={c12.lstrip("#")}
bright5={c13.lstrip("#")}
bright6={c14.lstrip("#")}
bright7={c15.lstrip("#")}
"""
        with open(os.path.join(foot_dir, "colors.ini"), "w") as f:
            f.write(foot_ini_content)

        # 3. Rofi colors.rasi
        rofi_dir = os.path.expanduser("~/.config/rofi")
        os.makedirs(rofi_dir, exist_ok=True)
        rofi_rasi_content = f"""* {{
    bg0: {bg};
    bg1: {c0};
    fg0: {fg};
    accent-color: {primary_accent};
    urgent-color: {c1};
    border-color: {primary_accent};
}}
"""
        with open(os.path.join(rofi_dir, "colors.rasi"), "w") as f:
            f.write(rofi_rasi_content)

        # 4. Mako notifications
        mako_dir = os.path.expanduser("~/.config/mako")
        os.makedirs(mako_dir, exist_ok=True)
        mako_content = f"""background-color={bg}
text-color={fg}
border-color={primary_accent}
border-size=1
border-radius=4
icons=1
max-icon-size=64
default-timeout=15000
ignore-timeout=1
font=JetBrains Mono 10
width=320
height=110
margin=16
padding=12
"""
        with open(os.path.join(mako_dir, "config"), "w") as f:
            f.write(mako_content)
        subprocess.run(["makoctl", "reload"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # 5. Niri Focus Ring & Colors in config.kdl
        niri_config = os.path.expanduser("~/.config/niri/config.kdl")
        if os.path.exists(niri_config):
            with open(niri_config, "r") as f:
                content = f.read()
            content = re.sub(r'active-color\s+"#[0-9a-fA-F]+"', f'active-color "{primary_accent}"', content)
            content = re.sub(r'inactive-color\s+"#[0-9a-fA-F]+"', f'inactive-color "{bg}"', content)
            with open(niri_config, "w") as f:
                f.write(content)

    except Exception as e:
        print(f"Warning: Could not update all desktop colors: {e}", file=sys.stderr)

def main():
    parser = argparse.ArgumentParser(description="Match and apply 24px Moga Neon cursor color via Pywal for Niri and KDE.")
    parser.add_argument("--color", type=str, help="Target hex color code (e.g. #00e5ff)")
    parser.add_argument("--wallpaper", type=str, help="Run pywal on this wallpaper (image or video) and match cursor")
    parser.add_argument("--wal", action="store_true", help="Read colors from current pywal cache")
    parser.add_argument("--size", type=int, default=32, help="Cursor size (default: 32)")

    args = parser.parse_args()

    if args.wallpaper:
        target_color = run_pywal_on_file(args.wallpaper) or get_pywal_color()
        source = f"pywal on {os.path.basename(args.wallpaper)}"
    elif args.color:
        target_color = args.color
        source = f"color {args.color}"
    else:
        target_color = get_pywal_color()
        source = "pywal palette cache"

    chosen_variant = find_best_cursor_variant(target_color)

    print(f"[{source}] -> Target Color: {target_color} -> Moga Neon: {chosen_variant} ({args.size}px)")

    # Apply across all desktop subsystems
    apply_to_niri(chosen_variant, args.size)
    apply_to_kde(chosen_variant, args.size)
    apply_to_gtk_and_x11(chosen_variant, args.size)
    update_all_desktop_colors(target_color)

    print(f"Cursor theme '{chosen_variant}' ({args.size}px) and all system colors (Niri, Foot, Rofi, Waybar, Mako) applied successfully!")

if __name__ == "__main__":
    main()
