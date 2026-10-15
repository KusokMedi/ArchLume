use std::{ffi::OsString, sync::Arc};

use smithay::{
    desktop::{PopupManager, Space, Window, WindowSurfaceType},
    input::{Seat, SeatState},
    output::Output,
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, LoopSignal, Mode, PostAction},
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason, ObjectId},
            protocol::wl_surface::WlSurface,
            Display, DisplayHandle, Resource,
        },
    },
    utils::{Logical, Point, Size},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

use crate::{
    wm::{self, SnapZone},
    workspace::Workspaces,
    CalloopData,
};

pub struct ArchLume {
    pub start_time: std::time::Instant,
    pub socket_name: OsString,
    pub display_handle: DisplayHandle,

    pub space: Space<Window>,
    pub loop_signal: LoopSignal,

    /// The primary output, stored so workspace switching can move its view.
    pub output: Option<Output>,
    /// Fixed workspaces and the currently active one.
    pub workspaces: Workspaces,
    /// Floating sizes remembered for snapped windows (un-snap restore).
    pub snap_memory: wm::SnapMemory<ObjectId>,

    // Smithay state
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<ArchLume>,
    pub data_device_state: DataDeviceState,
    pub popups: PopupManager,

    pub seat: Seat<Self>,
}

impl ArchLume {
    pub fn new(event_loop: &mut EventLoop<CalloopData>, display: Display<Self>) -> Self {
        let start_time = std::time::Instant::now();

        let dh = display.handle();

        let compositor_state = CompositorState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&dh);
        let mut seat_state = SeatState::new();
        let data_device_state = DataDeviceState::new::<Self>(&dh);
        let popups = PopupManager::default();

        // A seat groups keyboards, pointers and touch devices. A seat typically
        // has a pointer and maintains keyboard and pointer focus. For this
        // example we assume keyboard and pointer are always present; a real
        // compositor tracks hot-plugging.
        let mut seat: Seat<Self> = seat_state.new_wl_seat(&dh, "winit");
        seat.add_keyboard(Default::default(), 200, 25).unwrap();
        seat.add_pointer();

        // A space represents a two-dimensional plane onto which windows and
        // outputs are mapped. Windows get a position and stacking order through
        // mapping; outputs become views of a part of the space and are rendered
        // via `Space::render_output`.
        let space = Space::default();

        let socket_name = Self::init_wayland_listener(display, event_loop);

        // Used to stop the event loop.
        let loop_signal = event_loop.get_signal();

        Self {
            start_time,
            display_handle: dh,

            space,
            loop_signal,
            output: None,
            workspaces: Workspaces::new(1),
            snap_memory: wm::SnapMemory::new(),
            socket_name,

            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            seat_state,
            data_device_state,
            popups,
            seat,
        }
    }

    fn init_wayland_listener(
        display: Display<ArchLume>,
        event_loop: &mut EventLoop<CalloopData>,
    ) -> OsString {
        // Creates a new listening socket, automatically choosing the next
        // available `wayland` socket name.
        let listening_socket = ListeningSocketSource::new_auto().unwrap();

        let socket_name = listening_socket.socket_name().to_os_string();

        let loop_handle = event_loop.handle();

        loop_handle
            .insert_source(listening_socket, move |client_stream, _, state| {
                state
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                    .unwrap();
            })
            .expect("Failed to init the wayland event source.");

        // Insert the display itself so client events are processed by
        // wayland-server.
        loop_handle
            .insert_source(
                Generic::new(display, Interest::READ, Mode::Level),
                |_, display, state| {
                    // Safety: we never drop the display.
                    unsafe {
                        display
                            .get_mut()
                            .dispatch_clients(&mut state.state)
                            .unwrap();
                    }
                    Ok(PostAction::Continue)
                },
            )
            .unwrap();

        socket_name
    }

    pub fn surface_under(
        &self,
        pos: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        self.space
            .element_under(pos)
            .and_then(|(window, location)| {
                window
                    .surface_under(pos - location.to_f64(), WindowSurfaceType::ALL)
                    .map(|(s, p)| (s, (p + location).to_f64()))
            })
    }

    /// Apply a snap zone to `window`: move it to the target geometry and ask
    /// the client to resize via an xdg_toplevel configure. The window keeps its
    /// snapshot geometry only once the client acknowledges and commits.
    pub fn snap_window(&mut self, window: &Window, zone: SnapZone, geometry: wm::Rect) {
        let location: Point<i32, Logical> =
            (geometry.x.round() as i32, geometry.y.round() as i32).into();
        let size: Size<i32, Logical> = (
            geometry.w.round().max(1.0) as i32,
            geometry.h.round().max(1.0) as i32,
        )
            .into();

        if let Some(toplevel) = window.toplevel() {
            // Remember the floating size so dragging back to the centre can
            // restore it (only the first time the window is snapped).
            let floating = window.geometry().size;
            self.snap_memory
                .remember(toplevel.wl_surface().id(), (floating.w, floating.h));

            toplevel.with_pending_state(|state| {
                state.size = Some(size);
                if zone.is_maximized() {
                    state.states.set(xdg_toplevel::State::Maximized);
                } else {
                    state.states.unset(xdg_toplevel::State::Maximized);
                }
            });
            let _serial = toplevel.send_configure();
        }

        self.space.map_element(window.clone(), location, true);
    }

    /// Restore a snapped window to its remembered floating size (un-snap). The
    /// window stays wherever the drag left it; only its size is configured
    /// back.
    pub fn un_snap_window(&mut self, window: &Window) {
        let Some(toplevel) = window.toplevel() else {
            return;
        };
        let Some((w, h)) = self.snap_memory.restore(&toplevel.wl_surface().id()) else {
            return;
        };

        toplevel.with_pending_state(|state| {
            state.size = Some((w.max(1), h.max(1)).into());
            state.states.unset(xdg_toplevel::State::Maximized);
        });
        let _serial = toplevel.send_configure();
    }

    /// Move the output's view to the active workspace's region.
    pub fn apply_workspace_offset(&mut self) {
        if let Some(output) = self.output.clone() {
            let x = self.workspaces.current_offset();
            self.space.map_output(&output, (x, 0));
        }
    }

    /// Switch to a workspace by zero-based index. Returns whether the active
    /// workspace changed.
    pub fn switch_workspace(&mut self, index: usize) -> bool {
        if self.workspaces.switch_to(index) {
            self.apply_workspace_offset();
            true
        } else {
            false
        }
    }

    /// Move the currently focused window to workspace `index` and switch to it
    /// (AGENTS section 6.4: `Shift + Super + digit`).
    pub fn move_focused_window_to_workspace(&mut self, index: usize) -> bool {
        let Some(surface) = self.seat.get_keyboard().unwrap().current_focus() else {
            return false;
        };
        let window = self
            .space
            .elements()
            .find(|w| w.toplevel().is_some_and(|t| t.wl_surface() == &surface))
            .cloned();
        let Some(window) = window else {
            return false;
        };
        let Some(location) = self.space.element_location(&window) else {
            return false;
        };

        let target_x = self.workspaces.offset_of(index);
        self.space.map_element(window, (target_x, location.y), true);
        self.switch_workspace(index);
        true
    }
}

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
