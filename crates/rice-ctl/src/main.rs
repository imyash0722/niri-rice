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
#[command(about = "Unified desktop controller & native theming engine for Niri rice", version = "0.1.0")]
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
    /// Desktop power actions
    Power {
        #[command(subcommand)]
        action: PowerAction,
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
}

#[derive(Subcommand)]
enum PowerAction {
    /// Lock the screen
    Lock,
    /// Suspend to RAM
    Suspend,
    /// Hibernate to disk
    Hibernate,
    /// Reboot system
    Reboot,
    /// Power off system
    Shutdown,
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

#[allow(dead_code)]
fn expand_home(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
        PathBuf::from(home).join(stripped)
    } else {
        PathBuf::from(path)
    }
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
        .args(["--file", "kcminputrc", "--group", "Mouse", "--key", "cursorTheme", variant])
        .output();
    let _ = Command::new("kwriteconfig6")
        .args(["--file", "kcminputrc", "--group", "Mouse", "--key", "cursorSize", &size.to_string()])
        .output();

    // 3. GTK 3 & 4 settings.ini and gsettings
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "cursor-theme", variant])
        .output();
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "cursor-size", &size.to_string()])
        .output();

    for gtk in ["gtk-3.0", "gtk-4.0"] {
        let ini_path = PathBuf::from(&home).join(format!(".config/{}/settings.ini", gtk));
        if let Ok(content) = fs::read_to_string(&ini_path) {
            let re_theme = Regex::new(r"gtk-cursor-theme-name=.*").unwrap();
            let re_size = Regex::new(r"gtk-cursor-theme-size=.*").unwrap();
            let updated = re_theme.replace_all(&content, format!("gtk-cursor-theme-name={}", variant));
            let updated = re_size.replace_all(&updated, format!("gtk-cursor-theme-size={}", size));
            let _ = fs::write(ini_path, updated.as_bytes());
        }
    }

    // 4. X11 & Environment
    let icons_dir = PathBuf::from(&home).join(".icons/default");
    let _ = fs::create_dir_all(&icons_dir);
    let _ = fs::write(
        icons_dir.join("index.theme"),
        format!("[Icon Theme]\nName=Default\nComment=Default Cursor Theme\nInherits={}\n", variant),
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

    let Ok(data) = fs::read_to_string(&wal_cache) else { return };
    let Ok(cache) = serde_json::from_str::<WalCache>(&data) else { return };

    let bg = cache.special.get("background").cloned().unwrap_or_else(|| "#121318".to_string());
    let fg = cache.special.get("foreground").cloned().unwrap_or_else(|| "#e3e2e9".to_string());
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
    let is_waybar_running = Command::new("pgrep").args(["-x", "waybar"]).output().map(|o| o.status.success()).unwrap_or(false);
    if is_waybar_running {
        let _ = Command::new("killall").args(["-SIGUSR2", "waybar"]).output();
    }

    // 3. Reload Mako notifications
    let _ = Command::new("makoctl").arg("reload").output();

    // 4. Trigger Niri smooth screen transition
    if std::env::var("NIRI_SOCKET").is_ok() {
        let _ = Command::new("niri").args(["msg", "action", "do-screen-transition"]).output();
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
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc_getuid() }));
    if let Ok(entries) = fs::read_dir(&runtime_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("nvim") {
                let sock = entry.path().to_string_lossy().to_string();
                let _ = Command::new("nvim")
                    .args(["--server", &sock, "--remote-send", "<Esc>:colorscheme neopywal<CR>"])
                    .output();
            }
        }
    }

    // 7. Sync D-Bus environment
    let _ = Command::new("dbus-update-activation-environment")
        .args(["--systemd", "--all"])
        .output();
}

unsafe fn libc_getuid() -> u32 {
    1000
}

fn set_wallpaper(file: &Path, size: u32) {
    let canonical = fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf());
    if !canonical.exists() {
        eprintln!("Error: Wallpaper file '{}' does not exist.", canonical.display());
        return;
    }

    println!("[rice-ctl] Setting wallpaper: {}", canonical.display());

    // 1. Resolve preview image for Matugen (if video/gif, extract frame)
    let ext = canonical.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let is_video = matches!(ext.as_str(), "mp4" | "mkv" | "webm" | "mov" | "avi" | "gif");
    let image_target = if is_video {
        let parent = canonical.parent().unwrap_or_else(|| Path::new("/tmp"));
        let stem = canonical.file_stem().and_then(|s| s.to_str()).unwrap_or("wall");
        let png_sibling = parent.join(format!("{}.png", stem));
        if png_sibling.exists() {
            png_sibling
        } else {
            let tmp_frame = PathBuf::from(format!("/tmp/matugen_frame_{}.jpg", stem));
            let _ = Command::new("ffmpeg")
                .args(["-y", "-ss", "00:00:01", "-i", &canonical.to_string_lossy(), "-vframes", "1", "-f", "image2", &tmp_frame.to_string_lossy()])
                .output();
            tmp_frame
        }
    } else {
        canonical.clone()
    };

    // 2. Run Matugen in Rust
    println!("[rice-ctl] Extracting Material You palette via Matugen...");
    let status = Command::new("matugen")
        .args(["image", &image_target.to_string_lossy(), "--source-color-index", "0", "-q"])
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
    println!("[rice-ctl] Matched cursor: {} ({}px) for accent {}", cursor_variant, size, primary_color);
    apply_cursor(cursor_variant, size);

    // 5. Update desktop colors (Niri, Waybar, Mako, Terminals, Neovim)
    apply_desktop_colors(Some(&primary_color));

    // 6. Apply live wallpaper via awww
    println!("[rice-ctl] Applying live wallpaper via awww...");
    let is_awww_running = Command::new("pgrep").args(["-x", "awww-daemon"]).output().map(|o| o.status.success()).unwrap_or(false);
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
    let _ = fs::write(PathBuf::from(&home).join(".config/wallpaper"), canonical.to_string_lossy().as_bytes());
    let themes_dir = PathBuf::from(&home).join(".config/niri/themes");
    let _ = fs::write(themes_dir.join("active-wallpaper.txt"), canonical.to_string_lossy().as_bytes());

    // 8. Sync KDE plasma wallpaper
    let _ = Command::new("plasma-apply-wallpaperimage")
        .arg(&image_target)
        .output();

    println!("[rice-ctl] Theme and wallpaper switched successfully!");
}

fn random_wallpaper(dir: Option<PathBuf>, size: u32) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let wall_dir = dir.unwrap_or_else(|| PathBuf::from(&home).join("Pictures/Wall"));

    if !wall_dir.exists() {
        eprintln!("Error: Wallpaper directory '{}' does not exist.", wall_dir.display());
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
                if matches!(ext_lower.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "mp4") {
                    valid_files.push(path);
                }
            }
        }
    }

    if valid_files.is_empty() {
        eprintln!("Error: No supported wallpaper images found in '{}'", wall_dir.display());
        return;
    }

    let mut rng = rand::thread_rng();
    if let Some(chosen) = valid_files.choose(&mut rng) {
        set_wallpaper(chosen, size);
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Theme { action } => match action {
            ThemeAction::Set { file, size } => set_wallpaper(&file, size),
            ThemeAction::Random { dir, size } => random_wallpaper(dir, size),
        },
        Commands::Power { action } => match action {
            PowerAction::Lock => {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
                let wall = fs::read_to_string(PathBuf::from(&home).join(".config/wallpaper")).unwrap_or_default();
                let _ = Command::new("swaylock")
                    .args(["-f", "-i", wall.trim(), "--effect-vignette", "0.4:0.4", "--effect-blur", "32x3", "--clock", "--indicator"])
                    .spawn();
            }
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
