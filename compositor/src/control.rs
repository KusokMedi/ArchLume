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
//! | `stats`        | `ok frames <n> mean <us> max <us>` |
//! | `preview <zone>` | `ok <x> <y> <w> <h>` or `ok off` |
//! | `gesture <fingers> <dx> <dy>` | `ok prev|next <n>` or `ok none` |
//! | `layers`       | `ok <count>` (mapped layer surfaces) |
//! | `focused`      | `ok <title>` or `ok -` (focused window title) |
//! | `launcher`     | `ok on\|off` (query the launcher state) |
//! | `launcher on\|off\|toggle` | `ok on\|off` (set/toggle the launcher) |
//! | `key <name>`   | `ok` (inject a key press+release; automation/debug) |
//!
//! `err ...` responses indicate a failed request; unknown commands reply
//! `err unknown command`.
//!
//! `key` accepts a single character (`a`-`z`, `0`-`9`) or a name: `space`,
//! `backspace`, `return`, `escape`, `tab`. It synthesises the event through the
//! seat's keyboard, so it is delivered to whichever surface currently holds
//! keyboard focus — used by automated tests to drive the shell without a
//! physical input device.

use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::mpsc::Sender as ReplySender,
};

use smithay::{
    backend::input::KeyState, desktop::layer_map_for_output, input::keyboard::FilterResult,
    reexports::calloop::channel::Sender as ControlSender, utils::SERIAL_COUNTER,
};

use crate::{
    gestures::{self, GestureAction},
    wm::SnapZone,
    ArchLume,
};

/// Channel used by a worker to send a single response line back to a client.
pub type Reply = ReplySender<String>;

/// A control request, delivered to the event loop together with its reply
/// channel.
pub enum Command {
    Ping(Reply),
    CurrentWorkspace(Reply),
    SwitchWorkspace {
        number: usize,
        reply: Reply,
    },
    MoveFocusedWindow {
        number: usize,
        reply: Reply,
    },
    ListWindows(Reply),
    WindowGeometry {
        index: usize,
        reply: Reply,
    },
    CloseWindow {
        index: usize,
        reply: Reply,
    },
    FrameStats(Reply),
    SnapPreview {
        zone: Option<SnapZone>,
        reply: Reply,
    },
    Gesture {
        fingers: u8,
        dx: f64,
        dy: f64,
        reply: Reply,
    },
    LayerCount(Reply),
    FocusedTitle(Reply),
    LauncherState(Reply),
    SetLauncher(Option<bool>, Reply),
    InjectKey {
        code: u32,
        reply: Reply,
    },
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

/// A parsed request, before it is paired with a reply channel. Kept separate
/// from [`Command`] so parsing is pure and unit-testable.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Request {
    Ping,
    CurrentWorkspace,
    ListWindows,
    Switch(usize),
    Move(usize),
    Window(usize),
    Close(usize),
    Stats,
    Preview(Option<SnapZone>),
    Gesture(u8, f64, f64),
    Layers,
    Focused,
    Launcher,
    SetLauncher(Option<bool>),
    Key(u32),
}

fn parse_request(line: &str) -> Option<Request> {
    fn number(parts: &mut std::str::SplitWhitespace<'_>) -> Option<usize> {
        parts.next()?.parse().ok()
    }
    let mut parts = line.split_whitespace();
    match parts.next()? {
        "ping" => Some(Request::Ping),
        "workspace" => Some(Request::CurrentWorkspace),
        "windows" => Some(Request::ListWindows),
        "layers" => Some(Request::Layers),
        "focused" => Some(Request::Focused),
        "launcher" => Some(match parts.next() {
            None => Request::Launcher,
            Some("on") => Request::SetLauncher(Some(true)),
            Some("off") => Request::SetLauncher(Some(false)),
            Some("toggle") => Request::SetLauncher(None),
            Some(_) => return None,
        }),
        "key" => Some(Request::Key(keycode(parts.next()?)?)),
        "switch" => Some(Request::Switch(number(&mut parts)?)),
        "move" => Some(Request::Move(number(&mut parts)?)),
        "window" => Some(Request::Window(number(&mut parts)?)),
        "close" => Some(Request::Close(number(&mut parts)?)),
        "stats" => Some(Request::Stats),
        "preview" => Some(Request::Preview(parse_zone(parts.next().unwrap_or("off"))?)),
        "gesture" => Some(Request::Gesture(
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
        )),
        _ => None,
    }
}

/// `"on"`/`"off"` for a boolean flag.
fn on_off(value: bool) -> &'static str {
    if value {
        "on"
    } else {
        "off"
    }
}

/// Map a zone name to an optional snap zone. `off`/`none` clear the preview;
/// unknown names are rejected.
fn parse_zone(name: &str) -> Option<Option<SnapZone>> {
    Some(match name {
        "off" | "none" => None,
        "left" => Some(SnapZone::LeftHalf),
        "right" => Some(SnapZone::RightHalf),
        "max" | "maximize" => Some(SnapZone::Maximize),
        "top-left" => Some(SnapZone::TopLeft),
        "top-right" => Some(SnapZone::TopRight),
        "bottom-left" => Some(SnapZone::BottomLeft),
        "bottom-right" => Some(SnapZone::BottomRight),
        _ => return None,
    })
}

/// Map a character or key name to a Linux evdev keycode. Only the keys needed
/// for automation are supported; anything else is rejected.
fn keycode(name: &str) -> Option<u32> {
    Some(match name {
        "space" => 57,
        "backspace" => 14,
        "return" | "enter" => 28,
        "escape" | "esc" => 1,
        "tab" => 15,
        "a" => 30,
        "b" => 48,
        "c" => 46,
        "d" => 32,
        "e" => 18,
        "f" => 33,
        "g" => 34,
        "h" => 35,
        "i" => 23,
        "j" => 36,
        "k" => 37,
        "l" => 38,
        "m" => 50,
        "n" => 49,
        "o" => 24,
        "p" => 25,
        "q" => 16,
        "r" => 19,
        "s" => 31,
        "t" => 20,
        "u" => 22,
        "v" => 47,
        "w" => 17,
        "x" => 45,
        "y" => 21,
        "z" => 44,
        "1" => 2,
        "2" => 3,
        "3" => 4,
        "4" => 5,
        "5" => 6,
        "6" => 7,
        "7" => 8,
        "8" => 9,
        "9" => 10,
        "0" => 11,
        _ => return None,
    })
}

/// Milliseconds since the process start, for synthesised input timestamps.
fn now_msec() -> u32 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u32
}

fn parse(line: &str, reply: Reply) -> Option<Command> {
    Some(match parse_request(line)? {
        Request::Ping => Command::Ping(reply),
        Request::CurrentWorkspace => Command::CurrentWorkspace(reply),
        Request::ListWindows => Command::ListWindows(reply),
        Request::Switch(number) => Command::SwitchWorkspace { number, reply },
        Request::Move(number) => Command::MoveFocusedWindow { number, reply },
        Request::Window(index) => Command::WindowGeometry { index, reply },
        Request::Close(index) => Command::CloseWindow { index, reply },
        Request::Stats => Command::FrameStats(reply),
        Request::Preview(zone) => Command::SnapPreview { zone, reply },
        Request::Gesture(fingers, dx, dy) => Command::Gesture {
            fingers,
            dx,
            dy,
            reply,
        },
        Request::Layers => Command::LayerCount(reply),
        Request::Focused => Command::FocusedTitle(reply),
        Request::Launcher => Command::LauncherState(reply),
        Request::SetLauncher(open) => Command::SetLauncher(open, reply),
        Request::Key(code) => Command::InjectKey { code, reply },
    })
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
        Command::FrameStats(reply) => {
            let stats = &state.frame_stats;
            let _ = reply.send(format!(
                "ok frames {} mean {} max {}",
                stats.frames(),
                stats.mean_us(),
                stats.max_us()
            ));
        }
        Command::SnapPreview { zone, reply } => match state.set_snap_preview(zone) {
            Some(rect) => {
                let _ = reply.send(format!(
                    "ok {} {} {} {}",
                    rect.x as i32, rect.y as i32, rect.w as i32, rect.h as i32
                ));
            }
            None => {
                let _ = reply.send("ok off".to_string());
            }
        },
        Command::Gesture {
            fingers,
            dx,
            dy,
            reply,
        } => {
            let action = gestures::direction_for(dx, dy, gestures::DEFAULT_THRESHOLD)
                .map(|direction| gestures::action_for(fingers, direction))
                .unwrap_or(GestureAction::None);
            match action {
                GestureAction::PrevWorkspace => {
                    state.switch_workspace_by(-1);
                    let _ = reply.send(format!("ok prev {}", workspace_number(state)));
                }
                GestureAction::NextWorkspace => {
                    state.switch_workspace_by(1);
                    let _ = reply.send(format!("ok next {}", workspace_number(state)));
                }
                GestureAction::None => {
                    let _ = reply.send("ok none".to_string());
                }
            }
        }
        Command::LayerCount(reply) => {
            let count: usize = state
                .space
                .outputs()
                .map(|output| layer_map_for_output(output).layers().count())
                .sum();
            let _ = reply.send(format!("ok {count}"));
        }
        Command::LauncherState(reply) => {
            let _ = reply.send(format!("ok {}", on_off(state.launcher_open)));
        }
        Command::SetLauncher(open, reply) => {
            let is_open = match open {
                Some(open) => state.set_launcher(open),
                None => state.toggle_launcher(),
            };
            let _ = reply.send(format!("ok {}", on_off(is_open)));
        }
        Command::InjectKey { code, reply } => {
            let time = now_msec();
            for key_state in [KeyState::Pressed, KeyState::Released] {
                let serial = SERIAL_COUNTER.next_serial();
                // `Keycode` is an XKB keycode, which is the evdev code + 8
                // (the same offset the winit backend applies).
                state.seat.get_keyboard().unwrap().input::<(), _>(
                    state,
                    (code + 8).into(),
                    key_state,
                    serial,
                    time,
                    |_, _, _| FilterResult::Forward,
                );
            }
            let _ = reply.send("ok".to_string());
        }
        Command::FocusedTitle(reply) => {
            let title = state
                .focused_window_title()
                .map(|t| t.replace(['\n', '\r'], " "))
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| "-".to_string());
            let _ = reply.send(format!("ok {title}"));
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

#[cfg(test)]
mod tests {
    use super::parse_request;
    use super::Request::*;
    use crate::wm::SnapZone;

    #[test]
    fn parses_argumentless_commands() {
        assert_eq!(parse_request("ping"), Some(Ping));
        assert_eq!(parse_request("workspace"), Some(CurrentWorkspace));
        assert_eq!(parse_request("windows"), Some(ListWindows));
        assert_eq!(parse_request("layers"), Some(Layers));
        assert_eq!(parse_request("focused"), Some(Focused));
        assert_eq!(parse_request("launcher"), Some(Launcher));
        assert_eq!(parse_request("launcher on"), Some(SetLauncher(Some(true))));
        assert_eq!(
            parse_request("launcher off"),
            Some(SetLauncher(Some(false)))
        );
        assert_eq!(parse_request("launcher toggle"), Some(SetLauncher(None)));
        assert_eq!(parse_request("launcher maybe"), None);
        assert_eq!(parse_request("stats"), Some(Stats));
        // Surrounding whitespace is ignored.
        assert_eq!(parse_request("  ping  "), Some(Ping));
    }

    #[test]
    fn parses_key_injection() {
        assert_eq!(parse_request("key z"), Some(Key(44)));
        assert_eq!(parse_request("key 4"), Some(Key(5)));
        assert_eq!(parse_request("key return"), Some(Key(28)));
        assert_eq!(parse_request("key backspace"), Some(Key(14)));
        assert_eq!(parse_request("key space"), Some(Key(57)));
        assert_eq!(parse_request("key bogus"), None);
        assert_eq!(parse_request("key"), None);
    }

    #[test]
    fn parses_commands_with_arguments() {
        assert_eq!(parse_request("switch 5"), Some(Switch(5)));
        assert_eq!(parse_request("move 10"), Some(Move(10)));
        assert_eq!(parse_request("window 3"), Some(Window(3)));
        assert_eq!(parse_request("close 0"), Some(Close(0)));
        // Extra tokens are ignored.
        assert_eq!(parse_request("switch 5 extra"), Some(Switch(5)));
    }

    #[test]
    fn parses_preview_zones() {
        assert_eq!(
            parse_request("preview left"),
            Some(Preview(Some(SnapZone::LeftHalf)))
        );
        assert_eq!(
            parse_request("preview right"),
            Some(Preview(Some(SnapZone::RightHalf)))
        );
        assert_eq!(
            parse_request("preview max"),
            Some(Preview(Some(SnapZone::Maximize)))
        );
        assert_eq!(parse_request("preview off"), Some(Preview(None)));
        // A bare `preview` clears the overlay.
        assert_eq!(parse_request("preview"), Some(Preview(None)));
        assert_eq!(parse_request("preview bogus"), None);
    }

    #[test]
    fn parses_gestures() {
        assert_eq!(
            parse_request("gesture 3 -0.5 0"),
            Some(Gesture(3, -0.5, 0.0))
        );
        assert_eq!(parse_request("gesture 4 0 0.4"), Some(Gesture(4, 0.0, 0.4)));
        assert_eq!(parse_request("gesture 3 -0.5"), None);
        assert_eq!(parse_request("gesture x 0 0"), None);
        assert_eq!(parse_request("gesture 3 a b"), None);
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(parse_request(""), None);
        assert_eq!(parse_request("bogus"), None);
        assert_eq!(parse_request("switch"), None);
        assert_eq!(parse_request("switch abc"), None);
        assert_eq!(parse_request("switch -1"), None);
    }
}
