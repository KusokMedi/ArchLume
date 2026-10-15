//! Control socket for the compositor.
//!
//! A small line-based protocol over a UNIX socket, used by the ArchLume shell
//! and by automated tests to drive and inspect the compositor without needing a
//! graphical session. It is intentionally dependency-free and versionless for
//! now; a richer, versioned protocol can replace it later (see
//! `docs/DECISIONS.md`).
//!
//! Socket path: `$XDG_RUNTIME_DIR/archlume-control.sock` (the path is also
//! exported as `ARCHLUME_CONTROL` so child clients can find it).
//!
//! Protocol (one request per line, one response per line):
//!
//! | request        | response                          |
//! |----------------|-----------------------------------|
//! | `ping`         | `ok pong`                         |
//! | `workspace`    | `ok <n>` (1-based workspace)      |
//! | `switch <n>`   | `ok <n> changed|same`             |
//! | `move <n>`     | `ok <n>` or `err no focused window` |
//! | `windows`      | `ok <count>`                      |
//! | `window <i>`   | `ok <x> <y> <w> <h>` or `err no such window` |
//! | `close <i>`    | `ok` or `err no such window`      |
//!
//! `err ...` responses indicate a failed request; unknown commands reply
//! `err unknown command`.

use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::mpsc::Sender as ReplySender,
};

use smithay::reexports::calloop::channel::Sender as ControlSender;

use crate::ArchLume;

/// Channel used by a worker to send a single response line back to a client.
pub type Reply = ReplySender<String>;

/// A control request, delivered to the event loop together with its reply
/// channel.
pub enum Command {
    Ping(Reply),
    CurrentWorkspace(Reply),
    SwitchWorkspace { number: usize, reply: Reply },
    MoveFocusedWindow { number: usize, reply: Reply },
    ListWindows(Reply),
    WindowGeometry { index: usize, reply: Reply },
    CloseWindow { index: usize, reply: Reply },
}

/// Running control server. Keeps the socket path; the accept thread is
/// detached and lives for the process.
pub struct Server {
    pub path: PathBuf,
}

impl Server {
    /// Bind the socket and start accepting connections.
    pub fn start(path: PathBuf, tx: ControlSender<Command>) -> std::io::Result<Self> {
        // Remove a stale socket from a previous run.
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path)?;

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let tx = tx.clone();
                std::thread::spawn(move || handle_connection(stream, tx));
            }
        });

        Ok(Self { path })
    }
}

/// Default control socket path for the current process.
pub fn default_socket_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir).join("archlume-control.sock");
        }
    }
    std::env::temp_dir().join(format!("archlume-control-{}.sock", std::process::id()))
}

fn handle_connection(stream: UnixStream, tx: ControlSender<Command>) {
    let Ok(read_half) = stream.try_clone() else {
        return;
    };
    let reader = BufReader::new(read_half);
    let mut writer = stream;

    for line in reader.lines() {
        let Ok(line) = line else { break };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let (reply_tx, reply_rx) = std::sync::mpsc::channel::<String>();
        let Some(command) = parse(line, reply_tx) else {
            let _ = writeln!(writer, "err unknown command");
            continue;
        };

        if tx.send(command).is_err() {
            break;
        }
        match reply_rx.recv() {
            Ok(response) => {
                if writeln!(writer, "{response}").is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn parse(line: &str, reply: Reply) -> Option<Command> {
    let mut parts = line.split_whitespace();
    match parts.next()? {
        "ping" => Some(Command::Ping(reply)),
        "workspace" => Some(Command::CurrentWorkspace(reply)),
        "windows" => Some(Command::ListWindows(reply)),
        "switch" => {
            let number = parts.next()?.parse::<usize>().ok()?;
            Some(Command::SwitchWorkspace { number, reply })
        }
        "move" => {
            let number = parts.next()?.parse::<usize>().ok()?;
            Some(Command::MoveFocusedWindow { number, reply })
        }
        "window" => {
            let index = parts.next()?.parse::<usize>().ok()?;
            Some(Command::WindowGeometry { index, reply })
        }
        "close" => {
            let index = parts.next()?.parse::<usize>().ok()?;
            Some(Command::CloseWindow { index, reply })
        }
        _ => None,
    }
}

/// Execute a control command against the compositor and send the reply. Called
/// on the event-loop thread.
pub fn handle(command: Command, state: &mut ArchLume) {
    match command {
        Command::Ping(reply) => {
            let _ = reply.send("ok pong".to_string());
        }
        Command::CurrentWorkspace(reply) => {
            let _ = reply.send(format!("ok {}", workspace_number(state)));
        }
        Command::SwitchWorkspace { number, reply } => {
            let changed = state.switch_workspace(index_of(number));
            let _ = reply.send(format!(
                "ok {} {}",
                workspace_number(state),
                if changed { "changed" } else { "same" }
            ));
        }
        Command::MoveFocusedWindow { number, reply } => {
            if state.move_focused_window_to_workspace(index_of(number)) {
                let _ = reply.send(format!("ok {}", workspace_number(state)));
            } else {
                let _ = reply.send("err no focused window".to_string());
            }
        }
        Command::ListWindows(reply) => {
            let count = state.space.elements().count();
            let _ = reply.send(format!("ok {count}"));
        }
        Command::WindowGeometry { index, reply } => {
            let geometry = state.space.elements().nth(index).and_then(|window| {
                let location = state.space.element_location(window)?;
                let size = window.geometry().size;
                Some((location.x, location.y, size.w, size.h))
            });
            match geometry {
                Some((x, y, w, h)) => {
                    let _ = reply.send(format!("ok {x} {y} {w} {h}"));
                }
                None => {
                    let _ = reply.send("err no such window".to_string());
                }
            }
        }
        Command::CloseWindow { index, reply } => {
            let window = state.space.elements().nth(index).cloned();
            match window.and_then(|window| window.toplevel().map(|t| t.send_close())) {
                Some(()) => {
                    let _ = reply.send("ok".to_string());
                }
                None => {
                    let _ = reply.send("err no such window".to_string());
                }
            }
        }
    }
}

/// User-facing, 1-based workspace number of the active workspace.
fn workspace_number(state: &ArchLume) -> usize {
    state.workspaces.current() + 1
}

/// Convert a 1-based workspace number from the protocol to a 0-based index.
fn index_of(number: usize) -> usize {
    number.saturating_sub(1)
}
