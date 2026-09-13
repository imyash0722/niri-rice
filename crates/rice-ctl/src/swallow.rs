use niri_ipc::socket::Socket;
use niri_ipc::{Action, ColumnDisplay, Event, Request, Response, Window};
use std::collections::{HashMap, HashSet};
use std::fs;

/// Helper to read PPID of a process from /proc/<pid>/stat
fn get_ppid(pid: i32) -> Option<i32> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // Field 2 (comm) can contain spaces and parentheses: e.g. "1234 (bash foo) S 1000 ..."
    let rparen = stat.rfind(')')?;
    let rest = &stat[rparen + 1..];
    let mut fields = rest.split_whitespace();
    fields.next()?; // state
    let ppid_str = fields.next()?;
    ppid_str.parse().ok()
}

/// Collect ancestor PIDs by climbing /proc/<pid>/stat
fn get_ancestor_pids(mut pid: i32) -> Vec<i32> {
    let mut ancestors = Vec::new();
    let mut seen = HashSet::new();
    seen.insert(pid);

    while let Some(ppid) = get_ppid(pid) {
        if ppid <= 1 || seen.contains(&ppid) {
            break;
        }
        ancestors.push(ppid);
        seen.insert(ppid);
        pid = ppid;
    }
    ancestors
}

/// Check if an app_id / title represents a terminal
fn is_terminal_window(window: &Window) -> bool {
    let app_id = window.app_id.as_deref().unwrap_or("").to_lowercase();
    app_id.contains("kitty")
        || app_id.contains("alacritty")
        || app_id.contains("foot")
        || app_id.contains("wezterm")
        || app_id.contains("terminal")
}

/// Check if a window should be swallowed (media viewers, documents, etc.)
fn is_swallowable_child(window: &Window) -> bool {
    let app_id = window.app_id.as_deref().unwrap_or("").to_lowercase();
    let title = window.title.as_deref().unwrap_or("").to_lowercase();

    let targets = [
        "mpv", "imv", "zathura", "feh", "nsxiv", "qimgv",
        "viewnior", "eog", "loupe", "evince", "gnuplot", "freecad"
    ];

    targets.iter().any(|t| app_id.contains(t) || title.contains(t))
}

pub fn run_swallow_daemon() -> Result<(), Box<dyn std::error::Error>> {
    println!("[swallow] Starting window swallowing daemon for Niri...");
    let mut event_socket = Socket::connect()?;

    let reply = event_socket.send(Request::EventStream)?;
    if !matches!(reply, Ok(Response::Handled)) {
        return Err("Failed to start niri IPC event stream".into());
    }

    let mut read_event = event_socket.read_events();

    // Map of child_window_id -> (parent_window_id, original_column_display)
    let mut swallowed_map: HashMap<u64, (u64, ColumnDisplay)> = HashMap::new();

    while let Ok(event) = read_event() {
        match event {
            Event::WindowOpenedOrChanged { window } => {
                let child_id = window.id;

                // Skip if already tracked as swallowed
                if swallowed_map.contains_key(&child_id) {
                    continue;
                }

                // Check if window is a candidate to be swallowed
                if let Some(child_pid) = window.pid {
                    if is_swallowable_child(&window) {
                        let ancestors = get_ancestor_pids(child_pid);
                        if ancestors.is_empty() {
                            continue;
                        }

                        // Query active windows to find parent terminal
                        let mut cmd_socket = match Socket::connect() {
                            Ok(s) => s,
                            Err(_) => continue,
                        };

                        let windows_reply = cmd_socket.send(Request::Windows);
                        if let Ok(Ok(Response::Windows(windows))) = windows_reply {
                            let parent = windows.iter().find(|w| {
                                if let Some(w_pid) = w.pid {
                                    ancestors.contains(&w_pid) && is_terminal_window(w)
                                } else {
                                    false
                                }
                            });

                            if let Some(parent_win) = parent {
                                let parent_id = parent_win.id;
                                println!(
                                    "[swallow] Swallowing child {} ({:?}) into parent terminal {} ({:?})",
                                    child_id, window.app_id, parent_id, parent_win.app_id
                                );

                                // Default to normal column display before tabbing
                                let original_display = ColumnDisplay::Normal;
                                swallowed_map.insert(child_id, (parent_id, original_display));

                                // 1. Focus parent terminal
                                let _ = cmd_socket.send(Request::Action(Action::FocusWindow {
                                    id: parent_id,
                                }));

                                // 2. Set column display to Tabbed
                                let _ = cmd_socket.send(Request::Action(Action::SetColumnDisplay {
                                    display: ColumnDisplay::Tabbed,
                                }));

                                // 3. Move child into tiling if floating
                                if window.is_floating {
                                    let _ = cmd_socket.send(Request::Action(Action::MoveWindowToTiling {
                                        id: Some(child_id),
                                    }));
                                }

                                // 4. Consume child into parent's column
                                let _ = cmd_socket.send(Request::Action(Action::ConsumeOrExpelWindowLeft {
                                    id: Some(child_id),
                                }));

                                // 5. Focus swallowed child window
                                let _ = cmd_socket.send(Request::Action(Action::FocusWindow {
                                    id: child_id,
                                }));
                            }
                        }
                    }
                }
            }
            Event::WindowClosed { id } => {
                if let Some((parent_id, original_display)) = swallowed_map.remove(&id) {
                    println!("[swallow] Swallowed window {} closed, restoring parent {}", id, parent_id);

                    if let Ok(mut cmd_socket) = Socket::connect() {
                        // Focus parent terminal
                        let _ = cmd_socket.send(Request::Action(Action::FocusWindow {
                            id: parent_id,
                        }));

                        // If no other swallowed children belong to this parent, restore normal column display
                        let has_other_children = swallowed_map.values().any(|(p, _)| *p == parent_id);
                        if !has_other_children {
                            let _ = cmd_socket.send(Request::Action(Action::SetColumnDisplay {
                                display: original_display,
                            }));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
