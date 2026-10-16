//! ArchLume panel (Milestone 5).
//!
//! A Wayland client that asks the compositor for a top-layer surface via
//! wlr-layer-shell and draws a bar with `wl_shm`. It shows a live local clock
//! on the right and a brand label on the left; widgets, launcher integration
//! and interactivity build on top of this.
//!
//! Intentionally depends only on `smithay-client-toolkit`, `wayland-client`
//! and `libc` (for local time); text uses the self-contained bitmap font in
//! `font.rs`.

mod font;

use std::{
    os::fd::AsRawFd,
    time::{SystemTime, UNIX_EPOCH},
};

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

/// Panel height in logical pixels.
const PANEL_HEIGHT: u32 = 28;
/// Panel background (sRGB), the ArchLume accent.
const PANEL_COLOR: [u8; 3] = [0x1E, 0x88, 0xE5];
/// Text colour.
const TEXT_COLOR: [u8; 3] = [0xFF, 0xFF, 0xFF];
/// Bitmap-font scale.
const TEXT_SCALE: u32 = 2;
/// Horizontal padding for text.
const PAD: i32 = 10;
/// How long to wait for Wayland events before ticking the clock (ms).
const POLL_INTERVAL_MS: libc::c_int = 250;

fn main() {
    let conn = Connection::connect_to_env().expect("connect to Wayland display");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("enumerate globals");
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("wlr-layer-shell is not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

    let surface = compositor.create_surface(&qh);
    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("archlume-panel"), None);
    layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer.set_exclusive_zone(PANEL_HEIGHT as i32);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.set_size(0, PANEL_HEIGHT);
    // Commit with no buffer so the compositor sends the initial configure.
    layer.commit();

    let pool = SlotPool::new(1024 * PANEL_HEIGHT as usize * 4, &shm).expect("create shm pool");
    let mut panel = Panel {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool,
        layer,
        exit: false,
        width: 0,
        height: PANEL_HEIGHT,
        clock: String::new(),
    };

    loop {
        event_queue.dispatch_pending(&mut panel).expect("dispatch");
        if panel.exit {
            break;
        }
        // Tick the clock (redraws only when the text changes).
        let now = clock_string();
        if panel.width > 0 && now != panel.clock {
            panel.clock = now;
            panel.draw(&qh);
        }
        event_queue.flush().expect("flush");
        // Block up to a short interval, then loop so the clock keeps ticking.
        if let Some(guard) = event_queue.prepare_read() {
            let mut pfd = libc::pollfd {
                fd: guard.connection_fd().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            let ready = unsafe { libc::poll(&mut pfd, 1, POLL_INTERVAL_MS) };
            if ready > 0 && pfd.revents & libc::POLLIN != 0 {
                let _ = guard.read();
            }
        }
    }
}

/// Local wall-clock time as `HH:MM:SS` via the C library's `localtime_r`.
fn clock_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as libc::time_t)
        .unwrap_or(0);
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let ok = unsafe { libc::localtime_r(&secs, &mut tm) };
    if ok.is_null() {
        return "--:--:--".to_string();
    }
    format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
}

struct Panel {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    pool: SlotPool,
    layer: LayerSurface,
    exit: bool,
    width: u32,
    height: u32,
    clock: String,
}

impl Panel {
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        let (width, height) = (self.width, self.height);
        let stride = width as i32 * 4;
        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create shm buffer");

        let mut canvas = font::Canvas {
            data: canvas,
            width,
            height,
        };
        canvas.fill(PANEL_COLOR);

        let text_y = ((height as i32) - (7 * TEXT_SCALE) as i32) / 2;
        canvas.draw_text(PAD, text_y, TEXT_SCALE, "ARCHLUME", TEXT_COLOR);
        let clock_x = width as i32 - PAD - font::text_width(&self.clock, TEXT_SCALE) as i32;
        canvas.draw_text(clock_x, text_y, TEXT_SCALE, &self.clock, TEXT_COLOR);

        let surface = self.layer.wl_surface();
        surface.damage_buffer(0, 0, width as i32, height as i32);
        surface.frame(qh, surface.clone());
        buffer.attach_to(surface).expect("attach buffer");
        self.layer.commit();
    }
}

impl CompositorHandler for Panel {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        // Redraws are driven by the clock, not by frame callbacks.
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for Panel {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for Panel {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let (mut width, mut height) = configure.new_size;
        if width == 0 {
            width = 1024;
        }
        if height == 0 {
            height = PANEL_HEIGHT;
        }
        let resized = width != self.width || height != self.height;
        self.width = width;
        self.height = height;
        // Draw once we know our size (and on any resize).
        if resized || self.clock.is_empty() {
            self.clock = clock_string();
            self.draw(qh);
        }
    }
}

impl ShmHandler for Panel {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

delegate_compositor!(Panel);
delegate_output!(Panel);
delegate_shm!(Panel);
delegate_layer!(Panel);
delegate_registry!(Panel);

impl ProvidesRegistryState for Panel {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}
