use clap::{Parser, Subcommand};
use rand::seq::SliceRandom;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "rice-ctl")]
#[command(
    about = "Unified desktop controller & native theming engine for Niri rice",
    version = "0.3.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Theme and wallpaper management
    Theme {
        #[command(subcommand)]
        action: ThemeAction,
    },
    /// Desktop power actions and interactive menu
    Power {
        #[command(subcommand)]
        action: Option<PowerAction>,
    },
    /// Desktop environment reload (Niri, Waybar, Mako, awww)
    Reload,
    /// Caffeine sleep-inhibit manager
    Caffeine {
        #[arg(default_value = "toggle")]
        action: String,
    },
    /// Audio volume and hardware LED controls
    Volume {
        #[command(subcommand)]
        action: VolumeAction,
    },
    /// Notification management
    Dnd {
        #[arg(default_value = "toggle")]
        action: String,
    },
    /// Media control wrapper
    Media {
        #[arg(default_value = "play-pause")]
        action: String,
    },
}

#[derive(Subcommand)]
enum ThemeAction {
    /// Interactive Rofi wallpaper picker
    Select {
        /// Cursor size in pixels
        #[arg(short, long, default_value = "32")]
        size: u32,
    },
    /// Apply wallpaper, extract Matugen Material You colors, and sync desktop
    Set {
        /// Path to wallpaper image or video
        file: PathBuf,
        /// Cursor size in pixels
        #[arg(short, long, default_value = "32")]
        size: u32,
    },
    /// Pick and apply a random wallpaper from wallpaper directory
    Random {
        /// Wallpaper directory
        #[arg(short, long)]
        dir: Option<PathBuf>,
        /// Cursor size in pixels
        #[arg(short, long, default_value = "32")]
        size: u32,
    },
    /// Load a predefined theme from ~/.config/niri/themes
    Load {
        /// Name of the theme
        name: String,
        /// Cursor size in pixels
        #[arg(short, long, default_value = "32")]
        size: u32,
    },
}

#[derive(Subcommand)]
enum PowerAction {
    /// Open interactive Rofi power menu
    Menu,
    /// Lock the screen via quickshell or loginctl
    Lock,
    /// Suspend to RAM
    Suspend,
    /// Hibernate to disk
    Hibernate,
    /// Reboot system
    Reboot,
    /// Power off system
    Shutdown,
    /// Smart idle suspension with AC / SSH / Caffeine safeguards
    IdleSuspend,
}

#[derive(Subcommand)]
enum VolumeAction {
    /// Toggle audio sink mute with hardware LED feedback
    MuteToggle,
    /// Toggle microphone source mute with hardware LED feedback
    MicMuteToggle,
}

#[derive(Deserialize, Debug)]
struct WalCache {
    #[serde(default)]
    special: HashMap<String, String>,
    #[serde(default)]
    colors: HashMap<String, String>,
}

const MOGA_NEON_VARIANTS: &[(&str, &str)] = &[
    ("Moga-Neon-Cyan", "#00F0FF"),
    ("Moga-Neon-Sky", "#00B0FF"),
    ("Moga-Neon-Water", "#0077BE"),
    ("Moga-Neon-Blue", "#2979FF"),
    ("Moga-Neon-Green", "#00E676"),
    ("Moga-Neon-Olieve", "#808000"),
    ("Moga-Neon-Butter", "#FFF59D"),
    ("Moga-Neon-Yellow", "#FFD600"),
    ("Moga-Neon-Orange", "#FF6D00"),
    ("Moga-Neon-Sandy", "#C7A77B"),
    ("Moga-Neon-Red", "#FF1744"),
    ("Moga-Neon-Rose", "#FF4081"),
    ("Moga-Neon-Magenta", "#E040FB"),
    ("Moga-Neon-Purple", "#7C4DFF"),
    ("Moga-Neon-PurpleBrighter", "#AA00FF"),
];

fn hex_to_rgb(hex: &str) -> (f32, f32, f32) {
    let clean = hex.trim().trim_start_matches('#');
    let s = if clean.len() == 3 {
        clean.chars().flat_map(|c| [c, c]).collect::<String>()
    } else {
        clean.to_string()
    };
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0) as f32 / 255.0;
    (r, g, b)
}

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let v = max;
    let s = if max == 0.0 { 0.0 } else { delta / max };
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        (((g - b) / delta) % 6.0) / 6.0
    } else if max == g {
        (((b - r) / delta) + 2.0) / 6.0
    } else {
        (((r - g) / delta) + 4.0) / 6.0
    };
    let h = if h < 0.0 { h + 1.0 } else { h };
    (h, s, v)
}

fn color_distance(hsv1: (f32, f32, f32), hsv2: (f32, f32, f32)) -> f32 {
    let (h1, s1, v1) = hsv1;
    let (h2, s2, v2) = hsv2;
    let dh_diff = (h1 - h2).abs();
    let dh = (dh_diff.min(1.0 - dh_diff)) * 2.0;
    let ds = (s1 - s2).abs();
    let dv = (v1 - v2).abs();
    (dh * 4.0).powi(2) + (ds * 1.0).powi(2) + (dv * 0.5).powi(2)
}

fn find_best_cursor_variant(target_hex: &str) -> &'static str {
    let (r, g, b) = hex_to_rgb(target_hex);
    let target_hsv = rgb_to_hsv(r, g, b);
    let mut best_name = "Moga-Neon-Cyan";
    let mut min_dist = f32::MAX;
    for (name, hex) in MOGA_NEON_VARIANTS {
        let (vr, vg, vb) = hex_to_rgb(hex);
        let dist = color_distance(target_hsv, rgb_to_hsv(vr, vg, vb));
        if dist < min_dist {
            min_dist = dist;
            best_name = name;
        }
    }
    best_name
}

fn find_matugen_bin() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let candidates = [
        PathBuf::from(&home).join(".cargo/bin/matugen"),
        PathBuf::from(&home).join(".local/bin/matugen"),
        PathBuf::from("/usr/bin/matugen"),
        PathBuf::from("/usr/local/bin/matugen"),
    ];
    for c in candidates {
        if c.is_file() {
            return c;
        }
    }
    PathBuf::from("matugen")
}

fn apply_cursor(variant: &str, size: u32) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());

    // 1. Niri cursor
    let niri_themes = PathBuf::from(&home).join(".config/niri/themes");
    let _ = fs::create_dir_all(&niri_themes);
    let cursor_kdl = format!(
        "cursor {{\n    xcursor-theme \"{}\"\n    xcursor-size {}\n}}\n",
        variant, size
    );
    let _ = fs::write(niri_themes.join("active-cursor.kdl"), cursor_kdl);

    // 2. KDE Plasma live & permanent
    let _ = Command::new("plasma-apply-cursortheme")
        .args([variant, "--size", &size.to_string()])
        .output();
    let _ = Command::new("kwriteconfig6")
        .args([
            "--file",
            "kcminputrc",
            "--group",
            "Mouse",
            "--key",
            "cursorTheme",
            variant,
        ])
        .output();
    let _ = Command::new("kwriteconfig6")
        .args([
            "--file",
            "kcminputrc",
            "--group",
            "Mouse",
            "--key",
            "cursorSize",
            &size.to_string(),
        ])
        .output();

    // 3. GTK 3 & 4 settings.ini and gsettings
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "cursor-theme", variant])
        .output();
    let _ = Command::new("gsettings")
        .args([
            "set",
            "org.gnome.desktop.interface",
            "cursor-size",
            &size.to_string(),
        ])
        .output();

    for gtk in ["gtk-3.0", "gtk-4.0"] {
        let ini_path = PathBuf::from(&home).join(format!(".config/{}/settings.ini", gtk));
        if let Ok(content) = fs::read_to_string(&ini_path) {
            let re_theme = Regex::new(r"gtk-cursor-theme-name=.*").unwrap();
            let re_size = Regex::new(r"gtk-cursor-theme-size=.*").unwrap();
            let updated =
                re_theme.replace_all(&content, format!("gtk-cursor-theme-name={}", variant));
            let updated = re_size.replace_all(&updated, format!("gtk-cursor-theme-size={}", size));
            let _ = fs::write(ini_path, updated.as_bytes());
        }
    }

    // 4. X11 & Environment
    let icons_dir = PathBuf::from(&home).join(".icons/default");
    let _ = fs::create_dir_all(&icons_dir);
    let _ = fs::write(
        icons_dir.join("index.theme"),
        format!(
            "[Icon Theme]\nName=Default\nComment=Default Cursor Theme\nInherits={}\n",
            variant
        ),
    );

    let env_dir = PathBuf::from(&home).join(".config/environment.d");
    let _ = fs::create_dir_all(&env_dir);
    let _ = fs::write(
        env_dir.join("10-cursor.conf"),
        format!("XCURSOR_THEME={}\nXCURSOR_SIZE={}\n", variant, size),
    );
}

fn apply_desktop_colors(target_color: Option<&str>) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let wal_cache = PathBuf::from(&home).join(".cache/wal/colors.json");
    if !wal_cache.exists() {
        return;
    }

    let Ok(data) = fs::read_to_string(&wal_cache) else {
        return;
    };
    let Ok(cache) = serde_json::from_str::<WalCache>(&data) else {
        return;
    };

    let bg = cache
        .special
        .get("background")
        .cloned()
        .unwrap_or_else(|| "#121318".to_string());
    let fg = cache
        .special
        .get("foreground")
        .cloned()
        .unwrap_or_else(|| "#e3e2e9".to_string());
    let primary = target_color
        .map(|s| s.to_string())
        .or_else(|| cache.colors.get("color4").cloned())
        .unwrap_or_else(|| "#b3c5ff".to_string());

    // 1. Update Niri focus-ring colors in config.kdl
    let niri_config = PathBuf::from(&home).join(".config/niri/config.kdl");
    if let Ok(content) = fs::read_to_string(&niri_config) {
        let re_active = Regex::new(r##"active-color\s+"#[0-9a-fA-F]+""##).unwrap();
        let re_inactive = Regex::new(r##"inactive-color\s+"#[0-9a-fA-F]+""##).unwrap();
        let updated = re_active.replace_all(&content, format!("active-color \"{}\"", primary));
        let updated = re_inactive.replace_all(&updated, format!("inactive-color \"{}\"", bg));
        let _ = fs::write(niri_config, updated.as_bytes());
    }

    // 2. Live reload Waybar via SIGUSR2
    let is_waybar_running = Command::new("pgrep")
        .args(["-x", "waybar"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if is_waybar_running {
        let _ = Command::new("killall").args(["-SIGUSR2", "waybar"]).output();
    }

    // 3. Reload Mako notifications
    let _ = Command::new("makoctl").arg("reload").output();

    // 4. Trigger Niri smooth screen transition
    if std::env::var("NIRI_SOCKET").is_ok() {
        let _ = Command::new("niri")
            .args(["msg", "action", "do-screen-transition"])
            .output();
    }

    // 5. Broadcast ANSI OSC sequences to all active terminals (/dev/pts/*)
    let mut seq = Vec::new();
    let esc = "\x1b";
    for i in 0..16 {
        let default_c = if i == 0 { &bg } else { &fg };
        let c = cache.colors.get(&format!("color{}", i)).unwrap_or(default_c);
        seq.extend_from_slice(format!("{}]4;{};{}{}\\", esc, i, c, esc).as_bytes());
    }
    seq.extend_from_slice(format!("{}]10;{}{}\\", esc, fg, esc).as_bytes());
    seq.extend_from_slice(format!("{}]11;{}{}\\", esc, bg, esc).as_bytes());
    seq.extend_from_slice(format!("{}]12;{}{}\\", esc, primary, esc).as_bytes());
    seq.extend_from_slice(format!("{}]13;{}{}\\", esc, primary, esc).as_bytes());
    seq.extend_from_slice(format!("{}]17;{}{}\\", esc, primary, esc).as_bytes());
    seq.extend_from_slice(format!("{}]19;{}{}\\", esc, bg, esc).as_bytes());
    seq.extend_from_slice(format!("{}]4;232;{}{}\\", esc, bg, esc).as_bytes());
    seq.extend_from_slice(format!("{}]4;256;{}{}\\", esc, fg, esc).as_bytes());
    seq.extend_from_slice(format!("{}]708;{}{}\\", esc, bg, esc).as_bytes());

    let _ = fs::write(PathBuf::from(&home).join(".cache/wal/sequences"), &seq);

    if let Ok(entries) = fs::read_dir("/dev/pts") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.chars().all(|c| c.is_ascii_digit()) {
                    if let Ok(mut f) = fs::OpenOptions::new().write(true).open(&path) {
                        let _ = f.write_all(&seq);
                    }
                }
            }
        }
    }

    // 6. Broadcast colorscheme reload to active Neovim instances
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    if let Ok(entries) = fs::read_dir(&runtime_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("nvim") {
                let sock = entry.path().to_string_lossy().to_string();
                let _ = Command::new("nvim")
                    .args([
                        "--server",
                        &sock,
                        "--remote-send",
                        "<Esc>:colorscheme neopywal<CR>",
                    ])
                    .output();
            }
        }
    }

    // 7. Sync D-Bus environment
    let _ = Command::new("dbus-update-activation-environment")
        .args(["--systemd", "--all"])
        .output();
}

fn set_wallpaper(file: &Path, size: u32) {
    let canonical = fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf());
    if !canonical.exists() {
        eprintln!(
            "Error: Wallpaper file '{}' does not exist.",
            canonical.display()
        );
        return;
    }

    println!("[rice-ctl] Setting wallpaper: {}", canonical.display());

    // 1. Resolve preview image for Matugen (if video/gif, extract frame)
    let ext = canonical
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let is_video = matches!(ext.as_str(), "mp4" | "mkv" | "webm" | "mov" | "avi" | "gif");
    let image_target = if is_video {
        let parent = canonical.parent().unwrap_or_else(|| Path::new("/tmp"));
        let stem = canonical
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("wall");
        let png_sibling = parent.join(format!("{}.png", stem));
        if png_sibling.exists() {
            png_sibling
        } else {
            let tmp_frame = PathBuf::from(format!("/tmp/matugen_frame_{}.jpg", stem));
            let _ = Command::new("ffmpeg")
                .args([
                    "-y",
                    "-ss",
                    "00:00:01",
                    "-i",
                    &canonical.to_string_lossy(),
                    "-vframes",
                    "1",
                    "-f",
                    "image2",
                    &tmp_frame.to_string_lossy(),
                ])
                .output();
            tmp_frame
        }
    } else {
        canonical.clone()
    };

    // 2. Run Matugen in Rust
    println!("[rice-ctl] Extracting Material You palette via Matugen...");
    let matugen_bin = find_matugen_bin();
    let status = Command::new(matugen_bin)
        .args([
            "image",
            &image_target.to_string_lossy(),
            "--source-color-index",
            "0",
            "-q",
        ])
        .status();

    if let Err(e) = status {
        eprintln!("Warning: Failed to execute matugen: {}", e);
    }

    // 3. Read primary accent color from generated cache
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let wal_cache = PathBuf::from(&home).join(".cache/wal/colors.json");
    let mut primary_color = "#b3c5ff".to_string();

    if let Ok(data) = fs::read_to_string(&wal_cache) {
        if let Ok(cache) = serde_json::from_str::<WalCache>(&data) {
            if let Some(p) = cache.colors.get("color4") {
                primary_color = p.clone();
            }
        }
    }

    // 4. Cursor matching & application
    let cursor_variant = find_best_cursor_variant(&primary_color);
    println!(
        "[rice-ctl] Matched cursor: {} ({}px) for accent {}",
        cursor_variant, size, primary_color
    );
    apply_cursor(cursor_variant, size);

    // 5. Update desktop colors (Niri, Waybar, Mako, Terminals, Neovim)
    apply_desktop_colors(Some(&primary_color));

    // 6. Apply live wallpaper via awww
    println!("[rice-ctl] Applying live wallpaper via awww...");
    let is_awww_running = Command::new("pgrep")
        .args(["-x", "awww-daemon"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !is_awww_running {
        let _ = Command::new("awww-daemon").spawn();
        std::thread::sleep(std::time::Duration::from_millis(150));
    }

    let _ = Command::new("awww")
        .args([
            "img",
            &image_target.to_string_lossy(),
            "--transition-type",
            "center",
            "--transition-pos",
            "center",
            "--transition-duration",
            "1.2",
            "--transition-fps",
            "120",
            "--transition-bezier",
            ".25,1,.5,1",
        ])
        .output();

    // 7. Save active wallpaper state
    let _ = fs::write(
        PathBuf::from(&home).join(".config/wallpaper"),
        canonical.to_string_lossy().as_bytes(),
    );
    let themes_dir = PathBuf::from(&home).join(".config/niri/themes");
    let _ = fs::write(
        themes_dir.join("active-wallpaper.txt"),
        canonical.to_string_lossy().as_bytes(),
    );

    // 8. Sync KDE plasma wallpaper
    let _ = Command::new("plasma-apply-wallpaperimage")
        .arg(&image_target)
        .output();

    println!("[rice-ctl] Theme and wallpaper switched successfully!");
}

fn select_wallpaper(size: u32) {
    // 1. Singleton Toggle: If already running, close and exit
    let pkill_output = Command::new("pkill")
        .args(["-f", "rofi.*wallpaper-select.rasi"])
        .output();
    if let Ok(out) = pkill_output {
        if out.status.success() {
            return;
        }
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let wall_dir = PathBuf::from(&home).join("Pictures/Wall");
    let thumb_dir = wall_dir.join(".thumbnails");

    if !wall_dir.exists() {
        let _ = Command::new("notify-send")
            .args(["Wallpaper Picker", "Folder ~/Pictures/Wall not found"])
            .output();
        return;
    }

    // 2. Discover unique wallpaper concepts
    let mut concepts = Vec::new();
    if thumb_dir.exists() {
        if let Ok(entries) = fs::read_dir(&thumb_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("png") {
                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                        concepts.push(stem.to_string());
                    }
                }
            }
        }
    }

    if concepts.is_empty() {
        if let Ok(entries) = fs::read_dir(&wall_dir) {
            let re = Regex::new(r"_16x[0-9]+").unwrap();
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !name.starts_with('.') {
                        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        let clean_stem = re.replace_all(stem, "").to_string();
                        concepts.push(clean_stem);
                    }
                }
            }
        }
    }

    concepts.sort();
    concepts.dedup();

    if concepts.is_empty() {
        let _ = Command::new("notify-send")
            .args(["Wallpaper Picker", "No wallpapers found in ~/Pictures/Wall"])
            .output();
        return;
    }

    // 3. Build rofi items: {display}\0icon\x1f{thumb}\n
    let mut rofi_input = Vec::new();
    for c in &concepts {
        let display = c.replace('_', " ").replace('-', " ");
        let display_title: String = display
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let thumb_path = thumb_dir.join(format!("{}.png", c));
        let icon_path = if thumb_path.exists() {
            thumb_path.to_string_lossy().to_string()
        } else {
            let direct = wall_dir.join(format!("{}.png", c));
            if direct.exists() {
                direct.to_string_lossy().to_string()
            } else {
                wall_dir
                    .join(format!("{}_16x10.png", c))
                    .to_string_lossy()
                    .to_string()
            }
        };

        rofi_input.extend_from_slice(format!("{}\0icon\x1f{}\n", display_title, icon_path).as_bytes());
    }

    let rofi_theme = PathBuf::from(&home).join(".config/rofi/wallpaper-select.rasi");
    let mut child = match Command::new("rofi")
        .args([
            "-dmenu",
            "-p",
            "󰸉 Wallpapers",
            "-i",
            "-show-icons",
            "-format",
            "i",
            "-theme",
            &rofi_theme.to_string_lossy(),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error launching rofi: {}", e);
            return;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(&rofi_input);
    }

    let output = match child.wait_with_output() {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Error waiting on rofi: {}", e);
            return;
        }
    };

    let selected_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selected_str.is_empty() {
        return;
    }

    let Ok(idx) = selected_str.parse::<usize>() else {
        return;
    };
    if idx >= concepts.len() {
        return;
    }

    let concept = &concepts[idx];

    // Prefer 16:10 for 16:10 displays, fall back gracefully
    let candidates = [
        wall_dir.join(format!("{}_16x10.mp4", concept)),
        wall_dir.join(format!("{}_16x10.png", concept)),
        wall_dir.join(format!("{}_16x9.mp4", concept)),
        wall_dir.join(format!("{}_16x9.png", concept)),
        wall_dir.join(format!("{}.png", concept)),
        wall_dir.join(format!("{}.jpg", concept)),
        wall_dir.join(format!("{}.jpeg", concept)),
        wall_dir.join(format!("{}.webp", concept)),
        wall_dir.join(format!("{}.mp4", concept)),
    ];

    let mut chosen_file = None;
    for cand in candidates {
        if cand.exists() {
            chosen_file = Some(cand);
            break;
        }
    }

    if let Some(file) = chosen_file {
        let name = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Wallpaper");
        let clean_display = name
            .replace("_16x10", "")
            .replace("_16x9", "")
            .replace('_', " ")
            .replace('-', " ");
        let _ = Command::new("notify-send")
            .args([
                "-a",
                "Wallpaper Manager",
                "-i",
                "preferences-desktop-wallpaper",
                "Applying Wallpaper...",
                &clean_display,
            ])
            .output();

        set_wallpaper(&file, size);
    }
}

fn random_wallpaper(dir: Option<PathBuf>, size: u32) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let wall_dir = dir.unwrap_or_else(|| PathBuf::from(&home).join("Pictures/Wall"));

    if !wall_dir.exists() {
        eprintln!(
            "Error: Wallpaper directory '{}' does not exist.",
            wall_dir.display()
        );
        return;
    }

    let Ok(entries) = fs::read_dir(&wall_dir) else {
        eprintln!("Error: Could not read wallpaper directory.");
        return;
    };

    let mut valid_files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if matches!(
                    ext_lower.as_str(),
                    "png" | "jpg" | "jpeg" | "webp" | "gif" | "mp4"
                ) {
                    valid_files.push(path);
                }
            }
        }
    }

    if valid_files.is_empty() {
        eprintln!(
            "Error: No supported wallpaper images found in '{}'",
            wall_dir.display()
        );
        return;
    }

    let mut rng = rand::thread_rng();
    if let Some(chosen) = valid_files.choose(&mut rng) {
        set_wallpaper(chosen, size);
    }
}

fn load_theme(theme: &str, size: u32) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let themes_dir = PathBuf::from(&home).join(".config/niri/themes");
    let theme_dir = themes_dir.join(theme);

    if !theme_dir.exists() {
        eprintln!(
            "Error: Theme '{}' not found in {}",
            theme,
            themes_dir.display()
        );
        return;
    }

    println!("[rice-ctl] Applying theme: {}...", theme);

    // 1. Update Niri Animations
    let niri_kdl = theme_dir.join("niri.kdl");
    if let Ok(content) = fs::read_to_string(&niri_kdl) {
        let anim_lines: Vec<&str> = content
            .lines()
            .filter(|l| l.trim().starts_with("include"))
            .collect();
        let _ = fs::write(
            themes_dir.join("active-animations.kdl"),
            anim_lines.join("\n"),
        );
    }

    // 2. Wallpaper resolution
    let wall_png = theme_dir.join("wallpaper.png");
    let wall_mp4 = theme_dir.join("wallpaper.mp4");
    let target_wall = if wall_png.exists() {
        wall_png
    } else if wall_mp4.exists() {
        wall_mp4
    } else {
        eprintln!("Warning: No wallpaper found for theme {}", theme);
        return;
    };

    set_wallpaper(&target_wall, size);
    println!("[rice-ctl] Theme '{}' applied successfully!", theme);
}

fn lock_screen() {
    let pgrep = Command::new("pgrep")
        .args(["-f", "quickshell.*lock_shell\\.qml"])
        .output();
    if let Ok(out) = pgrep {
        if out.status.success() {
            return;
        }
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let lock_candidates = [
        PathBuf::from(&home).join(".local/share/quickshell-lockscreen/lock.sh"),
        PathBuf::from(&home).join(".local/share/qylock/quickshell-lockscreen/lock.sh"),
        PathBuf::from(&home).join(".local/share/qylock/lock.sh"),
    ];

    for c in lock_candidates {
        if c.is_file() {
            let _ = Command::new(c).spawn();
            return;
        }
    }

    let _ = Command::new("loginctl").arg("lock-session").spawn();
}

fn power_menu() {
    let options = "Lock\nLogout\nSuspend\nHibernate\nReboot\nShutdown\n";
    let mut child = match Command::new("rofi")
        .args(["-dmenu", "-i", "-p", "Power", "-lines", "6"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error spawning rofi power menu: {}", e);
            return;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(options.as_bytes());
    }

    let Ok(output) = child.wait_with_output() else {
        return;
    };
    let choice = String::from_utf8_lossy(&output.stdout).trim().to_string();

    match choice.as_str() {
        "Lock" => lock_screen(),
        "Logout" => {
            let _ = Command::new("niri").args(["msg", "action", "quit"]).output();
        }
        "Suspend" => {
            let _ = Command::new("systemctl").arg("suspend").spawn();
        }
        "Hibernate" => {
            let _ = Command::new("systemctl").arg("hibernate").spawn();
        }
        "Reboot" => {
            let _ = Command::new("systemctl").arg("reboot").spawn();
        }
        "Shutdown" => {
            let _ = Command::new("systemctl").arg("poweroff").spawn();
        }
        _ => {}
    }
}

fn idle_suspend() {
    // 1. Caffeine Override
    if Path::new("/tmp/caffeine_active").exists() {
        return;
    }

    // 2. Check systemd sleep inhibitors
    if let Ok(out) = Command::new("systemd-inhibit")
        .args(["--list", "--no-legend"])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
        for line in text.lines() {
            if line.contains("block")
                && (line.contains("sleep") || line.contains("idle"))
                && !line.contains("hypridle")
            {
                return;
            }
        }
    }

    // 3. Check for AC Power
    let on_ac = (|| {
        if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("AD") || name.starts_with("AC") || name.starts_with("ucsi-source-psy") {
                    if let Ok(content) = fs::read_to_string(entry.path().join("online")) {
                        if content.trim() == "1" {
                            return true;
                        }
                    }
                }
            }
        }
        if let Ok(bat_status) = fs::read_to_string("/sys/class/power_supply/BAT0/status") {
            let s = bat_status.trim();
            if !s.is_empty() && s != "Discharging" {
                return true;
            }
        }
        false
    })();

    // 4. Check for active SSH connections
    let has_ssh = (|| {
        if let Ok(out) = Command::new("ss")
            .args(["-H", "-t", "state", "established", "( sport = :22 )"])
            .output()
        {
            if !out.stdout.is_empty() {
                return true;
            }
        }
        let pgrep = Command::new("pgrep")
            .args(["-f", "^sshd: [a-zA-Z0-9]"])
            .output();
        if let Ok(out) = pgrep {
            if out.status.success() {
                return true;
            }
        }
        false
    })();

    if on_ac || has_ssh {
        let _ = Command::new("niri")
            .args(["msg", "action", "power-off-monitors"])
            .output();
        let _ = Command::new("brightnessctl")
            .args(["-d", "*::kbd_backlight", "set", "0"])
            .output();
        return;
    }

    // 5. Battery discharge without inhibitors: suspend
    let _ = Command::new("systemctl").arg("suspend").spawn();
}

fn reload_desktop() {
    println!("[rice-ctl] Reloading Niri rice desktop environment...");

    // 1. Reload Niri config
    let _ = Command::new("niri")
        .args(["msg", "action", "load-config-file"])
        .output();

    // 2. Restart Waybar cleanly
    let _ = Command::new("killall").arg("waybar").output();
    std::thread::sleep(std::time::Duration::from_millis(200));
    let _ = Command::new("waybar").spawn();

    // 3. Reload Wallpaper via awww
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let active_wall = PathBuf::from(&home).join(".config/niri/themes/active-wallpaper.txt");
    if let Ok(wall_path) = fs::read_to_string(active_wall) {
        let trimmed = wall_path.trim();
        if !trimmed.is_empty() && Path::new(trimmed).exists() {
            let _ = Command::new("awww")
                .args([
                    "img",
                    trimmed,
                    "--transition-type",
                    "center",
                    "--transition-pos",
                    "center",
                    "--transition-duration",
                    "1.2",
                    "--transition-fps",
                    "120",
                    "--transition-bezier",
                    ".25,1,.5,1",
                ])
                .output();
        }
    }

    // 4. Reload Mako notifications
    let _ = Command::new("makoctl").arg("reload").output();

    // 5. Trigger smooth screen transition
    let _ = Command::new("niri")
        .args(["msg", "action", "do-screen-transition"])
        .output();

    println!("[rice-ctl] Desktop reloaded successfully.");
}

fn caffeine_toggle() {
    let flag = Path::new("/tmp/caffeine_active");
    if flag.exists() {
        let _ = fs::remove_file(flag);
        let _ = Command::new("pkill")
            .args(["-f", "systemd-inhibit.*caffeine"])
            .output();
        let _ = fs::write("/sys/firmware/acpi/platform_profile", "low-power");
        let _ = Command::new("notify-send")
            .args([
                "-a",
                "Power Manager",
                "-i",
                "preferences-system-power",
                "Sleep Mode: Normal",
                "Auto-sleep enabled, cool profile active",
            ])
            .output();
    } else {
        let _ = fs::File::create(flag);
        let _ = Command::new("systemd-inhibit")
            .args([
                "--what=idle:sleep:handle-lid-switch",
                "--who=Caffeine",
                "--why=User requested no sleep",
                "sleep",
                "infinity",
            ])
            .spawn();
        let _ = fs::write("/sys/firmware/acpi/platform_profile", "performance");
        let _ = Command::new("notify-send")
            .args([
                "-a",
                "Power Manager",
                "-i",
                "caffeine",
                "Sleep Mode: Caffeinated",
                "Sleep & idle disabled (Performance mode)",
            ])
            .output();
    }
    let _ = Command::new("pkill")
        .args(["-RTMIN+13", "waybar"])
        .output();
}

fn caffeine_status() {
    let flag = Path::new("/tmp/caffeine_active");
    if flag.exists() {
        println!(
            r#"{{"text": " caffeinated", "class": "active", "tooltip": "Caffeine Active: Sleep Disabled (TLP AC/Perf)"}}"#
        );
    } else {
        println!(
            r#"{{"text": "󰒲 sleep on", "class": "inactive", "tooltip": "Sleep Enabled: Auto-suspend active (TLP Balanced)"}}"#
        );
    }
}

fn volume_mute_toggle() {
    let lockfile = Path::new("/tmp/audio_mute.lock");
    if lockfile.exists() {
        return;
    }
    let _ = fs::File::create(lockfile);
    let _ = Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
        .output();

    if let Ok(vol) = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
    {
        let text = String::from_utf8_lossy(&vol.stdout);
        let val = if text.contains("[MUTED]") { "1" } else { "0" };
        let _ = Command::new("brightnessctl")
            .args(["--device=platform::mute", "set", val])
            .output();
    }
    std::thread::sleep(std::time::Duration::from_millis(200));
    let _ = fs::remove_file(lockfile);
}

fn mic_mute_toggle() {
    let lockfile = Path::new("/tmp/mic_mute.lock");
    if lockfile.exists() {
        return;
    }
    let _ = fs::File::create(lockfile);
    let _ = Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
        .output();

    if let Ok(vol) = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"])
        .output()
    {
        let text = String::from_utf8_lossy(&vol.stdout);
        let val = if text.contains("[MUTED]") { "1" } else { "0" };
        let _ = Command::new("brightnessctl")
            .args(["--device=platform::micmute", "set", val])
            .output();
    }
    std::thread::sleep(std::time::Duration::from_millis(200));
    let _ = fs::remove_file(lockfile);
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Theme { action } => match action {
            ThemeAction::Select { size } => select_wallpaper(size),
            ThemeAction::Set { file, size } => set_wallpaper(&file, size),
            ThemeAction::Random { dir, size } => random_wallpaper(dir, size),
            ThemeAction::Load { name, size } => load_theme(&name, size),
        },
        Commands::Power { action } => {
            let act = action.unwrap_or(PowerAction::Menu);
            match act {
                PowerAction::Menu => power_menu(),
                PowerAction::Lock => lock_screen(),
                PowerAction::Suspend => {
                    let _ = Command::new("systemctl").arg("suspend").spawn();
                }
                PowerAction::Hibernate => {
                    let _ = Command::new("systemctl").arg("hibernate").spawn();
                }
                PowerAction::Reboot => {
                    let _ = Command::new("systemctl").arg("reboot").spawn();
                }
                PowerAction::Shutdown => {
                    let _ = Command::new("systemctl").arg("poweroff").spawn();
                }
                PowerAction::IdleSuspend => idle_suspend(),
            }
        }
        Commands::Reload => reload_desktop(),
        Commands::Caffeine { action } => match action.as_str() {
            "toggle" => caffeine_toggle(),
            "status" | _ => caffeine_status(),
        },
        Commands::Volume { action } => match action {
            VolumeAction::MuteToggle => volume_mute_toggle(),
            VolumeAction::MicMuteToggle => mic_mute_toggle(),
        },
        Commands::Dnd { action } => {
            if action == "toggle" {
                let _ = Command::new("makoctl").args(["mode", "-t", "dnd"]).output();
            }
        }
        Commands::Media { action } => {
            let _ = Command::new("playerctl").arg(action).output();
        }
    }
}
