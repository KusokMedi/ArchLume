use crate::{grabs::resize_grab, state::ClientState, ArchLume};
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor, delegate_shm,
    reexports::wayland_server::{
        protocol::{wl_buffer, wl_surface::WlSurface},
        Client,
    },
    wayland::{
        buffer::BufferHandler,
        compositor::{
            get_parent, is_sync_subsurface, CompositorClientState, CompositorHandler,
            CompositorState,
        },
        shm::{ShmHandler, ShmState},
    },
};

use super::xdg_shell;

impl CompositorHandler for ArchLume {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(window) = self
                .space
                .elements()
                .find(|w| w.toplevel().unwrap().wl_surface() == &root)
            {
                window.on_commit();
            }
        };

        xdg_shell::handle_commit(&mut self.popups, &self.space, surface);
        resize_grab::handle_commit(&mut self.space, surface);
        handle_layer_commit(self, surface);
    }
}

/// After a layer surface commits, re-arrange its output's layer map and, on the
/// first commit, send the initial configure (the protocol requires it to follow
/// the client's first commit so the client can request a size first).
fn handle_layer_commit(state: &mut ArchLume, surface: &WlSurface) {
    use smithay::wayland::compositor::with_states;
    use smithay::{
        desktop::{layer_map_for_output, WindowSurfaceType},
        wayland::shell::wlr_layer::LayerSurfaceData,
    };

    let Some(output) = state.space.outputs().find(|output| {
        layer_map_for_output(output)
            .layer_for_surface(surface, WindowSurfaceType::TOPLEVEL)
            .is_some()
    }) else {
        return;
    };
    let output = output.clone();

    let initial_configure_sent = with_states(surface, |states| {
        states
            .data_map
            .get::<LayerSurfaceData>()
            .map(|data| data.lock().unwrap().initial_configure_sent)
            .unwrap_or(true)
    });

    let mut map = layer_map_for_output(&output);
    map.arrange();
    if !initial_configure_sent {
        if let Some(layer) = map.layer_for_surface(surface, WindowSurfaceType::TOPLEVEL) {
            layer.layer_surface().send_configure();
        }
    }
    drop(map);

    // A layer may now want (or no longer want) exclusive keyboard focus.
    state.update_layer_keyboard_focus();
}

impl BufferHandler for ArchLume {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl ShmHandler for ArchLume {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

delegate_compositor!(ArchLume);
delegate_shm!(ArchLume);
