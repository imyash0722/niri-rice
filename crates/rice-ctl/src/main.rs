use clap::{Parser, Subcommand};
use rand::seq::SliceRandom;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "rice-ctl")]
#[command(
    about = "Unified desktop controller & native theming engine for Niri rice",
    version = "2.0.0"
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
    /// Battery and power conservation management
    Battery {
        #[command(subcommand)]
        action: Option<BatteryAction>,
    },
    /// Display and output management
    Output {
        #[command(subcommand)]
        action: OutputAction,
    },
    /// Interactive Wi-Fi connection manager
    Wifi {
        #[command(subcommand)]
        action: Option<WifiAction>,
    },
    /// Web bookmarks and quick search
    Browse,
    /// Open a URL, hyperlink, or file path with line number
    Open {
        target: String,
    },
    /// Interactive fuzzy link picker for current tmux pane or screen
    LinkPicker,
    /// Calendar notifications
    Calendar {
        #[arg(default_value = "curr")]
        action: String,
    },
    /// Waybar style selector and module formatters
    Bar {
        #[command(subcommand)]
        action: BarAction,
    },
    /// Screenshot capture tool
    Screenshot {
        #[arg(default_value = "area")]
        mode: String,
    },
    /// File search and launcher
    Files,
    /// System memory info
    Mem,
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
    /// Interactive notification history and manager
    History,
    /// Media control wrapper
    Media {
        #[arg(default_value = "play-pause")]
        action: String,
    },
    /// Open quick notes editor
    Notes,
    /// Interactive SSH session launcher
    Ssh,
    /// Focus open terminal or launch main terminal
    Terminal,
    /// System setup and administration tools
    System {
        #[command(subcommand)]
        action: SystemAction,
    },
}

#[derive(Subcommand)]
enum SystemAction {
    /// Interactive fingerprint scanner setup and test utility
    Fingerprint,
    /// Set swapfile hibernation resume and offset
    ResumeOffset,
}

#[derive(Subcommand)]
enum ThemeAction {
    /// Interactive Rofi wallpaper picker
    Select {
        #[arg(short, long, default_value = "32")]
        size: u32,
    },
    /// Apply wallpaper, extract Matugen Material You colors, and sync desktop
    Set {
        file: PathBuf,
        #[arg(short, long, default_value = "32")]
        size: u32,
    },
    /// Pick and apply a random wallpaper from wallpaper directory
    Random {
        #[arg(short, long)]
        dir: Option<PathBuf>,
        #[arg(short, long, default_value = "32")]
        size: u32,
    },
    /// Load a predefined theme from ~/.config/niri/themes
    Load {
        name: String,
        #[arg(short, long, default_value = "32")]
        size: u32,
    },
    /// Set active window animation style
    Animation {
        name: String,
    },
}

#[derive(Subcommand)]
enum PowerAction {
    Menu,
    Lock,
    Suspend,
    Hibernate,
    Reboot,
    Shutdown,
    IdleSuspend,
    Profile,
}

#[derive(Subcommand)]
enum BatteryAction {
    Menu,
    Status,
    ToggleConservation,
    CycleProfile,
}

#[derive(Subcommand)]
enum OutputAction {
    /// Automatically applies per-output scaling based on native resolution
    AutoScale,
}

#[derive(Subcommand)]
enum WifiAction {
    Menu,
}

#[derive(Subcommand)]
enum BarAction {
    Select,
    NukeBack,
    NukePpd,
    NukeVol,
}

#[derive(Subcommand)]
enum VolumeAction {
    MuteToggle,
    MicMuteToggle,
}

#[derive(Deserialize, Debug)]
struct WalCache {
    #[serde(default)]
    special: HashMap<String, String>,
    #[serde(default)]
    colors: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct NiriMode {
    width: u32,
    height: u32,
}

#[derive(Deserialize, Debug)]
struct NiriOutput {
    #[allow(dead_code)]
    name: String,
    modes: Vec<NiriMode>,
    current_mode: Option<usize>,
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

    let niri_themes = PathBuf::from(&home).join(".config/niri/themes");
    let _ = fs::create_dir_all(&niri_themes);
    let cursor_kdl = format!(
        "cursor {{\n    xcursor-theme \"{}\"\n    xcursor-size {}\n}}\n",
        variant, size
    );
    let _ = fs::write(niri_themes.join("active-cursor.kdl"), cursor_kdl);

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

    let re_theme = Regex::new(r"gtk-cursor-theme-name=.*").unwrap();
    let re_size = Regex::new(r"gtk-cursor-theme-size=.*").unwrap();
    for gtk in ["gtk-3.0", "gtk-4.0"] {
        let ini_path = PathBuf::from(&home).join(format!(".config/{}/settings.ini", gtk));
        if let Ok(content) = fs::read_to_string(&ini_path) {
            let updated =
                re_theme.replace_all(&content, format!("gtk-cursor-theme-name={}", variant));
            let updated = re_size.replace_all(&updated, format!("gtk-cursor-theme-size={}", size));
            let _ = fs::write(ini_path, updated.as_bytes());
        }
    }

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
        .or_else(|| cache.special.get("cursor").cloned())
        .or_else(|| cache.colors.get("color5").cloned())
        .unwrap_or_else(|| "#f4b2e2".to_string());

    let niri_config = PathBuf::from(&home).join(".config/niri/config.kdl");
    if let Ok(content) = fs::read_to_string(&niri_config) {
        let re_active = Regex::new(r##"active-color\s+"#[0-9a-fA-F]+""##).unwrap();
        let re_inactive = Regex::new(r##"inactive-color\s+"#[0-9a-fA-F]+""##).unwrap();
        let updated = re_active.replace_all(&content, format!("active-color \"{}\"", primary));
        let updated = re_inactive.replace_all(&updated, format!("inactive-color \"{}\"", bg));
        let _ = fs::write(niri_config, updated.as_bytes());
    }

    let is_waybar_running = Command::new("pgrep")
        .args(["-x", "waybar"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if is_waybar_running {
        let _ = Command::new("killall").args(["-SIGUSR2", "waybar"]).output();
    }

    let _ = Command::new("makoctl").arg("reload").output();
    let _ = Command::new("pkill").args(["-SIGUSR1", "-x", "kitty"]).output();

    if std::env::var("NIRI_SOCKET").is_ok() {
        let _ = Command::new("niri")
            .args(["msg", "action", "do-screen-transition"])
            .output();
    }

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

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let wal_cache = PathBuf::from(&home).join(".cache/wal/colors.json");
    let mut primary_color = "#b3c5ff".to_string();

    if let Ok(data) = fs::read_to_string(&wal_cache) {
        if let Ok(cache) = serde_json::from_str::<WalCache>(&data) {
            if let Some(p) = cache.special.get("cursor").or_else(|| cache.colors.get("color5")) {
                primary_color = p.clone();
            }
        }
    }

    let cursor_variant = find_best_cursor_variant(&primary_color);
    println!(
        "[rice-ctl] Matched cursor: {} ({}px) for accent {}",
        cursor_variant, size, primary_color
    );
    apply_cursor(cursor_variant, size);

    apply_desktop_colors(Some(&primary_color));

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

    let _ = fs::write(
        PathBuf::from(&home).join(".config/wallpaper"),
        canonical.to_string_lossy().as_bytes(),
    );
    let themes_dir = PathBuf::from(&home).join(".config/niri/themes");
    let _ = fs::write(
        themes_dir.join("active-wallpaper.txt"),
        canonical.to_string_lossy().as_bytes(),
    );

    let _ = Command::new("plasma-apply-wallpaperimage")
        .arg(&image_target)
        .output();

    println!("[rice-ctl] Theme and wallpaper switched successfully!");
}

fn select_wallpaper(size: u32) {
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

    let mut rofi_input = Vec::new();
    for c in &concepts {
        let display = c.replace(['_', '-'], " ");
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
            .replace(['_', '-'], " ");
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

fn set_animation(anim: &str) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let anim_file = PathBuf::from(&home).join(format!(".config/niri/animations/{}.kdl", anim));
    if !anim_file.exists() {
        eprintln!("[rice-ctl] Animation '{}' not found.", anim);
        return;
    }
    let content = format!("include \"../animations/{}.kdl\"\n", anim);
    let _ = fs::write(
        PathBuf::from(&home).join(".config/niri/themes/active-animations.kdl"),
        content,
    );
    if std::env::var("NIRI_SOCKET").is_ok() {
        let _ = Command::new("niri")
            .args(["msg", "action", "do-screen-transition"])
            .output();
    }
    println!("[rice-ctl] Animation '{}' applied successfully!", anim);
}

fn lock_screen() {
    let pgrep = Command::new("pgrep")
        .args(["-x", "hyprlock"])
        .output();
    if let Ok(out) = pgrep {
        if out.status.success() {
            return;
        }
    }

    let _ = Command::new("hyprlock")
        .arg("--immediate-render")
        .spawn();
}

fn power_menu() {
    let options = "󰌾 Lock\n󰍃 Logout\n󰤄 Suspend\n󰒲 Hibernate\n󰜉 Reboot\n󰐥 Shutdown\n";
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let theme_path = PathBuf::from(&home).join(".config/rofi/powermenu.rasi");

    let mut child = match Command::new("rofi")
        .args(["-dmenu", "-i", "-p", "Power", "-theme", &theme_path.to_string_lossy()])
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

    if choice.contains("Lock") {
        lock_screen();
    } else if choice.contains("Logout") {
        let _ = Command::new("niri").args(["msg", "action", "quit"]).output();
    } else if choice.contains("Suspend") {
        let _ = Command::new("systemctl").arg("suspend").spawn();
    } else if choice.contains("Hibernate") {
        let _ = Command::new("systemctl").arg("hibernate").spawn();
    } else if choice.contains("Reboot") {
        let _ = Command::new("systemctl").arg("reboot").spawn();
    } else if choice.contains("Shutdown") {
        let _ = Command::new("systemctl").arg("poweroff").spawn();
    }
}

fn idle_suspend() {
    if Path::new("/tmp/caffeine_active").exists() {
        return;
    }

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

    let _ = Command::new("systemctl").arg("suspend").spawn();
}

fn power_profile_menu() {
    let current = fs::read_to_string("/sys/firmware/acpi/platform_profile")
        .unwrap_or_else(|_| "balanced".to_string())
        .trim()
        .to_string();

    let profiles = ["performance", "balanced", "low-power"];
    let mut menu_input = String::new();
    for p in profiles {
        if p == current {
            menu_input.push_str(&format!("{} (Active 🟢)\n", p));
        } else {
            menu_input.push_str(&format!("{} (⭕)\n", p));
        }
    }

    let use_fzf = std::io::stdin().is_terminal();
    let chosen = if use_fzf {
        let mut child = match Command::new("fzf")
            .args(["--ansi", "--prompt=Select Power Profile: ", "--height=10", "--layout=reverse", "--border"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return,
        };
        if let Some(mut sin) = child.stdin.take() {
            let _ = sin.write_all(menu_input.as_bytes());
        }
        let output = match child.wait_with_output() {
            Ok(o) => o,
            Err(_) => return,
        };
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
        let theme_path = PathBuf::from(&home).join(".config/rofi/dmenu.rasi");
        let mut rchild = match Command::new("rofi")
            .args(["-dmenu", "-i", "-p", "󰓅 Power Profile", "-theme", &theme_path.to_string_lossy()])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
        {
            Ok(rc) => rc,
            Err(e) => {
                eprintln!("Failed to spawn selector: {}", e);
                return;
            }
        };
        if let Some(mut sin) = rchild.stdin.take() {
            let _ = sin.write_all(menu_input.as_bytes());
        }
        let output = match rchild.wait_with_output() {
            Ok(o) => o,
            Err(_) => return,
        };
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };

    apply_power_profile_choice(&chosen);
}

fn apply_power_profile_choice(chosen: &str) {
    let raw = chosen.split_whitespace().next().unwrap_or("");
    if raw.is_empty() {
        return;
    }
    println!("Setting power profile to {}...", raw);
    let tee = Command::new("sudo")
        .args(["tee", "/sys/firmware/acpi/platform_profile"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn();
    if let Ok(mut c) = tee {
        if let Some(mut sin) = c.stdin.take() {
            let _ = sin.write_all(raw.as_bytes());
        }
        let _ = c.wait();
    }
    let _ = Command::new("pkill").args(["-RTMIN+2", "waybar"]).output();
}

#[derive(Debug, Deserialize, Clone)]
struct MakoNotificationItem {
    id: u32,
    app_name: Option<String>,
    summary: Option<String>,
    body: Option<String>,
    urgency: Option<String>,
}

fn notification_history_menu() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let theme_path = PathBuf::from(&home).join(".config/rofi/dmenu.rasi");

    let mut items: Vec<MakoNotificationItem> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    // 1. Fetch active notifications
    if let Ok(out) = Command::new("makoctl").args(["list", "-j"]).output() {
        if out.status.success() {
            if let Ok(list) = serde_json::from_slice::<Vec<MakoNotificationItem>>(&out.stdout) {
                for item in list {
                    if seen_ids.insert(item.id) {
                        items.push(item);
                    }
                }
            }
        }
    }

    // 2. Fetch history notifications
    if let Ok(out) = Command::new("makoctl").args(["history", "-j"]).output() {
        if out.status.success() {
            if let Ok(history) = serde_json::from_slice::<Vec<MakoNotificationItem>>(&out.stdout) {
                for item in history {
                    if seen_ids.insert(item.id) {
                        items.push(item);
                    }
                }
            }
        }
    }

    let mut options = String::new();
    if items.is_empty() {
        options.push_str("󰂚  No notification history\n");
    } else {
        options.push_str("󰆴  Clear All Notifications\n");
        for item in &items {
            let app = item.app_name.as_deref().unwrap_or("System");
            let summary = item.summary.as_deref().unwrap_or("").trim();
            let body = item.body.as_deref().unwrap_or("").replace('\n', " ").trim().to_string();
            let urgency_icon = match item.urgency.as_deref() {
                Some("critical") => "󰀦",
                _ => "󰂚",
            };

            let title = if !summary.is_empty() && !body.is_empty() {
                format!("{} [{}] {} - {}", urgency_icon, app, summary, body)
            } else if !summary.is_empty() {
                format!("{} [{}] {}", urgency_icon, app, summary)
            } else if !body.is_empty() {
                format!("{} [{}] {}", urgency_icon, app, body)
            } else {
                format!("{} [{}] Notification #{}", urgency_icon, app, item.id)
            };
            options.push_str(&title);
            options.push('\n');
        }
    }

    let mut child = match Command::new("rofi")
        .args([
            "-dmenu",
            "-i",
            "-format", "i",
            "-p", "󰂚 Notifications",
            "-theme", &theme_path.to_string_lossy(),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error spawning rofi notification history menu: {}", e);
            return;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(options.as_bytes());
    }

    let Ok(output) = child.wait_with_output() else { return };
    if !output.status.success() {
        return;
    }

    let selected_index_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let Ok(idx) = selected_index_str.parse::<usize>() else {
        return;
    };

    if items.is_empty() {
        return;
    }

    if idx == 0 {
        let _ = Command::new("makoctl").args(["dismiss", "-a"]).output();
        let _ = Command::new("systemctl").args(["--user", "restart", "mako"]).output();
        let _ = Command::new("notify-send")
            .args(["-a", "Mako", "Notifications", "All notifications cleared"])
            .spawn();
        return;
    }

    if let Some(selected) = items.get(idx - 1) {
        let _ = Command::new("makoctl").args(["dismiss", "-n", &selected.id.to_string()]).output();

        let full_text = match (&selected.summary, &selected.body) {
            (Some(s), Some(b)) if !s.is_empty() && !b.is_empty() => format!("{}: {}", s, b),
            (Some(s), _) if !s.is_empty() => s.clone(),
            (_, Some(b)) if !b.is_empty() => b.clone(),
            _ => format!("Notification #{}", selected.id),
        };

        if let Ok(mut copy_proc) = Command::new("wl-copy").stdin(std::process::Stdio::piped()).spawn() {
            if let Some(mut cin) = copy_proc.stdin.take() {
                let _ = cin.write_all(full_text.as_bytes());
            }
            let _ = copy_proc.wait();
        }

        let summary_preview = selected.summary.as_deref().unwrap_or("Notification text");
        let _ = Command::new("notify-send")
            .args(["-a", "Mako", "Copied to Clipboard", summary_preview])
            .spawn();
    }
}

fn ssh_menu() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let theme_path = PathBuf::from(&home).join(".config/rofi/dmenu.rasi");
    let _ = Command::new("rofi")
        .args([
            "-show", "ssh",
            "-theme", &theme_path.to_string_lossy(),
            "-terminal", "kitty",
            "-ssh-command", "kitty --class kitty.ssh -T '{host}' ssh {host}",
            "-display-ssh", "󰢹 SSH",
        ])
        .spawn();
}

fn fingerprint_menu() {
    let _ = Command::new("clear").status();
    println!("\x1b[0;34m======================================================\x1b[0m");
    println!("\x1b[0;34m       Fingerprint Scanner Setup & Management         \x1b[0m");
    println!("\x1b[0;34m======================================================\x1b[0m\n");

    let user = std::env::var("USER").unwrap_or_else(|_| "pineapple".to_string());

    println!("\x1b[1;33m[1] Detecting Fingerprint Scanner...\x1b[0m");
    let device_check = Command::new("fprintd-list").arg(&user).output();
    let has_device = match device_check {
        Ok(ref out) => {
            let s = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
            s.contains("found 1 devices") || s.contains("Device")
        }
        Err(_) => false,
    };

    if has_device {
        println!("\x1b[0;32m✓ Scanner Detected: Elan MOC Fingerprint Sensor\x1b[0m\n");
    } else {
        eprintln!("\x1b[0;31m✗ No supported fingerprint scanner detected by fprintd.\x1b[0m");
        return;
    }

    println!("\x1b[1;33m[2] Currently Enrolled Fingers for user '{}':\x1b[0m", user);
    let _ = Command::new("fprintd-list").arg(&user).status();
    println!();

    println!("\x1b[0;34m======================================================\x1b[0m");
    println!("What would you like to do?");
    println!("  1) Enroll Right Index Finger (Recommended)");
    println!("  2) Enroll Right Thumb");
    println!("  3) Enroll Left Index Finger");
    println!("  4) Enroll a custom finger");
    println!("  5) Test / Verify enrolled fingerprint");
    println!("  6) Open Graphical Settings (KDE User Accounts)");
    println!("  7) Delete all enrolled fingers");
    println!("  8) Exit");
    println!("\x1b[0;34m======================================================\x1b[0m");

    print!("Select an option [1-8]: ");
    let _ = std::io::stdout().flush();

    let mut choice = String::new();
    if std::io::stdin().read_line(&mut choice).is_err() {
        return;
    }
    let choice = choice.trim();

    match choice {
        "1" => {
            println!("\n\x1b[0;32mEnrolling Right Index Finger...\x1b[0m");
            println!("Place your right index finger on the power button sensor repeatedly when prompted.");
            let _ = Command::new("fprintd-enroll").args(["-f", "right-index-finger", &user]).status();
        }
        "2" => {
            println!("\n\x1b[0;32mEnrolling Right Thumb...\x1b[0m");
            println!("Place your right thumb on the sensor repeatedly when prompted.");
            let _ = Command::new("fprintd-enroll").args(["-f", "right-thumb", &user]).status();
        }
        "3" => {
            println!("\n\x1b[0;32mEnrolling Left Index Finger...\x1b[0m");
            println!("Place your left index finger on the sensor repeatedly when prompted.");
            let _ = Command::new("fprintd-enroll").args(["-f", "left-index-finger", &user]).status();
        }
        "4" => {
            println!("\nAvailable finger names: right-thumb, right-index-finger, right-middle-finger, right-ring-finger, right-little-finger, left-thumb, left-index-finger, left-middle-finger, left-ring-finger, left-little-finger");
            print!("Enter finger name: ");
            let _ = std::io::stdout().flush();
            let mut custom = String::new();
            if std::io::stdin().read_line(&mut custom).is_ok() {
                let finger = custom.trim();
                let _ = Command::new("fprintd-enroll").args(["-f", finger, &user]).status();
            }
        }
        "5" => {
            println!("\n\x1b[0;34mPlace your enrolled finger on the sensor to test...\x1b[0m");
            let _ = Command::new("fprintd-verify").arg(&user).status();
        }
        "6" => {
            println!("\nOpening KDE User Settings GUI...");
            let _ = Command::new("kcmshell6").arg("kcm_users").spawn();
        }
        "7" => {
            print!("Are you sure you want to delete all fingerprints? [y/N]: ");
            let _ = std::io::stdout().flush();
            let mut confirm = String::new();
            if std::io::stdin().read_line(&mut confirm).is_ok() && confirm.trim().eq_ignore_ascii_case("y") {
                let _ = Command::new("fprintd-delete").arg(&user).status();
                println!("\x1b[0;32mAll fingerprints deleted.\x1b[0m");
            }
        }
        _ => {
            println!("Exiting.");
        }
    }
}

fn setup_hibernate_resume() {
    let swapfile = Path::new("/swap/swapfile");
    if !swapfile.exists() {
        return;
    }

    let df_out = Command::new("df").args(["--output=source", "/swap/swapfile"]).output();
    let dev = match df_out {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout);
            text.lines().last().unwrap_or("").trim().to_string()
        }
        Err(_) => return,
    };

    if dev.is_empty() {
        return;
    }

    let stat_maj = Command::new("stat").args(["-c", "%t", &dev]).output();
    let stat_min = Command::new("stat").args(["-c", "%T", &dev]).output();
    let btrfs_out = Command::new("btrfs").args(["inspect-internal", "map-swapfile", "-r", "/swap/swapfile"]).output();

    if let (Ok(smaj), Ok(smin), Ok(btrfs)) = (stat_maj, stat_min, btrfs_out) {
        let maj_hex = String::from_utf8_lossy(&smaj.stdout).trim().to_string();
        let min_hex = String::from_utf8_lossy(&smin.stdout).trim().to_string();
        let offset = String::from_utf8_lossy(&btrfs.stdout).trim().to_string();

        if let (Ok(maj), Ok(min)) = (i64::from_str_radix(&maj_hex, 16), i64::from_str_radix(&min_hex, 16)) {
            let majmin = format!("{}:{}", maj, min);
            if !majmin.is_empty() && !offset.is_empty() {
                let _ = fs::write("/sys/power/resume", majmin.as_bytes());
                let _ = fs::write("/sys/power/resume_offset", offset.as_bytes());
                println!("Hibernate resume set to {} at offset {}", majmin, offset);
            }
        }
    }
}

fn reload_desktop() {
    println!("[rice-ctl] Reloading Niri rice desktop environment...");

    let _ = Command::new("niri")
        .args(["msg", "action", "load-config-file"])
        .output();

    let _ = Command::new("killall").arg("waybar").output();
    std::thread::sleep(std::time::Duration::from_millis(200));
    let _ = Command::new("waybar").spawn();

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

    let _ = Command::new("makoctl").arg("reload").output();

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

fn monitor_autoscale() {
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());

    for _ in 0..20 {
        if std::env::var("NIRI_SOCKET").is_ok() {
            break;
        }
        if let Ok(entries) = fs::read_dir(&runtime) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("niri") && name.ends_with(".sock") {
                        std::env::set_var("NIRI_SOCKET", p);
                        break;
                    }
                }
            }
        }
        if std::env::var("NIRI_SOCKET").is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    let Ok(output) = Command::new("niri").args(["msg", "-j", "outputs"]).output() else {
        eprintln!("[rice-ctl] Failed to query niri outputs");
        return;
    };

    if let Ok(map) = serde_json::from_slice::<HashMap<String, NiriOutput>>(&output.stdout) {
        for (name, out) in map {
            if let Some(idx) = out.current_mode {
                if let Some(mode) = out.modes.get(idx) {
                    let scale = if mode.height >= 2160 {
                        "1.5"
                    } else if mode.height >= 1440 {
                        "1.25"
                    } else {
                        "1.0"
                    };
                    println!("[rice-ctl] Output {}: {}x{} -> scale {}", name, mode.width, mode.height, scale);
                    let _ = Command::new("niri").args(["msg", "output", &name, "scale", scale]).output();
                }
            }
        }
    }
}

fn get_battery_stats() -> (u32, String, u32, String) {
    let capacity = fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse::<u32>()
        .unwrap_or(0);
    let status = fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .unwrap_or_else(|_| "Unknown".to_string())
        .trim()
        .to_string();
    let conservation = fs::read_to_string("/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode")
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse::<u32>()
        .unwrap_or(0);
    let profile = fs::read_to_string("/sys/firmware/acpi/platform_profile")
        .unwrap_or_else(|_| "balanced".to_string())
        .trim()
        .to_string();
    (capacity, status, conservation, profile)
}

fn toggle_conservation_mode() {
    let path = "/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode";
    let cur = fs::read_to_string(path).unwrap_or_else(|_| "0".to_string()).trim().to_string();
    let next = if cur == "1" { "0" } else { "1" };
    if fs::write(path, next).is_err() {
        let _ = Command::new("sudo").args(["tee", path]).stdin(std::process::Stdio::piped()).spawn().and_then(|mut c| {
            if let Some(mut stdin) = c.stdin.take() {
                let _ = stdin.write_all(next.as_bytes());
            }
            c.wait()
        });
    }
    let msg = if next == "1" { "Conservation Mode: ON (Limit 60%)" } else { "Conservation Mode: OFF (Full 100%)" };
    let _ = Command::new("notify-send").args(["-a", "Battery Manager", "-i", "battery", "ThinkBook Battery", msg]).output();
}

fn cycle_platform_profile() {
    let path = "/sys/firmware/acpi/platform_profile";
    let cur = fs::read_to_string(path).unwrap_or_else(|_| "balanced".to_string()).trim().to_string();
    let next = match cur.as_str() {
        "performance" => "balanced",
        "balanced" => "low-power",
        "low-power" => "performance",
        _ => "balanced",
    };
    if fs::write(path, next).is_err() {
        let _ = Command::new("sudo").args(["tee", path]).stdin(std::process::Stdio::piped()).spawn().and_then(|mut c| {
            if let Some(mut stdin) = c.stdin.take() {
                let _ = stdin.write_all(next.as_bytes());
            }
            c.wait()
        });
    }
    let _ = Command::new("pkill").args(["-RTMIN+2", "waybar"]).output();
    let _ = Command::new("notify-send").args(["-a", "Power Manager", "-i", "preferences-system-power", "Platform Profile", next]).output();
}

fn battery_menu() {
    let (cap, stat, cons, prof) = get_battery_stats();
    let cons_str = if cons == 1 { "ON (60% Cap)" } else { "OFF (100% Charge)" };
    let options = format!(
        "🔋 Battery: {}% ({})\n🛡️ Toggle Conservation Mode [Currently: {}]\n⚡ Cycle Platform Profile [Currently: {}]\n",
        cap, stat, cons_str, prof
    );

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let theme_path = PathBuf::from(&home).join(".config/rofi/dmenu.rasi");

    let mut child = match Command::new("rofi")
        .args(["-dmenu", "-i", "-p", "󰂀 Battery & Power", "-theme", &theme_path.to_string_lossy()])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error spawning rofi battery menu: {}", e);
            return;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(options.as_bytes());
    }

    let Ok(output) = child.wait_with_output() else { return };
    let choice = String::from_utf8_lossy(&output.stdout);

    if choice.contains("Toggle Conservation Mode") {
        toggle_conservation_mode();
    } else if choice.contains("Cycle Platform Profile") {
        cycle_platform_profile();
    }
}

fn wifi_menu() {
    let Ok(out) = Command::new("nmcli").args(["-t", "-f", "IN-USE,SIGNAL,SECURITY,SSID", "dev", "wifi", "list"]).output() else {
        eprintln!("[rice-ctl] Failed to run nmcli");
        return;
    };

    let text = String::from_utf8_lossy(&out.stdout);
    let mut networks: HashMap<String, (bool, u32, String)> = HashMap::new();

    for line in text.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 4 {
            let in_use = parts[0].trim() == "*";
            let signal = parts[1].trim().parse::<u32>().unwrap_or(0);
            let security = parts[2].trim().to_string();
            let ssid = parts[3..].join(":").trim().to_string();
            if !ssid.is_empty() && ssid != "--" {
                if let Some(existing) = networks.get(&ssid) {
                    if signal > existing.1 {
                        networks.insert(ssid.clone(), (in_use || existing.0, signal, security));
                    }
                } else {
                    networks.insert(ssid.clone(), (in_use, signal, security));
                }
            }
        }
    }

    let mut items: Vec<(String, bool, u32, String)> = networks.into_iter().map(|(ssid, (in_use, sig, sec))| (ssid, in_use, sig, sec)).collect();
    items.sort_by_key(|a| std::cmp::Reverse(a.2));

    let mut rofi_lines = String::new();
    rofi_lines.push_str("󰤮  Toggle Wi-Fi (On/Off)\n");
    rofi_lines.push_str("  Network Settings (nmtui)\n");

    for (ssid, in_use, sig, sec) in &items {
        let icon = if *in_use { "󰤨 [Connected]" } else if *sig > 70 { "󰤨" } else if *sig > 40 { "󰤥" } else { "󰤟" };
        let lock = if sec.is_empty() || sec == "--" { "" } else { "🔒" };
        rofi_lines.push_str(&format!("{} {} {} ({}%)\n", icon, ssid, lock, sig));
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let theme_path = PathBuf::from(&home).join(".config/rofi/dmenu.rasi");

    let mut child = match Command::new("rofi")
        .args(["-dmenu", "-i", "-p", "󰤨 Wi-Fi Networks", "-theme", &theme_path.to_string_lossy()])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error spawning rofi wifi: {}", e);
            return;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(rofi_lines.as_bytes());
    }

    let Ok(output) = child.wait_with_output() else { return };
    let choice = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if choice.is_empty() {
        return;
    }

    if choice.contains("Toggle Wi-Fi") {
        let _ = Command::new("nmcli").args(["radio", "wifi"]).output().map(|o| {
            let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
            let next = if s.contains("enabled") { "off" } else { "on" };
            let _ = Command::new("nmcli").args(["radio", "wifi", next]).output();
        });
    } else if choice.contains("Network Settings") {
        let _ = Command::new("kitty").args(["--class", "kitty.nmtui", "nmtui"]).spawn();
    } else {
        for (ssid, in_use, _, sec) in &items {
            if choice.contains(ssid) {
                if *in_use {
                    let _ = Command::new("notify-send").args(["Wi-Fi", &format!("Already connected to {}", ssid)]).output();
                    return;
                }
                if sec.is_empty() || sec == "--" {
                    let _ = Command::new("nmcli").args(["dev", "wifi", "connect", ssid]).spawn();
                } else {
                    if let Ok(pwd_child) = Command::new("rofi").args(["-dmenu", "-password", "-p", &format!("Password for {}", ssid), "-theme", &theme_path.to_string_lossy()]).stdout(std::process::Stdio::piped()).spawn() {
                        if let Ok(pwd_out) = pwd_child.wait_with_output() {
                            let pass = String::from_utf8_lossy(&pwd_out.stdout).trim().to_string();
                            if !pass.is_empty() {
                                let _ = Command::new("nmcli").args(["dev", "wifi", "connect", ssid, "password", &pass]).spawn();
                            }
                        }
                    }
                }
                break;
            }
        }
    }
}

fn browse_menu() {
    let bookmarks = [
        ("GitHub", "https://github.com"),
        ("Wikipedia", "https://en.wikipedia.org/wiki/Main_Page"),
        ("YouTube", "https://www.youtube.com/"),
        ("Arch Wiki", "https://wiki.archlinux.org/title/Main_page"),
        ("DeepSeek", "https://chat.deepseek.com/"),
        ("ChatGPT", "https://chatgpt.com/"),
        ("Claude", "https://claude.ai/new"),
        ("Gemini", "https://gemini.google.com/app"),
    ];

    let mut input = String::new();
    for (name, _) in &bookmarks {
        input.push_str(name);
        input.push('\n');
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let theme_path = PathBuf::from(&home).join(".config/rofi/dmenu.rasi");

    let mut child = match Command::new("rofi")
        .args(["-dmenu", "-p", "󰖟 Browse", "-i", "-theme", &theme_path.to_string_lossy()])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error spawning rofi browse: {}", e);
            return;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }

    let Ok(output) = child.wait_with_output() else { return };
    let choice = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if choice.is_empty() {
        return;
    }

    for (name, url) in &bookmarks {
        if choice.eq_ignore_ascii_case(name) {
            let _ = Command::new("xdg-open").arg(url).spawn();
            return;
        }
    }

    let mut encoded = String::new();
    for b in choice.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            other => {
                encoded.push_str(&format!("%{:02X}", other));
            }
        }
    }
    let url = format!("https://search.brave.com/search?q={}", encoded);
    let _ = Command::new("xdg-open").arg(url).spawn();
}

fn url_decode(s: &str) -> String {
    let mut res = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..=i + 2]).unwrap_or(""), 16) {
                res.push(b as char);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            res.push(' ');
            i += 1;
            continue;
        }
        res.push(bytes[i] as char);
        i += 1;
    }
    res
}

fn extract_line_number(frag: &str) -> Option<u32> {
    let s = frag.trim_start_matches(['L', 'l', ':']);
    let num_str = s.split(['-', ':', ',', '#']).next()?;
    num_str.parse::<u32>().ok()
}

fn open_in_editor(path: &Path, line: Option<u32>) {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    if let Ok(entries) = fs::read_dir(&runtime_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("nvim") {
                let sock = entry.path().to_string_lossy().to_string();
                let cmd = if let Some(l) = line {
                    format!("<Esc>:e +{} {}<CR>", l, path.display())
                } else {
                    format!("<Esc>:e {}<CR>", path.display())
                };
                if let Ok(st) = Command::new("nvim")
                    .args(["--server", &sock, "--remote-send", &cmd])
                    .output()
                {
                    if st.status.success() {
                        return;
                    }
                }
            }
        }
    }

    let has_kate = Command::new("which").arg("kate").output().map(|o| o.status.success()).unwrap_or(false);
    if has_kate {
        let mut cmd = Command::new("kate");
        if let Some(l) = line {
            cmd.args(["-l", &l.to_string()]);
        }
        cmd.arg(path);
        if cmd.spawn().is_ok() {
            return;
        }
    }

    let mut cmd = Command::new("kitty");
    cmd.args(["--class", "kitty.floating"]);
    cmd.arg("nvim");
    if let Some(l) = line {
        cmd.arg(format!("+{}", l));
    }
    cmd.arg(path);
    let _ = cmd.spawn();
}

fn open_target(target: &str) {
    let trimmed = target.trim().trim_matches(|c| c == '\'' || c == '"' || c == '<' || c == '>' || c == '(' || c == ')' || c == '[' || c == ']');
    if trimmed.is_empty() {
        return;
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        let _ = Command::new("xdg-open").arg(trimmed).spawn();
        return;
    }

    if let Some(stripped) = trimmed.strip_prefix("file://") {
        let path_part = if let Some(p) = stripped.strip_prefix("localhost/") {
            p
        } else {
            stripped
        };

        let (raw_path, line_num) = if let Some(idx) = path_part.find('#') {
            let p = &path_part[..idx];
            let frag = &path_part[idx + 1..];
            (p, extract_line_number(frag))
        } else if let Some(idx) = path_part.find(':') {
            let p = &path_part[..idx];
            let rest = &path_part[idx + 1..];
            (p, extract_line_number(rest))
        } else {
            (path_part, None)
        };

        let clean_path = url_decode(raw_path);
        let file_path = PathBuf::from(&clean_path);

        if file_path.exists() {
            if file_path.is_dir() {
                let _ = Command::new("dolphin").arg(&file_path).spawn();
                return;
            }

            if line_num.is_some() {
                open_in_editor(&file_path, line_num);
                return;
            }

            let _ = Command::new("xdg-open").arg(&file_path).spawn();
            return;
        }
    }

    if let Some(idx) = trimmed.find(':') {
        let p_str = &trimmed[..idx];
        let rest = &trimmed[idx + 1..];
        let p = PathBuf::from(p_str);
        if p.exists() {
            let line = extract_line_number(rest);
            open_in_editor(&p, line);
            return;
        }
    }

    let p = PathBuf::from(trimmed);
    if p.exists() {
        if p.is_dir() {
            let _ = Command::new("dolphin").arg(&p).spawn();
            return;
        }
        let _ = Command::new("xdg-open").arg(&p).spawn();
        return;
    }

    let _ = Command::new("xdg-open").arg(trimmed).spawn();
}

fn link_picker() {
    let mut links: Vec<String> = Vec::new();

    if let Ok(out) = Command::new("tmux")
        .args(["capture-pane", "-H", "-p", "-S", "-150"])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        for word in text.split_whitespace() {
            let w = word.trim_matches(|c| c == '\'' || c == '"' || c == '<' || c == '>');
            if (w.starts_with("http://") || w.starts_with("https://") || w.starts_with("file://")) && !links.contains(&w.to_string()) {
                links.push(w.to_string());
            }
        }
    }

    if let Ok(out) = Command::new("tmux")
        .args(["capture-pane", "-J", "-p", "-S", "-150"])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        let re = Regex::new(r#"(https?://[^\s"'`<>]+|file://[^\s"'`<>]+|[a-zA-Z0-9_./-]+\.(rs|ts|js|py|go|c|cpp|h|json|toml|kdl|md|sh|css|html)(:\d+)?)"#).unwrap();
        for cap in re.find_iter(&text) {
            let w = cap.as_str().trim_matches(|c| c == '\'' || c == '"' || c == '<' || c == '>' || c == '(' || c == ')');
            if !w.is_empty() && !links.contains(&w.to_string()) {
                links.push(w.to_string());
            }
        }
    }

    if links.is_empty() {
        let _ = Command::new("notify-send")
            .args(["-u", "low", "-a", "Link Picker", "Link Picker", "No links or file references found in current pane"])
            .output();
        return;
    }

    links.reverse();

    if links.len() == 1 {
        open_target(&links[0]);
        return;
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let theme_path = PathBuf::from(&home).join(".config/rofi/dmenu.rasi");

    let mut input = String::new();
    for l in &links {
        input.push_str(l);
        input.push('\n');
    }

    let mut child = match Command::new("rofi")
        .args(["-dmenu", "-p", "󰌹 Open Link", "-i", "-theme", &theme_path.to_string_lossy()])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }

    let Ok(output) = child.wait_with_output() else { return };
    let choice = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !choice.is_empty() {
        open_target(&choice);
    }
}

fn calendar_action(action: &str) {
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let path = PathBuf::from(runtime).join("calendar_notification_month");
    let cur_diff = fs::read_to_string(&path).unwrap_or_default().trim().parse::<i32>().unwrap_or(0);

    let diff = match action {
        "next" => cur_diff + 1,
        "prev" => cur_diff - 1,
        _ => 0,
    };
    let _ = fs::write(&path, diff.to_string());

    let cal_arg = if diff == 0 {
        String::new()
    } else if diff > 0 {
        format!("+{} months", diff)
    } else {
        format!("{} months ago", -diff)
    };

    let cal_output = if cal_arg.is_empty() {
        Command::new("cal").output()
    } else {
        Command::new("cal").args([&cal_arg]).output()
    };

    if let Ok(out) = cal_output {
        let text = String::from_utf8_lossy(&out.stdout);
        let mut lines: Vec<&str> = text.lines().collect();
        let head = if !lines.is_empty() { lines.remove(0) } else { "Calendar" };
        let body = lines.join("\n");
        let _ = Command::new("notify-send")
            .args([
                "-h", "string:x-canonical-private-synchronous:calendar",
                "-u", "normal",
                "-a", "calendar",
                head,
                &body,
            ])
            .output();
    }
}

fn bar_select() {
    let options = "main\nsmol\nnuke\n";
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let theme_path = PathBuf::from(&home).join(".config/rofi/dmenu.rasi");

    let mut child = match Command::new("rofi")
        .args(["-dmenu", "-p", "󱂬 Waybar Mode", "-theme", &theme_path.to_string_lossy()])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error spawning rofi bar select: {}", e);
            return;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(options.as_bytes());
    }

    let Ok(output) = child.wait_with_output() else { return };
    let choice = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if choice.is_empty() {
        return;
    }

    let _ = Command::new("killall").arg("waybar").output();
    std::thread::sleep(std::time::Duration::from_millis(200));

    let config = format!("{}/.config/waybar/{}.jsonc", home, choice);
    let style = format!("{}/.config/waybar/{}.css", home, choice);

    if Path::new(&config).exists() && Path::new(&style).exists() {
        let _ = Command::new("waybar").args(["-c", &config, "-s", &style]).spawn();
    } else {
        let _ = Command::new("waybar").spawn();
    }
}

fn nuke_back() {
    let get = Command::new("brightnessctl").arg("get").output();
    let max = Command::new("brightnessctl").arg("max").output();
    let cur: u32 = get.ok().and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok()).unwrap_or(0);
    let total: u32 = max.ok().and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok()).unwrap_or(1);
    let percent = cur * 100 / total;
    let filled = (percent * 10 + 50) / 100;
    let empty = 10_u32.saturating_sub(filled);
    let bar: String = "█".repeat(filled as usize) + &"░".repeat(empty as usize);
    println!(" [{}] {}%", bar, percent);
}

fn nuke_ppd() {
    let prof = fs::read_to_string("/sys/firmware/acpi/platform_profile").unwrap_or_default().trim().to_string();
    match prof.as_str() {
        "performance" => println!("!! CRITICAL !!"),
        "balanced" => println!("STABLE"),
        "low-power" => println!("|| FUEL EXHAUSTION ||"),
        _ => println!("STABLE"),
    }
}

fn nuke_vol() {
    let Ok(out) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output() else {
        println!(" [░░░░░░░░░░] 0%");
        return;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let muted = text.contains("[MUTED]");
    let vol_float: f32 = text.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let percent = if muted { 0 } else { (vol_float * 100.0) as u32 };
    let filled = (percent * 10 + 50) / 100;
    let empty = 10_u32.saturating_sub(filled);
    let bar: String = "█".repeat(filled as usize) + &"░".repeat(empty as usize);
    if muted {
        println!(" [{}] MUTED", bar);
    } else {
        println!(" [{}] {}%", bar, percent);
    }
}

fn take_screenshot(mode: &str) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let dir = PathBuf::from(&home).join("Pictures/Screenshots");
    let _ = fs::create_dir_all(&dir);

    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let file = dir.join(format!("screenshot_{}.png", ts));

    match mode {
        "area" => {
            let _ = Command::new("sh").args(["-c", &format!("grim -g \"$(slurp)\" - | tee \"{}\" | wl-copy", file.display())]).output();
        }
        "window" => {
            let _ = Command::new("sh").args(["-c", &format!("grim -g \"$(niri msg -j focused-window | jq -r '\"\\(.x),\\(.y) \\(.width)x\\(.height)\"')\" - | tee \"{}\" | wl-copy", file.display())]).output();
        }
        _ => {
            let _ = Command::new("sh").args(["-c", &format!("grim - | tee \"{}\" | wl-copy", file.display())]).output();
        }
    }

    let _ = Command::new("notify-send")
        .args([
            "-a", "Screenshot",
            "-i", "camera-photo",
            "Screenshot Saved",
            &format!("Saved to {}", file.display()),
        ])
        .output();
}

fn files_picker() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string());
    let theme_path = PathBuf::from(&home).join(".config/rofi/dmenu.rasi");
    let Ok(mut child) = Command::new("fd")
        .args(["--type", "f", "--exclude", "Games"])
        .stdout(std::process::Stdio::piped())
        .spawn() else { return };
    let Some(stdout) = child.stdout.take() else { return };
    if let Ok(rofi) = Command::new("rofi")
        .args(["-dmenu", "-i", "-p", "󰈞 Search Files:", "-theme", &theme_path.to_string_lossy()])
        .stdin(stdout)
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        if let Ok(out) = rofi.wait_with_output() {
            let choice = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !choice.is_empty() {
                let _ = Command::new("xdg-open").arg(choice).spawn();
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct NiriFocusTimestamp {
    secs: u64,
    nanos: u32,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct NiriWindow {
    id: u64,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    app_id: Option<String>,
    #[serde(default)]
    is_focused: bool,
    #[serde(default)]
    focus_timestamp: Option<NiriFocusTimestamp>,
}

fn is_terminal_window(w: &NiriWindow) -> bool {
    let Some(ref app_id) = w.app_id else { return false };
    matches!(app_id.as_str(), "kitty" | "kitty.floating")
}

fn spawn_main_terminal() {
    let _ = Command::new("kitty")
        .args(["tmux", "new-session", "-A", "-s", "main"])
        .spawn();
}

fn focus_or_spawn_terminal() {
    let output = Command::new("niri")
        .args(["msg", "-j", "windows"])
        .output();

    let Ok(out) = output else {
        spawn_main_terminal();
        return;
    };

    let Ok(windows): Result<Vec<NiriWindow>, _> = serde_json::from_slice(&out.stdout) else {
        spawn_main_terminal();
        return;
    };

    let mut term_windows: Vec<&NiriWindow> = windows
        .iter()
        .filter(|w| is_terminal_window(w))
        .collect();

    if term_windows.is_empty() {
        spawn_main_terminal();
        return;
    }

    let focused_idx = term_windows.iter().position(|w| w.is_focused);

    match focused_idx {
        Some(idx) => {
            if term_windows.len() > 1 {
                let next_idx = (idx + 1) % term_windows.len();
                let next_id = term_windows[next_idx].id;
                let _ = Command::new("niri")
                    .args(["msg", "action", "focus-window", "--id", &next_id.to_string()])
                    .spawn();
            }
        }
        None => {
            term_windows.sort_by(|a, b| {
                let ts_a = a.focus_timestamp.as_ref().map(|t| (t.secs, t.nanos)).unwrap_or((0, 0));
                let ts_b = b.focus_timestamp.as_ref().map(|t| (t.secs, t.nanos)).unwrap_or((0, 0));
                ts_b.cmp(&ts_a)
            });

            let target_id = term_windows[0].id;
            let _ = Command::new("niri")
                .args(["msg", "action", "focus-window", "--id", &target_id.to_string()])
                .spawn();
        }
    }
}

fn mem_info() {
    let _ = Command::new("notify-send")
        .args([
            "-u", "normal",
            "-a", "System Status",
            "-i", "dialog-information",
            "Memory Info",
            &fs::read_to_string("/proc/meminfo").unwrap_or_default(),
        ])
        .output();
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

fn dirs_home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/home/pineapple".to_string()))
}

fn main() {
    // Multicall support (Busybox style) for backward-compatible symlinks
    let args: Vec<String> = std::env::args().collect();
    let invoked_as = Path::new(&args[0])
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("rice-ctl");

    match invoked_as {
        "wall.sh" | "wall" => {
            select_wallpaper(32);
            return;
        }
        "rng.sh" | "rng" => {
            random_wallpaper(None, 32);
            return;
        }
        "caf.sh" | "caffeine" => {
            let action = args.get(1).map(|s| s.as_str()).unwrap_or("toggle");
            match action {
                "toggle" => caffeine_toggle(),
                _ => caffeine_status(),
            }
            return;
        }
        "cal.sh" => {
            let action = args.get(1).map(|s| s.as_str()).unwrap_or("curr");
            calendar_action(action);
            return;
        }
        "battery-rofi" | "battery.sh" | "battery" => {
            let action = args.get(1).map(|s| s.as_str()).unwrap_or("menu");
            match action {
                "status" | "--status" => {
                    let (cap, stat, cons, prof) = get_battery_stats();
                    println!("Battery: {}% ({}) | Conservation: {} | Profile: {}", cap, stat, cons, prof);
                }
                "toggle" | "--toggle" => toggle_conservation_mode(),
                "cycle" | "--profile" => cycle_platform_profile(),
                _ => battery_menu(),
            }
            return;
        }
        "wifi.sh" | "wifi" => {
            wifi_menu();
            return;
        }
        "browse.sh" | "browse" => {
            browse_menu();
            return;
        }
        "powermenu.sh" | "power.sh" => {
            power_menu();
            return;
        }
        "lock.sh" => {
            lock_screen();
            return;
        }
        "monitor-setup.sh" => {
            monitor_autoscale();
            return;
        }
        "reload.sh" => {
            reload_desktop();
            return;
        }
        "barsel.sh" => {
            bar_select();
            return;
        }
        "ss.sh" => {
            let mode = args.get(1).map(|s| s.as_str()).unwrap_or("area");
            take_screenshot(mode);
            return;
        }
        "note.sh" | "yazi-note.sh" => {
            let _ = Command::new("kitty")
                .args(["--class", "kitty.floating.notes", "nvim"])
                .current_dir(dirs_home().join("Notes"))
                .spawn();
            return;
        }
        "filerofi.sh" => {
            files_picker();
            return;
        }
        "open" => {
            if let Some(target) = args.get(1) {
                open_target(target);
            }
            return;
        }
        "link-picker" | "open-pane-links" => {
            link_picker();
            return;
        }
        "mem.sh" => {
            mem_info();
            return;
        }
        "dnd-toggle.sh" => {
            let _ = Command::new("makoctl").args(["mode", "-t", "dnd"]).output();
            return;
        }
        "mute-debounce" => {
            volume_mute_toggle();
            return;
        }
        "mic-debounce" => {
            mic_mute_toggle();
            return;
        }
        "set-animation.sh" => {
            if let Some(anim) = args.get(1) {
                set_animation(anim);
            }
            return;
        }
        "set-theme.sh" => {
            if let Some(t) = args.get(1) {
                load_theme(t, 32);
            }
            return;
        }
        "apply-wallpaper.sh" => {
            if let Some(f) = args.get(1) {
                set_wallpaper(Path::new(f), 32);
            }
            return;
        }
        "power-manager" => {
            power_profile_menu();
            return;
        }
        "rofi-ssh" => {
            ssh_menu();
            return;
        }
        "fingerprint-setup" => {
            fingerprint_menu();
            return;
        }
        "setup-hibernate-resume.sh" | "setup-hibernate-resume" => {
            setup_hibernate_resume();
            return;
        }
        "idle-suspend.sh" => {
            idle_suspend();
            return;
        }
        _ => {}
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Theme { action } => match action {
            ThemeAction::Select { size } => select_wallpaper(size),
            ThemeAction::Set { file, size } => set_wallpaper(&file, size),
            ThemeAction::Random { dir, size } => random_wallpaper(dir, size),
            ThemeAction::Load { name, size } => load_theme(&name, size),
            ThemeAction::Animation { name } => set_animation(&name),
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
                PowerAction::Profile => power_profile_menu(),
            }
        }
        Commands::Battery { action } => {
            let act = action.unwrap_or(BatteryAction::Menu);
            match act {
                BatteryAction::Menu => battery_menu(),
                BatteryAction::Status => {
                    let (cap, stat, cons, prof) = get_battery_stats();
                    println!("Battery: {}% ({}) | Conservation: {} | Profile: {}", cap, stat, cons, prof);
                }
                BatteryAction::ToggleConservation => toggle_conservation_mode(),
                BatteryAction::CycleProfile => cycle_platform_profile(),
            }
        }
        Commands::Output { action } => match action {
            OutputAction::AutoScale => monitor_autoscale(),
        },
        Commands::Wifi { action } => {
            let _ = action;
            wifi_menu();
        }
        Commands::Browse => browse_menu(),
        Commands::Calendar { action } => calendar_action(&action),
        Commands::Bar { action } => match action {
            BarAction::Select => bar_select(),
            BarAction::NukeBack => nuke_back(),
            BarAction::NukePpd => nuke_ppd(),
            BarAction::NukeVol => nuke_vol(),
        },
        Commands::Screenshot { mode } => take_screenshot(&mode),
        Commands::Files => files_picker(),
        Commands::Mem => mem_info(),
        Commands::Reload => reload_desktop(),
        Commands::Caffeine { action } => match action.as_str() {
            "toggle" => caffeine_toggle(),
            _ => caffeine_status(),
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
        Commands::History => notification_history_menu(),
        Commands::Media { action } => {
            let _ = Command::new("playerctl").arg(action).output();
        }
        Commands::Notes => {
            let _ = Command::new("kitty")
                .args(["--class", "kitty.floating.notes", "nvim"])
                .current_dir(dirs_home().join("Notes"))
                .spawn();
        }
        Commands::Open { target } => open_target(&target),
        Commands::LinkPicker => link_picker(),
        Commands::Ssh => ssh_menu(),
        Commands::Terminal => focus_or_spawn_terminal(),
        Commands::System { action } => match action {
            SystemAction::Fingerprint => fingerprint_menu(),
            SystemAction::ResumeOffset => setup_hibernate_resume(),
        },
    }
}
