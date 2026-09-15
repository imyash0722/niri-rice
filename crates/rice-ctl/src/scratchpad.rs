use niri_ipc::socket::Socket;
use niri_ipc::{
    Action, PositionChange, Request, Response, SizeChange, Window, WorkspaceReferenceArg,
};
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum ScratchpadDirection {
    FromTop,
    FromBottom,
    FromLeft,
    FromRight,
}

pub struct ScratchpadProfile {
    pub name: &'static str,
    pub app_id_pattern: &'static str,
    pub command: &'static str,
    pub direction: ScratchpadDirection,
    pub width_ratio: f64,
    pub height_ratio: f64,
    pub margin: f64,
}

const PROFILES: &[ScratchpadProfile] = &[
    ScratchpadProfile {
        name: "term",
        app_id_pattern: "float.dropterm",
        command: "wezterm start --class float.dropterm -- tmux new-session -A -s main",
        direction: ScratchpadDirection::FromTop,
        width_ratio: 0.65,
        height_ratio: 0.55,
        margin: 24.0,
    },
    ScratchpadProfile {
        name: "calc",
        app_id_pattern: "kcalc",
        command: "kcalc",
        direction: ScratchpadDirection::FromLeft,
        width_ratio: 0.20,
        height_ratio: 0.38,
        margin: 24.0,
    },
    ScratchpadProfile {
        name: "notes",
        app_id_pattern: "float.notes",
        command: "wezterm start --class float.notes -- nvim ~/notes.md",
        direction: ScratchpadDirection::FromRight,
        width_ratio: 0.45,
        height_ratio: 0.75,
        margin: 24.0,
    },
];

fn get_profile(name: &str) -> Option<&'static ScratchpadProfile> {
    PROFILES
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case(name) || p.app_id_pattern.contains(name))
}

/// Calculate on-screen (target_w, target_h, show_x, show_y)
fn calculate_show_geometry(
    profile: &ScratchpadProfile,
    screen_w: f64,
    screen_h: f64,
) -> (i32, i32, f64, f64) {
    let target_w = (screen_w * profile.width_ratio).round() as i32;
    let target_h = (screen_h * profile.height_ratio).round() as i32;
    let w = target_w as f64;
    let h = target_h as f64;
    let m = profile.margin;

    let (x, y) = match profile.direction {
        ScratchpadDirection::FromTop => ((screen_w - w) / 2.0, m),
        ScratchpadDirection::FromBottom => ((screen_w - w) / 2.0, screen_h - h - m),
        ScratchpadDirection::FromLeft => (m, (screen_h - h) / 2.0),
        ScratchpadDirection::FromRight => (screen_w - w - m, (screen_h - h) / 2.0),
    };

    (target_w, target_h, x, y)
}

/// Calculate off-screen hide position (hide_x, hide_y)
fn calculate_hide_geometry(
    profile: &ScratchpadProfile,
    screen_w: f64,
    screen_h: f64,
    target_w: i32,
    target_h: i32,
    show_x: f64,
    show_y: f64,
) -> (f64, f64) {
    let w = target_w as f64;
    let h = target_h as f64;
    let m = profile.margin;

    match profile.direction {
        ScratchpadDirection::FromTop => (show_x, -(h + m * 2.0)),
        ScratchpadDirection::FromBottom => (show_x, screen_h + m * 2.0),
        ScratchpadDirection::FromLeft => (-(w + m * 2.0), show_y),
        ScratchpadDirection::FromRight => (screen_w + m * 2.0, show_y),
    }
}

/// Helper to track fullscreen window un-fullscreened by scratchpad
fn fs_state_file(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("rice_scratchpad_fs_{name}"))
}

fn save_fs_state(name: &str, win_id: u64) {
    let _ = std::fs::write(fs_state_file(name), win_id.to_string());
}

fn pop_fs_state(name: &str) -> Option<u64> {
    let path = fs_state_file(name);
    let id_str = std::fs::read_to_string(&path).ok()?;
    let _ = std::fs::remove_file(path);
    id_str.trim().parse().ok()
}

/// Query focused output dimensions (width, height)
fn get_screen_dimensions(socket: &mut Socket) -> (f64, f64) {
    if let Ok(Ok(Response::FocusedOutput(Some(output)))) = socket.send(Request::FocusedOutput) {
        if let Some(logical) = output.logical {
            return (logical.width as f64, logical.height as f64);
        }
    }
    (1920.0, 1080.0)
}

/// Check if window is currently visible on screen
fn is_window_on_screen(window: &Window, profile: &ScratchpadProfile, screen_w: f64, screen_h: f64) -> bool {
    if let Some((x, y)) = window.layout.tile_pos_in_workspace_view {
        match profile.direction {
            ScratchpadDirection::FromTop => y >= 0.0 && y < screen_h,
            ScratchpadDirection::FromBottom => y < screen_h && (y + window.layout.tile_size.1) > 0.0,
            ScratchpadDirection::FromLeft => x >= 0.0 && x < screen_w,
            ScratchpadDirection::FromRight => x < screen_w && (x + window.layout.tile_size.0) > 0.0,
        }
    } else {
        false
    }
}

/// Find window matching profile
fn find_window(windows: &[Window], pattern: &str) -> Option<Window> {
    windows.iter().find(|w| {
        let app = w.app_id.as_deref().unwrap_or("").to_lowercase();
        let title = w.title.as_deref().unwrap_or("").to_lowercase();
        let pat = pattern.to_lowercase();
        app.contains(&pat) || title.contains(&pat)
    }).cloned()
}

pub fn toggle_scratchpad(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let profile = get_profile(name)
        .ok_or_else(|| format!("Unknown scratchpad profile: {name}. Available: term, calc, notes"))?;

    let mut socket = Socket::connect()?;
    let (screen_w, screen_h) = get_screen_dimensions(&mut socket);
    let (target_w, target_h, show_x, show_y) = calculate_show_geometry(profile, screen_w, screen_h);
    let (hide_x, hide_y) = calculate_hide_geometry(profile, screen_w, screen_h, target_w, target_h, show_x, show_y);

    let windows_reply = socket.send(Request::Windows)?;
    let windows = match windows_reply {
        Ok(Response::Windows(w)) => w,
        _ => return Err("Failed to query open windows".into()),
    };

    // Query active workspaces to find current focused workspace
    let current_ws = if let Ok(Ok(Response::Workspaces(ws))) = socket.send(Request::Workspaces) {
        ws.iter().find(|w| w.is_focused).map(|w| w.id)
    } else {
        None
    };

    if let Some(window) = find_window(&windows, profile.app_id_pattern) {
        let wid = window.id;
        let on_screen = is_window_on_screen(&window, profile, screen_w, screen_h);
        let in_current_workspace = current_ws.is_some() && window.workspace_id == current_ws;

        // If on screen and in current workspace: hide it!
        if on_screen && in_current_workspace {
            println!("[scratchpad] Sliding off-screen (hiding) '{}' (id: {})", profile.name, wid);
            let _ = socket.send(Request::Action(Action::MoveFloatingWindow {
                id: Some(wid),
                x: PositionChange::SetFixed(hide_x),
                y: PositionChange::SetFixed(hide_y),
            }));

            // Restore any fullscreen window that was un-fullscreened
            if let Some(fs_id) = pop_fs_state(profile.name) {
                println!("[scratchpad] Restoring fullscreen for window {}", fs_id);
                let _ = socket.send(Request::Action(Action::FullscreenWindow {
                    id: Some(fs_id),
                }));
            }

            return Ok(());
        }

        // Window is hidden off-screen, on another workspace, or unfocused: Summon & focus
        println!("[scratchpad] Summoning '{}' (id: {}) to on-screen view", profile.name, wid);

        // If current workspace has a fullscreen window, temporarily un-fullscreen it
        if let Some(fs_win) = windows.iter().find(|w| {
            !w.is_floating
                && w.workspace_id == current_ws
                && w.layout.tile_size == (screen_w, screen_h)
        }) {
            println!("[scratchpad] Temporarily un-fullscreening window {} on workspace...", fs_win.id);
            let _ = socket.send(Request::Action(Action::FullscreenWindow {
                id: Some(fs_win.id),
            }));
            save_fs_state(profile.name, fs_win.id);
        }

        // Move to active workspace if on another workspace
        if let Some(ws_id) = current_ws {
            if window.workspace_id != Some(ws_id) {
                let _ = socket.send(Request::Action(Action::MoveWindowToWorkspace {
                    window_id: Some(wid),
                    reference: WorkspaceReferenceArg::Id(ws_id),
                    focus: true,
                }));
            }
        }

        // Ensure floating
        let _ = socket.send(Request::Action(Action::MoveWindowToFloating {
            id: Some(wid),
        }));

        // Set dimensions
        let _ = socket.send(Request::Action(Action::SetWindowWidth {
            id: Some(wid),
            change: SizeChange::SetFixed(target_w),
        }));
        let _ = socket.send(Request::Action(Action::SetWindowHeight {
            id: Some(wid),
            change: SizeChange::SetFixed(target_h),
        }));

        // Move to on-screen position
        let _ = socket.send(Request::Action(Action::MoveFloatingWindow {
            id: Some(wid),
            x: PositionChange::SetFixed(show_x),
            y: PositionChange::SetFixed(show_y),
        }));

        // Focus window
        let _ = socket.send(Request::Action(Action::FocusWindow { id: wid }));
    } else {
        // If current workspace has a fullscreen window, temporarily un-fullscreen it
        if let Some(fs_win) = windows.iter().find(|w| {
            !w.is_floating
                && w.workspace_id == current_ws
                && w.layout.tile_size == (screen_w, screen_h)
        }) {
            println!("[scratchpad] Temporarily un-fullscreening window {} on workspace...", fs_win.id);
            let _ = socket.send(Request::Action(Action::FullscreenWindow {
                id: Some(fs_win.id),
            }));
            save_fs_state(profile.name, fs_win.id);
        }

        // Window does not exist, spawn it detached
        println!("[scratchpad] Spawning scratchpad '{}': {}", profile.name, profile.command);
        let _ = Command::new("sh")
            .args(["-c", &format!("nohup {} >/dev/null 2>&1 &", profile.command)])
            .spawn()?;

        // Wait up to 3 seconds for window to appear
        for _ in 0..30 {
            thread::sleep(Duration::from_millis(100));
            if let Ok(Ok(Response::Windows(windows))) = socket.send(Request::Windows) {
                if let Some(window) = find_window(&windows, profile.app_id_pattern) {
                    let wid = window.id;

                    let _ = socket.send(Request::Action(Action::MoveWindowToFloating {
                        id: Some(wid),
                    }));
                    let _ = socket.send(Request::Action(Action::SetWindowWidth {
                        id: Some(wid),
                        change: SizeChange::SetFixed(target_w),
                    }));
                    let _ = socket.send(Request::Action(Action::SetWindowHeight {
                        id: Some(wid),
                        change: SizeChange::SetFixed(target_h),
                    }));
                    let _ = socket.send(Request::Action(Action::MoveFloatingWindow {
                        id: Some(wid),
                        x: PositionChange::SetFixed(show_x),
                        y: PositionChange::SetFixed(show_y),
                    }));
                    let _ = socket.send(Request::Action(Action::FocusWindow { id: wid }));
                    break;
                }
            }
        }
    }

    Ok(())
}
