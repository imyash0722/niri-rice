use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;

use gdk_pixbuf::Pixbuf;
use gtk4::gdk::Display;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CssProvider, FlowBox, Image, Label, ListBox, ListBoxRow,
    Orientation, Picture, PolicyType, Popover, ScrolledWindow, SelectionMode, Spinner, Stack,
    StackTransitionType, StringList,
};
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, Application, ApplicationWindow, ColorScheme, ComboRow, HeaderBar, NavigationPage,
    NavigationSplitView, PreferencesGroup, PreferencesPage, SpinRow, StyleManager, SwitchRow,
    ToolbarView, WindowTitle,
};
use regex::Regex;

const THUMB_W: i32 = 160;
const THUMB_H: i32 = 100;

fn get_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/pineapple"))
}

fn get_cache_dir() -> PathBuf {
    let dir = get_home().join(".cache/rice-settings");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn get_thumbs_dir() -> PathBuf {
    let dir = get_cache_dir().join("thumbs");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn read_file_str(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn niri_reload() {
    let _ = Command::new("niri")
        .args(["msg", "action", "load-config-file"])
        .output();
}

fn patch_kdl(pattern: &str, replacement: &str) {
    let config_path = get_home().join(".config/niri/config.kdl");
    if let Ok(content) = fs::read_to_string(&config_path) {
        if let Ok(re) = Regex::new(pattern) {
            let patched = re.replace(&content, replacement).to_string();
            if patched != content {
                let _ = fs::write(&config_path, patched);
                niri_reload();
            }
        }
    }
}

fn patch_hypridle(index: usize, seconds: u64) {
    let path = get_home().join(".config/hypr/hypridle.conf");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(re) = Regex::new(r"timeout\s*=\s*\d+") {
            let matches: Vec<_> = re.find_iter(&content).collect();
            if index < matches.len() {
                let m = matches[index];
                let mut new_c = String::with_capacity(content.len() + 8);
                new_c.push_str(&content[..m.start()]);
                new_c.push_str(&format!("timeout = {}", seconds));
                new_c.push_str(&content[m.end()..]);
                let _ = fs::write(&path, new_c);
            }
        }
    }
}

fn patch_mako(key: &str, val: &str) {
    let path = get_home().join(".config/mako/config");
    if let Ok(content) = fs::read_to_string(&path) {
        let pattern = format!(r"(?m)^{}=.+$", regex::escape(key));
        if let Ok(re) = Regex::new(&pattern) {
            let replacement = format!("{}={}", key, val);
            let patched = re.replace(&content, &replacement).to_string();
            if patched != content {
                let _ = fs::write(&path, patched);
                let _ = Command::new("makoctl").arg("reload").output();
            }
        }
    }
}

fn patch_touchpad(option: &str, enabled: bool) {
    let config_path = get_home().join(".config/niri/config.kdl");
    if let Ok(c) = fs::read_to_string(&config_path) {
        if let Ok(re) = Regex::new(r"(?s)(touchpad\s*\{)([^}]*?)(\})") {
            if let Some(caps) = re.captures(&c) {
                let before = &caps[1];
                let block = &caps[2];
                let after = &caps[3];

                let new_block = if enabled {
                    if !block.contains(option) {
                        format!("{}\n        {}\n    ", block.trim_end(), option)
                    } else {
                        block.to_string()
                    }
                } else {
                    Regex::new(&format!(r"\s*\b{}\b", regex::escape(option)))
                        .map(|r| r.replace_all(block, "").to_string())
                        .unwrap_or_else(|_| block.to_string())
                };

                let whole = &caps[0];
                let replaced = format!("{}{}{}", before, new_block, after);
                let final_c = c.replacen(whole, &replaced, 1);
                let _ = fs::write(&config_path, final_c);
                niri_reload();
            }
        }
    }
}

fn patch_numlock(enabled: bool) {
    let config_path = get_home().join(".config/niri/config.kdl");
    if let Ok(c) = fs::read_to_string(&config_path) {
        if let Ok(re) = Regex::new(r"(?s)(keyboard\s*\{)([^}]*?)(\})") {
            if let Some(caps) = re.captures(&c) {
                let before = &caps[1];
                let block = &caps[2];
                let after = &caps[3];

                let new_block = if enabled {
                    if !block.contains("numlock") {
                        format!("{}\n        numlock\n    ", block.trim_end())
                    } else {
                        block.to_string()
                    }
                } else {
                    Regex::new(r"\s*\bnumlock\b")
                        .map(|r| r.replace_all(block, "").to_string())
                        .unwrap_or_else(|_| block.to_string())
                };

                let whole = &caps[0];
                let replaced = format!("{}{}{}", before, new_block, after);
                let final_c = c.replacen(whole, &replaced, 1);
                let _ = fs::write(&config_path, final_c);
                niri_reload();
            }
        }
    }
}

fn patch_border(enabled: bool) {
    let config_path = get_home().join(".config/niri/config.kdl");
    if let Ok(c) = fs::read_to_string(&config_path) {
        if let Ok(re) = Regex::new(r"(?s)(layout\s*\{.*?border\s*\{)([^}]*?)(\})") {
            if let Some(caps) = re.captures(&c) {
                let before = &caps[1];
                let block = &caps[2];
                let after = &caps[3];

                let new_block = if enabled {
                    block.replace("off", "")
                } else if !block.contains("off") {
                    format!("{}\n        off\n    ", block.trim_end())
                } else {
                    block.to_string()
                };

                let whole = &caps[0];
                let replaced = format!("{}{}{}", before, new_block, after);
                let final_c = c.replacen(whole, &replaced, 1);
                let _ = fs::write(&config_path, final_c);
                niri_reload();
            }
        }
    }
}

fn patch_shadow(enabled: bool) {
    let config_path = get_home().join(".config/niri/config.kdl");
    if let Ok(c) = fs::read_to_string(&config_path) {
        let pattern = r"(?s)(layout\s*\{.*?shadow\s*\{[^}]*?)\b(on|off)\b";
        let target = if enabled { "on" } else { "off" };
        if let Ok(re) = Regex::new(pattern) {
            let replaced = re.replace(&c, format!("${{1}}{}", target)).to_string();
            let _ = fs::write(&config_path, replaced);
            niri_reload();
        }
    }
}

fn load_matugen_css() {
    let css_path = get_cache_dir().join("theme.css");
    if css_path.exists() {
        if let Some(display) = Display::default() {
            let provider = CssProvider::new();
            provider.load_from_path(&css_path);
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }
}

fn sync_desktop_colors(wall_path: &str, scheme: &str, mode: &str, contrast: f64) {
    let wall = wall_path.to_string();
    let sch = scheme.to_string();
    let m = mode.to_string();
    thread::spawn(move || {
        let p = Path::new(&wall);
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let is_vid = matches!(ext.as_str(), "mp4" | "mkv" | "webm" | "mov" | "avi" | "gif");

        let img_target = if is_vid {
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("wall");
            let frame = PathBuf::from(format!("/tmp/matugen_frame_{}.jpg", stem));
            if !frame.exists() {
                let _ = Command::new("ffmpeg")
                    .args([
                        "-y",
                        "-ss",
                        "00:00:01",
                        "-i",
                        &wall,
                        "-vframes",
                        "1",
                        "-f",
                        "image2",
                        &frame.to_string_lossy(),
                    ])
                    .output();
            }
            if frame.exists() {
                frame.to_string_lossy().to_string()
            } else {
                wall.clone()
            }
        } else {
            wall.clone()
        };

        let _ = Command::new("matugen")
            .args([
                "image",
                &img_target,
                "-t",
                &sch,
                "-m",
                &m,
                "--contrast",
                &contrast.to_string(),
                "--source-color-index",
                "0",
                "-q",
            ])
            .output();

        glib::idle_add_local_once(|| {
            load_matugen_css();
        });

        let wal_cache = get_home().join(".cache/wal/colors.json");
        let mut primary_color = "#feb877".to_string();
        if let Ok(data) = fs::read_to_string(&wal_cache) {
            if let Ok(cache) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(p) = cache.get("special").and_then(|s| s.get("cursor")).or_else(|| cache.get("colors").and_then(|c| c.get("color5"))).and_then(|v| v.as_str()) {
                    primary_color = p.to_string();
                }
            }
        }
        crate::sync_animation_colors(&primary_color);

        let _ = Command::new("killall").args(["-SIGUSR2", "waybar"]).output();
        let _ = Command::new("makoctl").arg("reload").output();
        let _ = Command::new("pkill").args(["-SIGUSR1", "-x", "kitty"]).output();
        let _ = Command::new("niri")
            .args(["msg", "action", "do-screen-transition"])
            .output();
    });
}

fn get_active_wallpaper() -> String {
    let path = get_home().join(".config/niri/themes/active-wallpaper.txt");
    read_file_str(&path).trim().to_string()
}

fn get_cursor_theme() -> String {
    let path = get_home().join(".config/niri/themes/active-cursor.kdl");
    let c = read_file_str(&path);
    if let Ok(re) = Regex::new(r#"xcursor-theme\s+"([^"]+)""#) {
        if let Some(caps) = re.captures(&c) {
            return caps[1].to_string();
        }
    }
    "Moga-Neon-Sandy".to_string()
}

fn get_cursor_size() -> u32 {
    let path = get_home().join(".config/niri/themes/active-cursor.kdl");
    let c = read_file_str(&path);
    if let Ok(re) = Regex::new(r"xcursor-size\s+(\d+)") {
        if let Some(caps) = re.captures(&c) {
            if let Ok(v) = caps[1].parse::<u32>() {
                return v;
            }
        }
    }
    32
}

fn get_installed_cursors() -> Vec<String> {
    let mut themes = Vec::new();
    for base in [
        PathBuf::from("/usr/share/icons"),
        get_home().join(".local/share/icons"),
    ] {
        if let Ok(entries) = fs::read_dir(base) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() && p.join("cursors").exists() {
                    if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                        themes.push(name.to_string());
                    }
                }
            }
        }
    }
    themes.sort();
    themes.dedup();
    if themes.is_empty() {
        themes.push("Moga-Neon-Sandy".to_string());
    }
    themes
}

fn get_animation_presets() -> Vec<String> {
    let mut presets = Vec::new();
    let dir = get_home().join(".config/niri/animations");
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().map_or(false, |ext| ext == "kdl") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    presets.push(stem.to_string());
                }
            }
        }
    }
    presets.sort();
    presets
}

fn get_active_animation() -> String {
    let path = get_home().join(".config/niri/themes/active-animations.kdl");
    let c = read_file_str(&path);
    if let Ok(re) = Regex::new(r#"animations/([^"]+)\.kdl"#) {
        if let Some(caps) = re.captures(&c) {
            return caps[1].to_string();
        }
    }
    String::new()
}

fn get_cached_or_generate_thumb_path(wall: &Path) -> Option<PathBuf> {
    let thumbs_dir = get_thumbs_dir();
    let meta = fs::metadata(wall).ok()?;
    let mtime = meta
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let hash_key = format!("{:x}", md5_hash(format!("{}_{}_{}", wall.display(), mtime, meta.len()).as_bytes()));
    let thumb_path = thumbs_dir.join(format!("{}.jpg", hash_key));

    if thumb_path.exists() {
        return Some(thumb_path);
    }

    let ext = wall
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let is_vid = matches!(ext.as_str(), "mp4" | "mkv" | "webm" | "mov" | "avi" | "gif");

    if is_vid {
        let res = Command::new("ffmpegthumbnailer")
            .args([
                "-i",
                &wall.to_string_lossy(),
                "-o",
                &thumb_path.to_string_lossy(),
                "-s",
                &THUMB_W.to_string(),
                "-q",
                "8",
            ])
            .output();

        if res.map_or(false, |o| !o.status.success()) {
            let _ = Command::new("ffmpeg")
                .args([
                    "-y",
                    "-ss",
                    "00:00:01",
                    "-i",
                    &wall.to_string_lossy(),
                    "-vframes",
                    "1",
                    "-vf",
                    &format!("scale={}:{}:force_original_aspect_ratio=decrease", THUMB_W, THUMB_H),
                    &thumb_path.to_string_lossy(),
                ])
                .output();
        }
    } else {
        let _ = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                &wall.to_string_lossy(),
                "-vf",
                &format!("scale={}:{}:force_original_aspect_ratio=decrease", THUMB_W, THUMB_H),
                &thumb_path.to_string_lossy(),
            ])
            .output();
    }

    if thumb_path.exists() {
        Some(thumb_path)
    } else {
        None
    }
}

fn md5_hash(data: &[u8]) -> u128 {
    let mut hash: u128 = 0xcbf29ce484222325;
    for b in data {
        hash ^= *b as u128;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// ─── UI Builders ─────────────────────────────────────────────────────────────

fn build_appearance_page() -> PreferencesPage {
    let page = PreferencesPage::builder()
        .title("Appearance")
        .icon_name("preferences-desktop-wallpaper-symbolic")
        .build();

    let wall_grp = PreferencesGroup::builder().title("Wallpaper").build();
    page.add(&wall_grp);

    let active_wall = get_active_wallpaper();
    let cur_name = Path::new(&active_wall)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("(none)");

    let cur_row = ActionRow::builder()
        .title("Active Wallpaper")
        .subtitle(cur_name)
        .build();

    let btn_box = GtkBox::new(Orientation::Horizontal, 6);
    btn_box.set_valign(Align::Center);

    let browse_btn = Button::with_label("Browse Gallery…");
    browse_btn.add_css_class("suggested-action");
    browse_btn.connect_clicked(|_| {
        crate::select_wallpaper(32);
    });

    let rand_btn = Button::with_label("Random");
    rand_btn.connect_clicked(|_| {
        crate::random_wallpaper(None, 32);
    });

    btn_box.append(&browse_btn);
    btn_box.append(&rand_btn);
    cur_row.add_suffix(&btn_box);
    wall_grp.add(&cur_row);

    // FlowBox with async thumbnail loading
    let scroll = ScrolledWindow::builder()
        .min_content_height(240)
        .max_content_height(340)
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .build();

    let flowbox = FlowBox::builder()
        .selection_mode(SelectionMode::None)
        .max_children_per_line(12)
        .min_children_per_line(3)
        .row_spacing(6)
        .column_spacing(6)
        .margin_start(8)
        .margin_end(8)
        .margin_top(6)
        .margin_bottom(6)
        .build();

    let spinner = Spinner::builder()
        .spinning(true)
        .width_request(32)
        .height_request(32)
        .halign(Align::Center)
        .valign(Align::Center)
        .margin_top(20)
        .margin_bottom(20)
        .build();

    let vbox = GtkBox::new(Orientation::Vertical, 0);
    vbox.append(&spinner);
    vbox.append(&flowbox);
    scroll.set_child(Some(&vbox));
    wall_grp.add(&scroll);

    // Asynchronous thumbnail loader
    let (tx, rx) = mpsc::channel::<(PathBuf, Option<PathBuf>, bool)>();
    let wall_dir = get_home().join("Pictures/Wall");

    thread::spawn(move || {
        if let Ok(entries) = fs::read_dir(&wall_dir) {
            let mut walls: Vec<_> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                    matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "mp4" | "mkv")
                })
                .collect();
            walls.sort();

            for w in walls {
                let ext = w.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                let is_vid = matches!(ext.as_str(), "mp4" | "mkv" | "gif");
                let pb_path = get_cached_or_generate_thumb_path(&w);
                if tx.send((w, pb_path, is_vid)).is_err() {
                    break;
                }
            }
        }
    });

    let cur_row_clone = cur_row.clone();
    let flowbox_clone = flowbox.clone();
    let spinner_clone = spinner.clone();
    let active_wall_clone = active_wall.clone();

    glib::idle_add_local(move || {
        let mut count = 0;
        while let Ok((wall, thumb_path, is_vid)) = rx.try_recv() {
            count += 1;
            let btn = Button::builder().build();
            btn.add_css_class("flat");
            btn.set_tooltip_text(wall.file_name().and_then(|s| s.to_str()));

            let pb = thumb_path.as_ref().and_then(|p| Pixbuf::from_file(p).ok());
            if let Some(pix) = pb {
                let pic = Picture::for_pixbuf(&pix);
                pic.set_size_request(THUMB_W, THUMB_H);
                pic.set_can_shrink(true);
                btn.set_child(Some(&pic));
            } else {
                let box_ph = GtkBox::new(Orientation::Vertical, 4);
                box_ph.set_size_request(THUMB_W, THUMB_H);
                box_ph.set_halign(Align::Center);
                box_ph.set_valign(Align::Center);
                let icon = if is_vid { "video-x-generic-symbolic" } else { "image-x-generic-symbolic" };
                let img = Image::from_icon_name(icon);
                img.set_pixel_size(28);
                box_ph.append(&img);
                btn.set_child(Some(&box_ph));
            }

            if wall.to_string_lossy() == active_wall_clone {
                btn.add_css_class("suggested-action");
            }

            let w_clone = wall.clone();
            let cr = cur_row_clone.clone();
            btn.connect_clicked(move |_| {
                crate::set_wallpaper(&w_clone, 32);
                if let Some(name) = w_clone.file_name().and_then(|s| s.to_str()) {
                    cr.set_subtitle(name);
                }
            });

            flowbox_clone.insert(&btn, -1);
            if count >= 8 {
                return glib::ControlFlow::Continue;
            }
        }
        spinner_clone.set_visible(false);
        glib::ControlFlow::Break
    });

    // Preset Themes
    let preset_grp = PreferencesGroup::builder()
        .title("Preset Themes")
        .description("Curated wallpaper + color palette bundles")
        .build();
    page.add(&preset_grp);

    let presets = ["blue", "cyan", "green", "pink", "custom"];
    let preset_list = StringList::new(&presets);
    let preset_combo = ComboRow::builder()
        .title("Color Theme")
        .model(&preset_list)
        .selected(0)
        .build();
    preset_grp.add(&preset_combo);

    let apply_preset_row = ActionRow::builder().build();
    let apply_preset_btn = Button::with_label("Apply Preset");
    apply_preset_btn.add_css_class("suggested-action");
    apply_preset_btn.set_valign(Align::Center);
    let pc = preset_combo.clone();
    apply_preset_btn.connect_clicked(move |_| {
        let idx = pc.selected() as usize;
        if idx < presets.len() {
            crate::load_theme(presets[idx], 32);
        }
    });
    apply_preset_row.add_suffix(&apply_preset_btn);
    preset_grp.add(&apply_preset_row);

    // Matugen Material You Colors
    let mat_grp = PreferencesGroup::builder()
        .title("Material You Colors")
        .description("Regenerate color palette from active wallpaper and broadcast across desktop")
        .build();
    page.add(&mat_grp);

    let schemes = [
        "scheme-tonal-spot",
        "scheme-vibrant",
        "scheme-expressive",
        "scheme-monochrome",
        "scheme-fidelity",
        "scheme-neutral",
        "scheme-rainbow",
        "scheme-content",
    ];
    let scheme_list = StringList::new(&schemes);
    let scheme_combo = ComboRow::builder()
        .title("Scheme Type")
        .model(&scheme_list)
        .selected(0)
        .build();
    mat_grp.add(&scheme_combo);

    let modes = ["Dark", "Light", "Smart (auto)"];
    let mode_list = StringList::new(&modes);
    let mode_combo = ComboRow::builder()
        .title("Mode")
        .model(&mode_list)
        .selected(0)
        .build();
    mat_grp.add(&mode_combo);

    let contrast_row = SpinRow::with_range(-1.0, 1.0, 0.1);
    contrast_row.set_title("Contrast");
    contrast_row.set_subtitle("-1 = soft  ·  0 = standard  ·  1 = high contrast");
    contrast_row.set_digits(1);
    contrast_row.set_value(0.0);
    mat_grp.add(&contrast_row);

    let regen_row = ActionRow::builder().build();
    let regen_btn = Button::with_label("Regenerate & Sync Colors");
    regen_btn.add_css_class("suggested-action");
    regen_btn.set_valign(Align::Center);

    let sc = scheme_combo.clone();
    let mc = mode_combo.clone();
    let cr_spin = contrast_row.clone();
    regen_btn.connect_clicked(move |_| {
        let wall = get_active_wallpaper();
        if !wall.is_empty() {
            let s_idx = sc.selected() as usize;
            let scheme = schemes.get(s_idx).unwrap_or(&"scheme-tonal-spot");
            let mode = match mc.selected() {
                1 => "light",
                2 => "smart",
                _ => "dark",
            };
            let contrast = cr_spin.value();
            sync_desktop_colors(&wall, scheme, mode, contrast);
        }
    });
    regen_row.add_suffix(&regen_btn);
    mat_grp.add(&regen_row);

    // Waybar Style Selector
    let bar_grp = PreferencesGroup::builder()
        .title("Waybar Status Bar")
        .description("Select status bar styling and layout")
        .build();
    page.add(&bar_grp);

    let bar_styles = ["Main (Full Bar with Drawers)", "Smol (Compact Minimal Island)"];
    let bar_list = StringList::new(&bar_styles);
    let bar_combo = ComboRow::builder()
        .title("Bar Style")
        .model(&bar_list)
        .selected(0)
        .build();
    bar_grp.add(&bar_combo);

    bar_combo.connect_selected_notify(|combo| {
        let style = if combo.selected() == 1 { "smol" } else { "main" };
        let cfg = get_home().join(format!(".config/waybar/{}.jsonc", style));
        let css = get_home().join(format!(".config/waybar/{}.css", style));
        let _ = Command::new("killall").arg("waybar").output();
        thread::sleep(std::time::Duration::from_millis(200));
        let _ = Command::new("waybar")
            .args(["-c", &cfg.to_string_lossy(), "-s", &css.to_string_lossy()])
            .spawn();
    });

    // Cursor Settings
    let cur_grp = PreferencesGroup::builder().title("Cursor").build();
    page.add(&cur_grp);

    let cursors = get_installed_cursors();
    let cursor_strs: Vec<&str> = cursors.iter().map(|s| s.as_str()).collect();
    let cursor_list = StringList::new(&cursor_strs);
    let active_cur = get_cursor_theme();
    let sel_idx = cursors.iter().position(|c| c == &active_cur).unwrap_or(0);

    let cur_combo = ComboRow::builder()
        .title("Cursor Theme")
        .model(&cursor_list)
        .selected(sel_idx as u32)
        .build();
    cur_grp.add(&cur_combo);

    let cur_size_spin = SpinRow::with_range(16.0, 64.0, 4.0);
    cur_size_spin.set_title("Cursor Size");
    cur_size_spin.set_subtitle("Pixels (recommended: 24–48)");
    cur_size_spin.set_digits(0);
    cur_size_spin.set_value(get_cursor_size() as f64);
    cur_grp.add(&cur_size_spin);

    let apply_cur_row = ActionRow::builder().build();
    let apply_cur_btn = Button::with_label("Apply Cursor");
    apply_cur_btn.set_valign(Align::Center);
    let cc = cur_combo.clone();
    let cs = cur_size_spin.clone();
    let cursors_clone = cursors.clone();
    apply_cur_btn.connect_clicked(move |_| {
        let idx = cc.selected() as usize;
        if let Some(t) = cursors_clone.get(idx) {
            let size = cs.value() as u32;
            crate::apply_cursor(t, size);
        }
    });
    apply_cur_row.add_suffix(&apply_cur_btn);
    cur_grp.add(&apply_cur_row);

    page
}

fn build_power_page() -> PreferencesPage {
    let page = PreferencesPage::builder()
        .title("Power")
        .icon_name("battery-symbolic")
        .build();

    let prof_grp = PreferencesGroup::builder()
        .title("Power Profile")
        .description("Daoist internal martial arts triad: Nine Yin, Taiji, and Nine Yang")
        .build();
    page.add(&prof_grp);

    let profiles = [
        "❄  Nine Yin (Power Saver)",
        "☯  Taiji (Balanced)",
        "🔥  Nine Yang (Performance)",
    ];
    let prof_list = StringList::new(&profiles);

    let cur_mode = read_file_str(Path::new("/tmp/power_profile_mode")).trim().to_string();
    let init_sel = match cur_mode.as_str() {
        "nine-yin" => 0,
        "nine-yang" => 2,
        _ => 1,
    };

    let prof_combo = ComboRow::builder()
        .title("Profile")
        .model(&prof_list)
        .selected(init_sel)
        .build();
    prof_grp.add(&prof_combo);

    prof_combo.connect_selected_notify(|combo| {
        let target = match combo.selected() {
            0 => "nine-yin",
            2 => "nine-yang",
            _ => "taiji",
        };
        crate::set_power_profile(target);
    });

    let bat_grp = PreferencesGroup::builder()
        .title("Battery")
        .description("Lenovo IdeaPad / ThinkBook battery conservation")
        .build();
    page.add(&bat_grp);

    let cons_mode_path = Path::new("/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode");
    let is_cons = read_file_str(cons_mode_path).trim() == "1";

    let cons_switch = SwitchRow::builder()
        .title("Conservation Mode")
        .subtitle("Limit charging to 60% to maximize battery lifespan on AC power")
        .active(is_cons)
        .build();
    bat_grp.add(&cons_switch);

    cons_switch.connect_active_notify(|_| {
        crate::toggle_conservation_mode();
    });

    // Idle & Sleep
    let idle_grp = PreferencesGroup::builder()
        .title("Idle and Sleep")
        .description("Managed by hypridle — timers activate on system inactivity")
        .build();
    page.add(&idle_grp);

    let hypr_content = read_file_str(&get_home().join(".config/hypr/hypridle.conf"));
    let mut timeouts = vec![180, 300, 480, 900];
    if let Ok(re) = Regex::new(r"timeout\s*=\s*(\d+)") {
        for (i, caps) in re.captures_iter(&hypr_content).enumerate() {
            if i < timeouts.len() {
                if let Ok(val) = caps[1].parse::<u64>() {
                    timeouts[i] = val;
                }
            }
        }
    }

    let dim_spin = SpinRow::with_range(1.0, 60.0, 1.0);
    dim_spin.set_title("Dim Screen");
    dim_spin.set_value((timeouts[0] / 60) as f64);
    idle_grp.add(&dim_spin);

    let lock_spin = SpinRow::with_range(1.0, 60.0, 1.0);
    lock_spin.set_title("Lock Screen");
    lock_spin.set_value((timeouts[1] / 60) as f64);
    idle_grp.add(&lock_spin);

    let mon_spin = SpinRow::with_range(1.0, 120.0, 1.0);
    mon_spin.set_title("Monitors Off");
    mon_spin.set_value((timeouts[2] / 60) as f64);
    idle_grp.add(&mon_spin);

    let susp_spin = SpinRow::with_range(1.0, 180.0, 1.0);
    susp_spin.set_title("Suspend (Battery Only)");
    susp_spin.set_value((timeouts[3] / 60) as f64);
    idle_grp.add(&susp_spin);

    let apply_idle_row = ActionRow::builder().build();
    let apply_idle_btn = Button::with_label("Apply & Restart hypridle");
    apply_idle_btn.add_css_class("suggested-action");
    apply_idle_btn.set_valign(Align::Center);

    let ds = dim_spin.clone();
    let ls = lock_spin.clone();
    let ms = mon_spin.clone();
    let ss = susp_spin.clone();
    apply_idle_btn.connect_clicked(move |_| {
        patch_hypridle(0, ds.value() as u64 * 60);
        patch_hypridle(1, ls.value() as u64 * 60);
        patch_hypridle(2, ms.value() as u64 * 60);
        patch_hypridle(3, ss.value() as u64 * 60);
        let _ = Command::new("pkill").arg("hypridle").output();
        thread::sleep(std::time::Duration::from_millis(300));
        let _ = Command::new("hypridle").spawn();
    });
    apply_idle_row.add_suffix(&apply_idle_btn);
    idle_grp.add(&apply_idle_row);

    // Caffeine
    let caf_grp = PreferencesGroup::builder()
        .title("Caffeine")
        .description("Inhibit sleep and screen locking")
        .build();
    page.add(&caf_grp);

    let is_caff = Path::new("/tmp/caffeine_active").exists();
    let caf_switch = SwitchRow::builder()
        .title("Keep Awake (Caffeine)")
        .subtitle("Block automatic display sleep and system suspend")
        .active(is_caff)
        .build();
    caf_grp.add(&caf_switch);

    caf_switch.connect_active_notify(|_| {
        let flag = Path::new("/tmp/caffeine_active");
        if flag.exists() {
            let _ = fs::remove_file(flag);
            let _ = Command::new("pkill").args(["-f", "systemd-inhibit.*caffeine"]).output();
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
        }
        let _ = Command::new("pkill").args(["-RTMIN+13", "waybar"]).output();
    });

    page
}

fn build_input_page() -> PreferencesPage {
    let page = PreferencesPage::builder()
        .title("Input")
        .icon_name("input-touchpad-symbolic")
        .build();

    let tp_grp = PreferencesGroup::builder()
        .title("Touchpad")
        .description("Changes apply live to the Niri compositor")
        .build();
    page.add(&tp_grp);

    let niri_c = read_file_str(&get_home().join(".config/niri/config.kdl"));
    let tp_re = Regex::new(r"(?s)touchpad\s*\{([^}]*)\}").ok();
    let tp_block = tp_re
        .as_ref()
        .and_then(|r| r.captures(&niri_c))
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    let tap_sw = SwitchRow::builder()
        .title("Tap to Click")
        .active(tp_block.contains("tap"))
        .build();
    tp_grp.add(&tap_sw);
    tap_sw.connect_active_notify(|sw| patch_touchpad("tap", sw.is_active()));

    let ns_sw = SwitchRow::builder()
        .title("Natural Scroll")
        .active(tp_block.contains("natural-scroll"))
        .build();
    tp_grp.add(&ns_sw);
    ns_sw.connect_active_notify(|sw| patch_touchpad("natural-scroll", sw.is_active()));

    let dwt_sw = SwitchRow::builder()
        .title("Disable While Typing (DWT)")
        .subtitle("Suppress touchpad input while typing on keyboard")
        .active(tp_block.contains("dwt"))
        .build();
    tp_grp.add(&dwt_sw);
    dwt_sw.connect_active_notify(|sw| patch_touchpad("dwt", sw.is_active()));

    // Keyboard
    let kb_grp = PreferencesGroup::builder().title("Keyboard").build();
    page.add(&kb_grp);

    let kb_re = Regex::new(r"(?s)keyboard\s*\{([^}]*)\}").ok();
    let kb_block = kb_re
        .as_ref()
        .and_then(|r| r.captures(&niri_c))
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    let nl_sw = SwitchRow::builder()
        .title("NumLock on Startup")
        .active(kb_block.contains("numlock"))
        .build();
    kb_grp.add(&nl_sw);
    nl_sw.connect_active_notify(|sw| patch_numlock(sw.is_active()));

    let nvim_row = ActionRow::builder()
        .title("Edit niri Keybindings")
        .subtitle("Open ~/.config/niri/config.kdl in Neovim")
        .build();
    let nvim_btn = Button::with_label("Open in Neovim");
    nvim_btn.set_valign(Align::Center);
    nvim_btn.connect_clicked(|_| {
        let path = get_home().join(".config/niri/config.kdl");
        let _ = Command::new("kitty")
            .args(["--", "nvim", &path.to_string_lossy()])
            .spawn();
    });
    nvim_row.add_suffix(&nvim_btn);
    kb_grp.add(&nvim_row);

    page
}

fn build_layout_page() -> PreferencesPage {
    let page = PreferencesPage::builder()
        .title("Layout")
        .icon_name("view-grid-symbolic")
        .build();

    let win_grp = PreferencesGroup::builder()
        .title("Window Layout")
        .description("Compositor layout applies live across all columns")
        .build();
    page.add(&win_grp);

    let niri_c = read_file_str(&get_home().join(".config/niri/config.kdl"));
    let cur_gaps = Regex::new(r"\bgaps\s+(\d+)")
        .ok()
        .and_then(|r| r.captures(&niri_c))
        .and_then(|c| c[1].parse::<f64>().ok())
        .unwrap_or(6.0);

    let gaps_spin = SpinRow::with_range(0.0, 60.0, 2.0);
    gaps_spin.set_title("Window Gaps");
    gaps_spin.set_subtitle("Spacing between columns in pixels");
    gaps_spin.set_value(cur_gaps);
    win_grp.add(&gaps_spin);

    gaps_spin.connect_value_notify(|spin| {
        let val = spin.value() as u32;
        patch_kdl(r"(\bgaps\s+)\d+", &format!("${{1}}{}", val));
    });

    let has_shadow = Regex::new(r"(?s)layout\s*\{.*?shadow\s*\{([^}]*)\}")
        .ok()
        .and_then(|r| r.captures(&niri_c))
        .map(|c| c[1].contains("on") && !c[1].contains("off"))
        .unwrap_or(true);

    let shadow_sw = SwitchRow::builder()
        .title("Window Drop Shadows")
        .subtitle("Render soft drop shadows under active and inactive windows")
        .active(has_shadow)
        .build();
    win_grp.add(&shadow_sw);
    shadow_sw.connect_active_notify(|sw| patch_shadow(sw.is_active()));

    let has_border = Regex::new(r"(?s)layout\s*\{.*?border\s*\{([^}]*)\}")
        .ok()
        .and_then(|r| r.captures(&niri_c))
        .map(|c| !c[1].contains("off"))
        .unwrap_or(false);

    let border_sw = SwitchRow::builder()
        .title("Focus Border")
        .subtitle("Render an outline around focused columns")
        .active(has_border)
        .build();
    win_grp.add(&border_sw);
    border_sw.connect_active_notify(|sw| patch_border(sw.is_active()));

    // Blur
    let blur_grp = PreferencesGroup::builder()
        .title("Background Blur")
        .description("Dual-kawase blur iterations for transparent frosted windows")
        .build();
    page.add(&blur_grp);

    let cur_passes = Regex::new(r"(?s)blur\s*\{[^}]*?passes\s+(\d+)")
        .ok()
        .and_then(|r| r.captures(&niri_c))
        .and_then(|c| c[1].parse::<f64>().ok())
        .unwrap_or(4.0);

    let blur_spin = SpinRow::with_range(1.0, 10.0, 1.0);
    blur_spin.set_title("Blur Passes");
    blur_spin.set_value(cur_passes);
    blur_grp.add(&blur_spin);

    blur_spin.connect_value_notify(|spin| {
        let val = spin.value() as u32;
        patch_kdl(r"(blur\s*\{[^}]*?passes\s+)\d+", &format!("${{1}}{}", val));
    });

    // Actions
    let act_grp = PreferencesGroup::builder().title("Display & Desktop").build();
    page.add(&act_grp);

    let scale_row = ActionRow::builder()
        .title("Auto-Scale Displays")
        .subtitle("Apply per-output scaling based on native resolution")
        .build();
    let scale_btn = Button::with_label("Auto Scale");
    scale_btn.set_valign(Align::Center);
    scale_btn.connect_clicked(|_| {
        crate::monitor_autoscale();
    });
    scale_row.add_suffix(&scale_btn);
    act_grp.add(&scale_row);

    let reload_row = ActionRow::builder()
        .title("Reload Desktop")
        .subtitle("Restart Waybar, Mako, and reload niri config")
        .build();
    let reload_btn = Button::with_label("Reload");
    reload_btn.set_valign(Align::Center);
    reload_btn.connect_clicked(|_| {
        crate::reload_desktop();
    });
    reload_row.add_suffix(&reload_btn);
    act_grp.add(&reload_row);

    page
}

fn build_notifications_page() -> PreferencesPage {
    let page = PreferencesPage::builder()
        .title("Notifications")
        .icon_name("notification-symbolic")
        .build();

    let dnd_grp = PreferencesGroup::builder().title("Notification Center").build();
    page.add(&dnd_grp);

    let mako_mode = Command::new("makoctl")
        .arg("mode")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let is_dnd = mako_mode.contains("dnd");

    let dnd_sw = SwitchRow::builder()
        .title("Do Not Disturb")
        .subtitle("Silence all banner popups")
        .active(is_dnd)
        .build();
    dnd_grp.add(&dnd_sw);

    dnd_sw.connect_active_notify(|_| {
        let _ = Command::new("makoctl").args(["mode", "-t", "dnd"]).output();
        let _ = Command::new("pkill").args(["-RTMIN+8", "waybar"]).output();
    });

    let dismiss_row = ActionRow::builder()
        .title("Dismiss All Notifications")
        .build();
    let dismiss_btn = Button::with_label("Dismiss All");
    dismiss_btn.set_valign(Align::Center);
    dismiss_btn.connect_clicked(|_| {
        let _ = Command::new("makoctl").args(["dismiss", "-a"]).output();
    });
    dismiss_row.add_suffix(&dismiss_btn);
    dnd_grp.add(&dismiss_row);

    let hist_row = ActionRow::builder()
        .title("Notification History")
        .subtitle("Open interactive Rofi notification history")
        .build();
    let hist_btn = Button::with_label("Open History");
    hist_btn.set_valign(Align::Center);
    hist_btn.connect_clicked(|_| {
        crate::notification_history_menu();
    });
    hist_row.add_suffix(&hist_btn);
    dnd_grp.add(&hist_row);

    // Mako Styling
    let mako_grp = PreferencesGroup::builder().title("Notification Style").build();
    page.add(&mako_grp);

    let mako_c = read_file_str(&get_home().join(".config/mako/config"));

    let cur_timeout = Regex::new(r"(?m)^default-timeout=(\d+)")
        .ok()
        .and_then(|r| r.captures(&mako_c))
        .and_then(|c| c[1].parse::<f64>().ok())
        .unwrap_or(15000.0)
        / 1000.0;

    let timeout_spin = SpinRow::with_range(1.0, 120.0, 1.0);
    timeout_spin.set_title("Auto-Dismiss Timeout");
    timeout_spin.set_subtitle("Seconds before banner closes");
    timeout_spin.set_value(cur_timeout);
    mako_grp.add(&timeout_spin);

    timeout_spin.connect_value_notify(|spin| {
        let val = (spin.value() as u64) * 1000;
        patch_mako("default-timeout", &val.to_string());
    });

    let cur_w = Regex::new(r"(?m)^width=(\d+)")
        .ok()
        .and_then(|r| r.captures(&mako_c))
        .and_then(|c| c[1].parse::<f64>().ok())
        .unwrap_or(340.0);

    let width_spin = SpinRow::with_range(200.0, 700.0, 10.0);
    width_spin.set_title("Banner Width");
    width_spin.set_value(cur_w);
    mako_grp.add(&width_spin);

    width_spin.connect_value_notify(|spin| {
        patch_mako("width", &(spin.value() as u32).to_string());
    });

    let cur_h = Regex::new(r"(?m)^height=(\d+)")
        .ok()
        .and_then(|r| r.captures(&mako_c))
        .and_then(|c| c[1].parse::<f64>().ok())
        .unwrap_or(120.0);

    let height_spin = SpinRow::with_range(60.0, 400.0, 10.0);
    height_spin.set_title("Max Banner Height");
    height_spin.set_value(cur_h);
    mako_grp.add(&height_spin);

    height_spin.connect_value_notify(|spin| {
        patch_mako("height", &(spin.value() as u32).to_string());
    });

    page
}

fn build_defaults_page() -> PreferencesPage {
    let page = PreferencesPage::builder()
        .title("Default Apps")
        .icon_name("application-x-executable-symbolic")
        .build();

    let defs_grp = PreferencesGroup::builder()
        .title("Default Applications")
        .description("Choose default applications for file types and URL protocols")
        .build();
    page.add(&defs_grp);

    let mime_entries = [
        ("Web Browser", "x-scheme-handler/http"),
        ("File Manager", "inode/directory"),
        ("Text Editor", "text/plain"),
        ("Terminal", "x-scheme-handler/terminal"),
        ("Image Viewer", "image/jpeg"),
        ("PDF Viewer", "application/pdf"),
        ("Video Player", "video/mp4"),
    ];

    let mimeapps_c = read_file_str(&get_home().join(".config/mimeapps.list"))
        + "\n"
        + &read_file_str(&get_home().join(".local/share/applications/mimeapps.list"));

    for (label, mime) in mime_entries {
        let cur_default = Regex::new(&format!(r"(?m)^{}=(.+)$", regex::escape(mime)))
            .ok()
            .and_then(|r| r.captures(&mimeapps_c))
            .map(|c| c[1].trim().to_string())
            .unwrap_or_else(|| "(not set)".to_string());

        let row = ActionRow::builder()
            .title(label)
            .subtitle(&cur_default)
            .build();

        let change_btn = Button::with_label("Change…");
        change_btn.add_css_class("flat");
        change_btn.set_valign(Align::Center);

        let row_c = row.clone();
        let mime_s = mime.to_string();
        let btn_ref = change_btn.clone();

        change_btn.connect_clicked(move |_| {
            let apps = get_apps_for_mime(&mime_s);
            if apps.is_empty() {
                return;
            }

            let popover = Popover::new();
            let list_box = ListBox::new();
            list_box.set_selection_mode(SelectionMode::None);
            list_box.add_css_class("boxed-list");

            let scroll = ScrolledWindow::builder()
                .min_content_height(200)
                .min_content_width(260)
                .hscrollbar_policy(PolicyType::Never)
                .vscrollbar_policy(PolicyType::Automatic)
                .child(&list_box)
                .build();

            popover.set_child(Some(&scroll));

            for (app_name, df_file) in apps {
                let arow = ActionRow::builder()
                    .title(&app_name)
                    .subtitle(&df_file)
                    .activatable(true)
                    .build();

                let df_clone = df_file.clone();
                let m_clone = mime_s.clone();
                let r_clone = row_c.clone();
                let pop_clone = popover.clone();

                arow.connect_activated(move |_| {
                    let _ = Command::new("xdg-mime")
                        .args(["default", &df_clone, &m_clone])
                        .spawn();
                    r_clone.set_subtitle(&df_clone);
                    pop_clone.popdown();
                });

                list_box.append(&arow);
            }

            popover.set_parent(&btn_ref);
            popover.popup();
        });

        row.add_suffix(&change_btn);
        defs_grp.add(&row);
    }

    page
}

fn get_apps_for_mime(mime: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let is_terminal = mime == "x-scheme-handler/terminal";

    for base in [
        PathBuf::from("/usr/share/applications"),
        get_home().join(".local/share/applications"),
    ] {
        if let Ok(entries) = fs::read_dir(base) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().map_or(false, |ext| ext == "desktop") {
                    if let Ok(cnt) = fs::read_to_string(&p) {
                        if cnt.contains("NoDisplay=true") {
                            continue;
                        }
                        let mut matched = false;
                        if is_terminal {
                            if let Some(cat) = Regex::new(r"(?m)^Categories=(.+)$")
                                .ok()
                                .and_then(|r| r.captures(&cnt))
                            {
                                if cat[1].contains("TerminalEmulator") || cat[1].contains("Terminal") {
                                    matched = true;
                                }
                            }
                        } else if let Some(mimes) = Regex::new(r"(?m)^MimeType=(.+)$")
                            .ok()
                            .and_then(|r| r.captures(&cnt))
                        {
                            let parts: Vec<&str> = mimes[1].split(';').map(|s| s.trim()).collect();
                            if parts.contains(&mime)
                                || (mime == "x-scheme-handler/http"
                                    && (parts.contains(&"text/html")
                                        || parts.contains(&"x-scheme-handler/https")))
                            {
                                matched = true;
                            }
                        }

                        if matched {
                            let name = Regex::new(r"(?m)^Name=(.+)$")
                                .ok()
                                .and_then(|r| r.captures(&cnt))
                                .map(|c| c[1].trim().to_string())
                                .unwrap_or_else(|| p.file_stem().unwrap().to_string_lossy().to_string());
                            let df = p.file_name().unwrap().to_string_lossy().to_string();
                            results.push((name, df));
                        }
                    }
                }
            }
        }
    }
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results.dedup_by(|a, b| a.1 == b.1);
    results
}

fn build_animations_page() -> PreferencesPage {
    let page = PreferencesPage::builder()
        .title("Animations")
        .icon_name("media-playback-start-symbolic")
        .build();

    let anim_grp = PreferencesGroup::builder()
        .title("Window Animation Preset")
        .description("Custom GLSL shader animations for window transitions")
        .build();
    page.add(&anim_grp);

    let presets = get_animation_presets();
    let preset_strs: Vec<&str> = presets.iter().map(|s| s.as_str()).collect();
    let preset_list = StringList::new(&preset_strs);
    let cur_anim = get_active_animation();
    let sel_idx = presets.iter().position(|s| s == &cur_anim).unwrap_or(0);

    let anim_combo = ComboRow::builder()
        .title("Animation Preset")
        .model(&preset_list)
        .selected(sel_idx as u32)
        .build();
    anim_grp.add(&anim_combo);

    let apply_row = ActionRow::builder().build();
    let apply_btn = Button::with_label("Apply Animation");
    apply_btn.add_css_class("suggested-action");
    apply_btn.set_valign(Align::Center);

    let ac = anim_combo.clone();
    let presets_clone = presets.clone();
    apply_btn.connect_clicked(move |_| {
        let idx = ac.selected() as usize;
        if let Some(name) = presets_clone.get(idx) {
            crate::set_animation(name);
        }
    });
    apply_row.add_suffix(&apply_btn);
    anim_grp.add(&apply_row);

    let info_grp = PreferencesGroup::builder().title("Animation Directory").build();
    page.add(&info_grp);

    let browse_row = ActionRow::builder()
        .title("Browse Shader Animations")
        .subtitle(get_home().join(".config/niri/animations").to_string_lossy().as_ref())
        .build();
    let browse_btn = Button::with_label("Open Folder");
    browse_btn.set_valign(Align::Center);
    browse_btn.connect_clicked(|_| {
        let p = get_home().join(".config/niri/animations");
        let _ = Command::new("xdg-open").arg(p).spawn();
    });
    browse_row.add_suffix(&browse_btn);
    info_grp.add(&browse_row);

    page
}

// ─── Main Window ─────────────────────────────────────────────────────────────

fn build_ui(app: &Application) {
    StyleManager::default().set_color_scheme(ColorScheme::ForceDark);
    load_matugen_css();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Rice Settings")
        .default_width(980)
        .default_height(720)
        .build();

    let split = NavigationSplitView::new();
    split.set_max_sidebar_width(220.0);
    split.set_min_sidebar_width(170.0);
    split.set_sidebar_width_fraction(0.22);
    window.set_content(Some(&split));

    // Sidebar
    let sidebar_box = GtkBox::new(Orientation::Vertical, 0);
    let sidebar_tb = ToolbarView::new();
    sidebar_tb.set_content(Some(&sidebar_box));

    let sidebar_header = HeaderBar::new();
    sidebar_header.set_show_end_title_buttons(false);
    sidebar_header.set_title_widget(Some(&WindowTitle::new("Settings", "")));
    sidebar_tb.add_top_bar(&sidebar_header);

    let sidebar_nav = NavigationPage::builder()
        .title("Settings")
        .child(&sidebar_tb)
        .build();
    split.set_sidebar(Some(&sidebar_nav));

    let sidebar_list = ListBox::new();
    sidebar_list.add_css_class("navigation-sidebar");
    sidebar_list.set_selection_mode(SelectionMode::Single);
    sidebar_box.append(&sidebar_list);

    // Content Stack
    let stack = Stack::new();
    stack.set_transition_type(StackTransitionType::Crossfade);
    stack.set_transition_duration(120);

    let content_title = WindowTitle::new("Appearance", "");
    let content_header = HeaderBar::new();
    content_header.set_title_widget(Some(&content_title));

    let content_tb = ToolbarView::new();
    content_tb.add_top_bar(&content_header);
    content_tb.set_content(Some(&stack));

    let content_nav = NavigationPage::builder()
        .title("Content")
        .child(&content_tb)
        .build();
    split.set_content(Some(&content_nav));

    let pages: Vec<(&str, &str, PreferencesPage)> = vec![
        ("Appearance", "preferences-desktop-wallpaper-symbolic", build_appearance_page()),
        ("Power", "battery-symbolic", build_power_page()),
        ("Input", "input-touchpad-symbolic", build_input_page()),
        ("Layout", "view-grid-symbolic", build_layout_page()),
        ("Notifications", "notification-symbolic", build_notifications_page()),
        ("Default Apps", "application-x-executable-symbolic", build_defaults_page()),
        ("Animations", "media-playback-start-symbolic", build_animations_page()),
    ];

    for (name, icon, page_widget) in &pages {
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .child(page_widget)
            .build();
        stack.add_named(&scroll, Some(name));

        let row_box = GtkBox::new(Orientation::Horizontal, 12);
        row_box.set_margin_top(10);
        row_box.set_margin_bottom(10);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);

        let img = Image::from_icon_name(*icon);
        img.set_pixel_size(16);
        let lbl = Label::new(Some(name));
        lbl.set_xalign(0.0);
        lbl.set_hexpand(true);

        row_box.append(&img);
        row_box.append(&lbl);

        let lb_row = ListBoxRow::new();
        lb_row.set_child(Some(&row_box));
        sidebar_list.append(&lb_row);
    }

    if let Some(first_row) = sidebar_list.row_at_index(0) {
        sidebar_list.select_row(Some(&first_row));
    }

    let stack_c = stack.clone();
    let title_c = content_title.clone();
    let split_c = split.clone();
    let page_names: Vec<String> = pages.iter().map(|p| p.0.to_string()).collect();

    sidebar_list.connect_row_selected(move |_, row| {
        if let Some(r) = row {
            let idx = r.index() as usize;
            if let Some(name) = page_names.get(idx) {
                stack_c.set_visible_child_name(name);
                title_c.set_title(name);
                split_c.set_show_content(true);
            }
        }
    });

    window.present();
}

pub fn launch() {
    let app = Application::builder()
        .application_id("com.niri.rice.settings")
        .build();

    app.connect_activate(build_ui);
    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);
}
