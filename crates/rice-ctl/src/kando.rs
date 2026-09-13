use niri_ipc::socket::Socket;
use niri_ipc::{Action, Request, Response};
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Deserialize)]
struct IpcInfo {
    port: u16,
}

fn get_kando_port() -> Option<u16> {
    let home = std::env::var("HOME").ok()?;
    let info_path = PathBuf::from(home).join(".config/kando/ipc-info.json");
    if let Ok(content) = fs::read_to_string(&info_path) {
        if let Ok(info) = serde_json::from_str::<IpcInfo>(&content) {
            return Some(info.port);
        }
    }
    None
}

/// Send WebSocket text frame over stream
fn send_ws_frame(stream: &mut TcpStream, payload: &str) -> std::io::Result<()> {
    let bytes = payload.as_bytes();
    let len = bytes.len();
    let mut header = Vec::new();
    header.push(0x81); // Fin + text opcode

    let mask_key: [u8; 4] = [0x41, 0x47, 0x59, 0x31];
    if len <= 125 {
        header.push(0x80 | (len as u8));
    } else if len <= 65535 {
        header.push(0x80 | 126);
        header.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        header.push(0x80 | 127);
        header.extend_from_slice(&(len as u64).to_be_bytes());
    }
    header.extend_from_slice(&mask_key);

    let mut masked_payload = Vec::with_capacity(len);
    for (i, &b) in bytes.iter().enumerate() {
        masked_payload.push(b ^ mask_key[i % 4]);
    }

    stream.write_all(&header)?;
    stream.write_all(&masked_payload)?;
    stream.flush()?;
    Ok(())
}

/// Perform WebSocket client handshake
fn ws_connect(port: u16) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(Duration::from_millis(500)))?;

    let handshake = format!(
        "GET / HTTP/1.1\r\n\
         Host: 127.0.0.1:{port}\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
         Sec-WebSocket-Version: 13\r\n\r\n"
    );
    stream.write_all(handshake.as_bytes())?;

    let mut response = [0u8; 1024];
    let n = stream.read(&mut response)?;
    let resp_str = String::from_utf8_lossy(&response[..n]);
    if !resp_str.starts_with("HTTP/1.1 101") {
        return Err("Failed WebSocket handshake with Kando".into());
    }

    Ok(stream)
}

pub fn show_menu(menu_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let port = match get_kando_port() {
        Some(p) => p,
        None => {
            // Spawn kando daemon if not running
            Command::new("kando")
                .env("XDG_CURRENT_DESKTOP", "niri")
                .spawn()?;
            thread::sleep(Duration::from_millis(600));
            get_kando_port().ok_or("Kando IPC port not available")?
        }
    };

    let mut stream = ws_connect(port)?;

    // Check if focused window is currently fullscreen in Niri
    let mut niri_socket = Socket::connect().ok();
    let mut fullscreen_window_id: Option<u64> = None;

    if let Some(ref mut socket) = niri_socket {
        let screen_dim = match socket.send(Request::FocusedOutput) {
            Ok(Ok(Response::FocusedOutput(Some(out)))) => {
                out.logical.map(|l| (l.width as f64, l.height as f64))
            }
            _ => None,
        };

        if let Some((sw, sh)) = screen_dim {
            if let Ok(Ok(Response::FocusedWindow(Some(win)))) = socket.send(Request::FocusedWindow) {
                if !win.is_floating && win.layout.tile_size == (sw, sh) {
                    fullscreen_window_id = Some(win.id);
                    let _ = socket.send(Request::Action(Action::FullscreenWindow { id: Some(win.id) }));
                }
            }
        }
    }

    // Send show-menu frame
    let msg = format!(r#"{{"type":"show-menu","name":"{}"}}"#, menu_name);
    send_ws_frame(&mut stream, &msg)?;

    // If we temporarily un-fullscreened a window, wait for Kando to close, then restore fullscreen
    if let Some(win_id) = fullscreen_window_id {
        // Wait on stream until closed or interaction message received
        stream.set_read_timeout(Some(Duration::from_secs(60)))?;
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf);

        if let Some(ref mut socket) = niri_socket {
            let _ = socket.send(Request::Action(Action::FullscreenWindow { id: Some(win_id) }));
        }
    }

    Ok(())
}
