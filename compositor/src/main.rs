//! ArchLume compositor.
//!
//! A minimal but real Smithay-based Wayland compositor. It currently runs on
//! the winit backend (nested inside another Wayland session) and implements
//! xdg-shell toplevels/popups, pointer/keyboard input, focus, ten fixed
//! workspaces, floating window management with drag-to-edge snapping and
//! un-snap restore, and a line-based control socket for the shell and tests.
//!
//! The native DRM/libseat backend, gestures, and snap preview are future work
//! (see `docs/ROADMAP.md` and `docs/NATIVE-BACKEND.md`).
//!
//! Adapted from the Smithay `smallvil` example (MIT-licensed) at tag `v0.7.0`
//! and reorganized for ArchLume. See `docs/DECISIONS.md` (ADR-0002).

#![allow(irrefutable_let_patterns)]

mod control;
mod gestures;
mod grabs;
mod handlers;
mod input;
mod state;
mod stats;
mod winit;
mod wm;
mod workspace;

use smithay::reexports::{
    calloop::{channel, EventLoop},
    wayland_server::{Display, DisplayHandle},
};
pub use state::ArchLume;

pub struct CalloopData {
    state: ArchLume,
    display_handle: DisplayHandle,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    let mut event_loop: EventLoop<CalloopData> = EventLoop::try_new()?;

    let display: Display<ArchLume> = Display::new()?;
    let display_handle = display.handle();
    let state = ArchLume::new(&mut event_loop, display);

    let mut data = CalloopData {
        state,
        display_handle,
    };

    crate::winit::init_winit(&mut event_loop, &mut data)?;

    // Control socket: lets the shell and tests drive the compositor (see
    // control.rs). Requests are delivered on the event-loop thread, so handlers
    // can mutate compositor state directly.
    let (control_tx, control_rx) = channel::channel::<control::Command>();
    event_loop
        .handle()
        .insert_source(control_rx, |event, _, data| {
            if let channel::Event::Msg(command) = event {
                control::handle(command, &mut data.state);
            }
        })?;
    let _control_server = control::Server::start(control::default_socket_path(), control_tx)?;
    std::env::set_var("ARCHLUME_CONTROL", &_control_server.path);

    // Optionally launch a client for quick manual testing:
    //   archlume-compositor --command foot
    let mut args = std::env::args().skip(1);
    let flag = args.next();
    let arg = args.next();

    match (flag.as_deref(), arg) {
        (Some("-c") | Some("--command"), Some(command)) => {
            std::process::Command::new(command).spawn().ok();
        }
        _ => {
            // Best-effort default: launch the first terminal found on the
            // system. Ignored if none is installed.
            for term in ["foot", "alacritty", "kitty", "weston-terminal"] {
                if std::process::Command::new(term).spawn().is_ok() {
                    break;
                }
            }
        }
    }

    event_loop.run(None, &mut data, move |_| {
        // ArchLume compositor is running.
    })?;

    Ok(())
}
